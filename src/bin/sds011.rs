extern crate sds011;
extern crate clap;
extern crate chrono;
extern crate csv;

use clap::{App,Arg};
use std::env;
use std::path::{Path, PathBuf};
use std::time::Duration;
use chrono::prelude::*;
use sds011::sensor::Sensor;

const OUTPUT_DIR: &str = "OUTPUT_DIR";
const DEVICE_PATH: &str = "DEVICE_PATH";
const NUM_READINGS: &str = "NUM_READINGS";
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
        .arg(Arg::with_name(OUTPUT_DIR)
            .short("o")
            .long("output-dir")
            .help("Sets a custom output directory")
            .takes_value(true))
        .arg(Arg::with_name(NUM_READINGS)
            .short("n")
            .long("num-readings")
            .help("Number of sensor readings before exit. Zero means continue forever. Default = 0")
            .takes_value(true))
        .get_matches();

    let path_string: String = match device_env {
        Ok(path) =>  path,
        Err(_) => {
            match matches.value_of(DEVICE_PATH) {
                Some(path) => String::from(path),
                None => String::from(DEFAULT_MAC_DEVICE_PATH)
            }
        }
    };

    let out_dir_ref = env::current_dir().unwrap();
    let out_dir: PathBuf = match matches.value_of(OUTPUT_DIR) {
        Some(path) =>  PathBuf::from(path),
        None => out_dir_ref
    };
    let num_readings_str = matches.value_of(NUM_READINGS).unwrap_or("0");
    let num_readings: i32 = num_readings_str.parse().unwrap_or(0);

    println!("Output file directory: {:?}", out_dir);

    println!("Attempting to open device at path: {}", path_string);

    let path = Path::new(path_string.as_str());
    let sensor = Sensor::new(path).unwrap();
    println!("Opened device at path: {}", path_string);

    sensor.configure(Duration::from_secs(1));
    println!("Configured device");

    let response = sensor.get_sensor_info().unwrap();
    println!("Got sensor info");

    let start_time = Utc::now();
    let start_time_string = start_time.format("%Y-%m-%d %H_%M_%S").to_string();
    let mut output_file_path = out_dir.clone();
    output_file_path.push(start_time_string);
    output_file_path.set_extension("csv");

    println!("Recording measurements to file: {:?}", output_file_path);
    let mut csv_writer = csv::Writer::from_path(output_file_path.as_path()).unwrap();

    let mut i = 0;
    loop {
        if num_readings != 0 && i >= num_readings {
            break
        }
        let measurement = sensor.get_measurement().unwrap();
        println!("#{:?}/{:?}: {:?}", i+1, num_readings, measurement);
        csv_writer.serialize(measurement);
        csv_writer.flush();
        i += 1;
    }
    println!("All done. See ya later!");
}
