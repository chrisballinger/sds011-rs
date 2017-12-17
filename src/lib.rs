#[macro_use] extern crate enum_primitive;
extern crate num;
extern crate serial;

use num::FromPrimitive;

use std::path::Path;
use std::cell::RefCell;
use std::time::Duration;
use std::io::{Read, Write, ErrorKind};
use std::time::SystemTime;

use serial::SerialPort;

// Constants
enum_from_primitive! {
#[repr(u8)]
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Serial {
    Start = 0xAA,
    End = 0xAB,
    SendByte = 0xB4,
    ResponseByte = 0xC5,
    ReceiveByte = 0xC0,
    CommandTerminator = 0xFF
}}

const RESPONSE_LENGTH: usize = 10;
const COMMAND_LENGTH: usize = 19;

// Enumeration of SDS011 commands
enum_from_primitive! {
#[repr(u8)]
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Command {
    ReportMode = 2,
    Request = 4,
    DeviceId = 5,
    WorkState = 6,
    Firmware = 7,
    DutyCycle = 8
}}

pub struct SendData {
    command: Command,
    mode: CommandMode,
    data: Vec<u8>
}

#[derive(Debug)]
pub struct SensorMeasurement {
    pm2_5: f32,
    pm10: f32,
    timestamp: SystemTime
}

impl SendData {
    pub fn get_duty_cycle() -> Self {
        SendData::new(Command::DutyCycle, CommandMode::Getting, 0)
    }

    pub fn get_report_mode() -> Self {
        SendData::new(Command::ReportMode, CommandMode::Getting, 0)
    }

    pub fn get_firmware() -> Self {
        SendData::new(Command::Firmware, CommandMode::Getting, 0)
    }


    pub fn set_duty_cycle(value: u8) -> Self {
        SendData::new(Command::DutyCycle, CommandMode::Setting, value)
    }

    pub fn set_work_state(work_state: WorkState) -> Self {
        SendData::new(Command::WorkState, CommandMode::Setting, work_state as u8)
    }



    pub fn new(command: Command, mode: CommandMode, value: u8) -> Self {
        let data = vec![value];
        SendData { command, mode, data }
    }

    pub fn to_command_data(&self) -> Vec<u8> {
        let command = self.command as u8;
        let mode = self.mode as u8;
        let mut bytes_to_send: Vec<u8> = vec![Serial::Start as u8, Serial::SendByte as u8, command, mode];

        for i in 0..11 {
            if i < self.data.len() {
                let byte = self.data[i];
                bytes_to_send.push(byte);
            } else {
                bytes_to_send.push(0);
            }
        }
        bytes_to_send.push(Serial::CommandTerminator as u8);
        bytes_to_send.push(Serial::CommandTerminator as u8);

        let checksum = Sensor::generate_checksum(&bytes_to_send).unwrap();
        bytes_to_send.push(checksum);
        bytes_to_send.push(Serial::End as u8);

        assert_eq!(bytes_to_send.len(), COMMAND_LENGTH);

        bytes_to_send
    }
}

// Command to get the current configuration or set it
enum_from_primitive! {
#[repr(u8)]
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum CommandMode {
    Getting = 0,
    Setting = 1
}}

//Report modes of the sensor:
//In passive mode one has to send a request command,
//in order to get the measurement values as a response.
enum_from_primitive! {
#[repr(u8)]
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum ReportMode {
    Initiative = 0,
    Passive = 1
}}

//the Work states:
//In sleeping mode it does not send any data, the fan is turned off.
//To get data one has to wake it up'
enum_from_primitive! {
#[repr(u8)]
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum WorkState {
    Sleeping = 0,
    Measuring = 1
}}


//The unit of the measured values.
//Two modes are implemented:
//The default mode is MassConcentrationEuropean returning
//values in microgram/cubic meter (mg/m³).
//The other mode is ParticleConcentrationImperial returning values in
//particles / 0.01 cubic foot (pcs/0.01cft).
//The concentration is calculated by assuming
//different mean sphere diameters of pm10 or pm2.5 particles.
enum_from_primitive! {
#[repr(u8)]
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum UnitsOfMeasure {
    // µg / m³, the mode of the sensors firmware
    MassConcentrationEuropean = 0,
    // pcs/0.01 cft (particles / 0.01 cubic foot )
    ParticleConcentrationImperial = 1
}}

const PORT_SETTINGS: serial::PortSettings = serial::PortSettings {
    baud_rate:    serial::Baud9600,
    char_size:    serial::Bits8,
    parity:       serial::ParityNone,
    stop_bits:    serial::Stop1,
    flow_control: serial::FlowNone,
};


pub struct Sensor {
    serial_port: RefCell<serial::unix::TTYPort>,
    sensor_info: RefCell<SensorInfo>
}

impl Sensor {
    pub fn new(path: &Path) -> Result<Self, serial::Error> {
        let port = serial::open(path)?;
        let info = SensorInfo::default();
        let sensor = Sensor {
            serial_port: RefCell::new(port),
            sensor_info: RefCell::new(info)
        };
        Ok(sensor)
    }

    pub fn configure(&self, timeout: Duration) -> serial::Result<()> {
        let mut port = self.serial_port.borrow_mut();
        port.configure(&PORT_SETTINGS)?;
        port.set_timeout(timeout)?;
        Ok(())
    }

    /// reads sensor info from device
    pub fn get_sensor_info(&self) -> serial::Result<()> {
        let first_response = self.get_response(None)?;
        if first_response.len() == 0 {
            // device is sleeping?
            let measuring = SendData::set_work_state(WorkState::Measuring);
            self.send(&measuring)?;
            let duty_cycle = SendData::set_duty_cycle(0);
            self.send(&duty_cycle)?;
        }

        let get_duty_cycle = SendData::get_duty_cycle();
        let mut response = self.send(&get_duty_cycle)?;
        let duty_cycle = response[1];

        let get_report_mode = SendData::get_report_mode();
        response = self.send(&get_report_mode)?;
        let report_mode = ReportMode::from_u8(response[1]);

        let get_firmware = SendData::get_firmware();
        response = self.send(&get_firmware)?;
        let firmware = [response[0], response[1], response[2]];

        let mut info = self.sensor_info.borrow_mut();
        info.work_state = Some(WorkState::Measuring);
        info.duty_cycle = Some(duty_cycle);
        info.report_mode = report_mode;
        info.firmware = Some(firmware);

        println!("firmware: {:?}", info.firmware_string());

        Ok(())
    }

    pub fn read_bytes(&self, count: usize) -> Result<Vec<u8>, serial::Error> {
        let mut port = self.serial_port.borrow_mut();
        let mut buffer: Vec<u8> = vec![0; count];
        let result = port.read(&mut buffer[..])?;
        buffer.truncate(result);
        Ok(buffer)
    }

    pub fn write_bytes(&self, bytes: Vec<u8>) -> std::io::Result<()> {
        let mut port = self.serial_port.borrow_mut();
        port.write_all(bytes.as_slice())
    }

    pub fn send(&self, send_data: &SendData) -> Result<Vec<u8>, serial::Error> {
        let bytes_to_write = send_data.to_command_data();
        self.write_bytes(bytes_to_write)?;
        let mut response = self.get_response(Some(send_data.command))?;
        if response.len() != RESPONSE_LENGTH {
            eprintln!("received {:?} bytes expected {:?}", response.len(), RESPONSE_LENGTH);
        }

        if send_data.command != Command::Request {
            response = response[3..response.len() - 2].to_vec();
        }

        Ok(response)
    }

    pub fn get_measurement(&self) -> Result<SensorMeasurement, serial::Error> {
        let response = self.get_response(None)?;
        assert!(response.len() > 0);
        let data = response[2..6].to_vec();
        let pm2_5 = (data[0] as f32 + data[1] as f32 * 256.0) / 10.0;
        let pm10 = (data[2] as f32 + data[3] as f32 * 256.0) / 10.0;
        let timestamp = SystemTime::now();
        let measurement = SensorMeasurement { pm2_5, pm10, timestamp };
        Ok(measurement)
    }

    pub fn generate_checksum(data: &Vec<u8>) -> Option<u8> {
        let data_length = data.len();
        let expected_length = [RESPONSE_LENGTH - 2, COMMAND_LENGTH - 2];

        if !expected_length.contains(&(data_length)) {
            // invalid checksum length
            eprintln!("checksum error: invalid data length {:?}", data.len());
            return None
        }

        // check first byte
        if Serial::from_u8(data[0]) != Some(Serial::Start) {
            eprintln!("checksum error: missing start byte");
            return None
        }
        // check second byte
        let expected_second_byte = [Serial::SendByte as u8, Serial::ReceiveByte as u8, Serial::ResponseByte as u8];
        let second_byte = data[1];
        if !expected_second_byte.contains(&second_byte) {
            eprintln!("checksum error: second byte is invalid");
            return None
        }
        let third_byte = data[2];
        if second_byte != Serial::ReceiveByte as u8 && Command::from_u8(third_byte) == None {
            eprintln!("checksum error: data command byte is invalid");
            return None
        }
        let mut checksum: u8 = 0;
        for i in 2..data.len() {
            checksum = checksum.wrapping_add(data[i]);
        }
        checksum = checksum % 255;
        Some(checksum)
    }


    pub fn get_response(&self, command: Option<Command>) -> Result<Vec<u8>, serial::Error> {
        let mut bytes_received: Vec<u8> = Vec::new();

        let mut counter = 0;
        loop {
            counter = counter + 1;
            let first_read = match self.read_bytes(1) {
                Ok(read) =>  read,
                Err(err) => {
                    if err.kind() == serial::ErrorKind::Io(ErrorKind::TimedOut) {
                        continue;
                    } else {
                        panic!("read error: {:?}", err);
                    }
                }
            };
//            '''If no bytes are read the sensor might be in sleep mode.
//                It makes no sense to raise an exception here. The raise condition
//            should be checked in a context outside of this fuction.'''
            if first_read.len() > 0 {
                let first_byte = first_read[0];
                println!("byte1 #{:?} = {:X}", counter, first_byte);
                bytes_received.extend_from_slice(&[first_byte]);
//                # if this is true, serial data is coming in
                let serial_start = Serial::from_u8(first_byte);
                if serial_start == Some(Serial::Start) {
                    println!("found start byte!");
                    counter = counter + 1;
                    let next_read = match self.read_bytes(1) {
                        Ok(read) =>  read,
                        Err(err) => {
                            if err.kind() == serial::ErrorKind::Io(ErrorKind::TimedOut) {
                                continue;
                            } else {
                                panic!("read error: {:?}", err);
                            }
                        }
                    };
                    let next_byte = next_read[0];
                    println!("byte2 #{:?} = {:X}", counter, next_byte);
                    let serial_read = Serial::from_u8(next_byte);
                    println!("serial command: {:?}", serial_read);

                    if ((command != None && command != Some(Command::Request)) &&
                        serial_read == Some(Serial::ResponseByte)) ||
                        ((command == None || command == Some(Command::Request)) &&
                            serial_read == Some(Serial::ReceiveByte) )  {
                        bytes_received.push(next_byte);
                        break;
                    }
                }
            } else {
                let info = self.sensor_info.borrow();
                if info.duty_cycle == Some(0) {
                    println!("SDS011 A sensor response has not arrived within timeout limit.
                        If the sensor is in sleeping mode wake it up first!
                        Returning an empty byte array as response!");
                } else {
                    println!("SDS011 no response. Expected while in dutycycle.");
                }
                return Ok(Vec::new())
            }
        }

        let mut next_word = self.read_bytes(8)?;
        bytes_received.append(&mut next_word);

        // check if command matches response
        if command != None && command != Some(Command::Request) {
            if Serial::from_u8(bytes_received[1]) != Some(Serial::ResponseByte) {
                panic!("No responseByte  found in the response");
            }
            if Command::from_u8(bytes_received[2]) != command {
                panic!("Third byte of serial datareceived is not the expected response \
                    to the previous command");
            }
        }
        if command == None || command == Some(Command::Request) {
            if Serial::from_u8(bytes_received[1]) != Some(Serial::ReceiveByte) {
                panic!("SDS011 Received byte not found on the Value Request.");
            }
        }

        let len = bytes_received.len();

        let checksum_byte = bytes_received[len - 2];
        let checksum_data: Vec<u8> = bytes_received[0..len-2].to_vec();
        let generated_checksum = Sensor::generate_checksum(&checksum_data);
        if generated_checksum != Some(checksum_byte) {
            panic!("Invalid checksum! {:?} != {:?}", generated_checksum, checksum_byte);
        } else {
            println!("Checksum match: {:?} == {:?}", generated_checksum, checksum_byte);
        }

        // set device_id if needed
        let device_id = [bytes_received[len - 4], bytes_received[len - 3]];
        println!("device_id {:X}{:X}", device_id[0], device_id[1]);
        let mut info = self.sensor_info.borrow_mut();
        if info.device_id == None {
            info.device_id = Some(device_id);
        } else if let Some(existing_device_id) = info.device_id {
            if device_id != existing_device_id {
                panic!("SDS011 Data received  does not belong \
                            to this device with id.");
            }
        }

        Ok(bytes_received)
    }
}

#[derive(Debug, Default)]
pub struct SensorInfo {
    firmware: Option<[u8; 3]>,
    report_mode: Option<ReportMode>,
    work_state: Option<WorkState>,
    duty_cycle: Option<u8>,
    device_id: Option<[u8; 2]>
}

impl SensorInfo {
    pub fn firmware_string(&self) -> Option<String> {
        if let Some(firmware) = self.firmware {
            return Some(format!("{:02}{:02}{:02}", firmware[0], firmware[1], firmware[2]))
        }
        None
    }
}


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
