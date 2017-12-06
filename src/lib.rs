extern crate serial;

use std::path::Path;

// Constants
#[repr(u8)]
pub enum Serial {
    Start = 0xAA,
    End = 0xAB,
    SendByte = 0xB4,
    ResponseByte = 0xC5,
    ReceiveByte = 0xC0,
    CommandTerminator = 0xFF
}

const RESPONSE_LENGTH: u32 = 10;
const COMMAND_LENGTH: u32 = 19;

// Enumeration of SDS011 commands
#[repr(u8)]
pub enum Command {
    ReportMode = 2,
    Request = 4,
    DeviceId = 5,
    WorkState = 6,
    Firmware = 7,
    DutyCycle = 8
}

// Command to get the current configuration or set it
#[repr(u8)]
pub enum CommandMode {
    Getting = 0,
    Setting = 1
}

//Report modes of the sensor:
//In passive mode one has to send a request command,
//in order to get the measurement values as a response.
#[repr(u8)]
pub enum ReportModes {
    Sleeping = 0,
    Measuring = 1
}

//The unit of the measured values.
//Two modes are implemented:
//The default mode is MassConcentrationEuropean returning
//values in microgram/cubic meter (mg/m³).
//The other mode is ParticleConcentrationImperial returning values in
//particles / 0.01 cubic foot (pcs/0.01cft).
//The concentration is calculated by assuming
//different mean sphere diameters of pm10 or pm2.5 particles.
#[repr(u8)]
pub enum UnitsOfMeasure {
    // µg / m³, the mode of the sensors firmware
    MassConcentrationEuropean = 0,
    // pcs/0.01 cft (particles / 0.01 cubic foot )
    ParticleConcentrationImperial = 1
}

const PORT_SETTINGS: serial::PortSettings = serial::PortSettings {
    baud_rate:    serial::Baud9600,
    char_size:    serial::Bits8,
    parity:       serial::ParityNone,
    stop_bits:    serial::Stop1,
    flow_control: serial::FlowNone,
};


pub struct Sensor {
    serial_port: serial::unix::TTYPort
}

impl Sensor {
    pub fn new(path: &Path) -> Result<Self, serial::Error> {
        let port = serial::open(path)?;
        let sensor = Sensor {
            serial_port: port
        };
        Ok(sensor)
    }
}


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
