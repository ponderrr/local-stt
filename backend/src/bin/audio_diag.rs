#![allow(deprecated)]
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

fn main() {
    println!("=== CPAL Audio Diagnostics ===\n");

    println!("Available hosts:");
    for host_id in cpal::available_hosts() {
        println!("  - {:?}", host_id);
    }
    println!();

    for host_id in cpal::available_hosts() {
        println!("--- Testing host: {:?} ---", host_id);

        let host = match cpal::host_from_id(host_id) {
            Ok(h) => h,
            Err(e) => {
                println!("  FAILED to create host: {}", e);
                continue;
            }
        };

        let devices = match host.input_devices() {
            Ok(d) => d.collect::<Vec<_>>(),
            Err(e) => {
                println!("  FAILED to enumerate input devices: {}", e);
                continue;
            }
        };

        println!("  Input devices found: {}", devices.len());

        for (i, device) in devices.iter().enumerate() {
            let name = device.name().unwrap_or_else(|_| "UNKNOWN".into());
            println!("  [{}] {}", i, name);

            match device.default_input_config() {
                Ok(config) => {
                    println!("      Default config: {:?}", config);
                    println!("      Sample rate: {}", config.sample_rate());
                    println!("      Channels: {}", config.channels());
                    println!("      Sample format: {:?}", config.sample_format());
                }
                Err(e) => {
                    println!("      FAILED to get default config: {}", e);
                    continue;
                }
            }

            match device.supported_input_configs() {
                Ok(configs) => {
                    println!("      Supported configs:");
                    for (j, cfg) in configs.enumerate() {
                        println!(
                            "        [{}] channels={}, rate={}..{}, format={:?}",
                            j,
                            cfg.channels(),
                            cfg.min_sample_rate(),
                            cfg.max_sample_rate(),
                            cfg.sample_format()
                        );
                    }
                }
                Err(e) => println!("      FAILED to get supported configs: {}", e),
            }

            println!("      Attempting 3-second capture test...");
            let sample_count = Arc::new(AtomicU64::new(0));
            let count_clone = sample_count.clone();
            let error_flag = Arc::new(AtomicU64::new(0));
            let error_clone = error_flag.clone();

            let config = match device.default_input_config() {
                Ok(c) => c,
                Err(_) => continue,
            };

            let stream = match device.build_input_stream(
                &config.clone().into(),
                move |data: &[f32], _: &cpal::InputCallbackInfo| {
                    count_clone.fetch_add(data.len() as u64, Ordering::Relaxed);
                },
                move |err| {
                    eprintln!("      STREAM ERROR: {}", err);
                    error_clone.fetch_add(1, Ordering::Relaxed);
                },
                None,
            ) {
                Ok(s) => s,
                Err(e) => {
                    println!("      FAILED to build stream: {}", e);
                    continue;
                }
            };

            match stream.play() {
                Ok(()) => println!("      stream.play() succeeded"),
                Err(e) => {
                    println!("      stream.play() FAILED: {}", e);
                    continue;
                }
            }

            std::thread::sleep(Duration::from_secs(3));

            let total = sample_count.load(Ordering::Relaxed);
            let errors = error_flag.load(Ordering::Relaxed);
            drop(stream);

            println!(
                "      Results: {} samples captured, {} errors",
                total, errors
            );

            if total == 0 {
                println!("      *** ZERO SAMPLES — stream is NOT delivering audio ***");
            } else {
                let expected = config.sample_rate() as u64 * config.channels() as u64 * 3;
                let pct = (total as f64 / expected as f64 * 100.0) as u64;
                println!("      *** WORKING — got ~{}% of expected samples ***", pct);
            }
        }
        println!();
    }

    println!("=== Diagnosis Complete ===");
}
