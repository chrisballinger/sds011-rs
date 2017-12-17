extern crate sds011;
extern crate clap;

use clap::{App,Arg};
use std::env;
use std::path::Path;
use std::time::Duration;
use sds011::*;

const DEVICE_PATH: &str = "DEVICE_PATH";
const DEFAULT_MAC_DEVICE_PATH: &str = "/dev/tty.wchusbserial1410";

fn main() {

    let device_env = env::var(DEVICE_PATH);

    let matches = App::new("sds011")
        .version("1.0")
        .author("Chris Ballinger <chris@ballinger.io>")
        .about("SDS011 particle sensor")
        .arg(Arg::with_name(DEVICE_PATH)
            .help("Path to device e.g. /dev/tty.wchusbserial1410")
            .index(1))
        .get_matches();

    //let mut path_string = String::new();

    let path_string: String = match device_env {
        Ok(path) =>  path,
        Err(_) => {
            match matches.value_of(DEVICE_PATH) {
                Some(path) => String::from(path),
                None => String::from(DEFAULT_MAC_DEVICE_PATH)
                //None => panic!("No device path found!")
            }
        }
    };

    println!("Attempting to open device at path: {}", path_string);

    let path = Path::new(path_string.as_str());
    let sensor = Sensor::new(path).unwrap();
    println!("Opened device at path: {}", path_string);

    sensor.configure(Duration::from_secs(1));
    println!("Configured device");


    //let output = sensor.read_bytes(10).unwrap();
    //println!("Read bytes: {:?}", output);

    let response = sensor.get_sensor_info().unwrap();
    println!("response bytes: {:?}", response);

    for i in 0..100 {
        let measurement = sensor.get_measurement().unwrap();
        println!("measurement: {:?}", measurement);
    }
}
