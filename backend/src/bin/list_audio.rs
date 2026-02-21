#![allow(deprecated)]
use cpal::traits::{DeviceTrait, HostTrait};

fn main() {
    let host = cpal::default_host();
    match host.input_devices() {
        Ok(devices) => {
            println!("Available Input Devices:");
            for device in devices {
                if let Ok(name) = device.name() {
                    println!(" - {}", name);
                } else {
                    println!(" - (unknown name)");
                }
            }
        }
        Err(e) => {
            println!("Failed to get input devices: {}", e);
        }
    }

    if let Some(device) = host.default_input_device() {
        if let Ok(name) = device.name() {
            println!("\nDefault Input Device:");
            println!(" - {}", name);
        }
    } else {
        println!("\nNo default input device found.");
    }
}
