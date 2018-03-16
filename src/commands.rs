use sensor::*;

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

pub const RESPONSE_LENGTH: usize = 10;
pub const COMMAND_LENGTH: usize = 19;

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
    pub command: Command,
    mode: CommandMode,
    data: Vec<u8>,
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
        SendData {
            command,
            mode,
            data,
        }
    }

    pub fn to_command_data(&self) -> Vec<u8> {
        let command = self.command as u8;
        let mode = self.mode as u8;
        let mut bytes_to_send: Vec<u8> =
            vec![Serial::Start as u8, Serial::SendByte as u8, command, mode];

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
