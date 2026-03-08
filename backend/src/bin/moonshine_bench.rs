//! Moonshine inference benchmark.
//!
//! Loads a Moonshine model, generates test audio, runs multiple inference passes,
//! and reports latency statistics.
//!
//! Usage:
//!   cargo run --release --bin moonshine_bench [-- --model-dir <path> --variant <tiny|base> --passes <N>]

use std::path::PathBuf;
use std::time::Instant;
use transcribe_rs::engines::moonshine::{
    MoonshineEngine, MoonshineModelParams, ModelVariant,
};
use transcribe_rs::TranscriptionEngine;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let mut model_dir = dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("whispertype")
        .join("models")
        .join("moonshine-tiny");
    let mut variant = ModelVariant::Tiny;
    let mut passes: usize = 10;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--model-dir" => {
                i += 1;
                model_dir = PathBuf::from(&args[i]);
            }
            "--variant" => {
                i += 1;
                variant = match args[i].as_str() {
                    "base" => ModelVariant::Base,
                    _ => ModelVariant::Tiny,
                };
            }
            "--passes" => {
                i += 1;
                passes = args[i].parse().expect("--passes must be a number");
            }
            _ => {
                eprintln!("Unknown arg: {}", args[i]);
                std::process::exit(1);
            }
        }
        i += 1;
    }

    println!("=== Moonshine Benchmark ===");
    println!("Model dir: {}", model_dir.display());
    println!("Variant:   {:?}", variant);
    println!("Passes:    {}", passes);
    println!();

    // Load model
    println!("Loading model...");
    let load_start = Instant::now();
    let mut engine = MoonshineEngine::new();
    engine
        .load_model_with_params(&model_dir, MoonshineModelParams::variant(variant))
        .expect("Failed to load model");
    let load_ms = load_start.elapsed().as_millis();
    println!("Model loaded in {}ms", load_ms);
    println!();

    // Generate test audio: 3 seconds, mix of 440Hz sine + silence
    let sample_rate = 16000;
    let duration_sec = 3.0f32;
    let num_samples = (sample_rate as f32 * duration_sec) as usize;
    let samples: Vec<f32> = (0..num_samples)
        .map(|i| {
            let t = i as f32 / sample_rate as f32;
            if t < 1.0 {
                // 1s of 440Hz sine
                (2.0 * std::f32::consts::PI * 440.0 * t).sin() * 0.3
            } else if t < 2.0 {
                // 1s of silence
                0.0
            } else {
                // 1s of 880Hz sine
                (2.0 * std::f32::consts::PI * 880.0 * t).sin() * 0.3
            }
        })
        .collect();

    println!(
        "Test audio: {:.1}s ({} samples @ {}Hz)",
        duration_sec, num_samples, sample_rate
    );
    println!();

    // Run benchmark passes
    let mut latencies_ms = Vec::with_capacity(passes);

    for pass in 0..passes {
        let start = Instant::now();
        let result = engine
            .transcribe_samples(samples.clone(), None)
            .expect("Inference failed");
        let elapsed_ms = start.elapsed().as_millis() as f64;
        latencies_ms.push(elapsed_ms);

        if pass == 0 {
            println!("First pass output: {:?}", result.text.trim());
            println!();
        }
    }

    // Report stats
    let min = latencies_ms.iter().cloned().fold(f64::INFINITY, f64::min);
    let max = latencies_ms
        .iter()
        .cloned()
        .fold(f64::NEG_INFINITY, f64::max);
    let sum: f64 = latencies_ms.iter().sum();
    let mean = sum / passes as f64;
    let rtf = mean / (duration_sec as f64 * 1000.0);

    let mut sorted = latencies_ms.clone();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let p50 = sorted[passes / 2];
    let p95 = sorted[(passes as f64 * 0.95) as usize];

    println!("=== Results ({} passes) ===", passes);
    println!("  Min:    {:.1}ms", min);
    println!("  Max:    {:.1}ms", max);
    println!("  Mean:   {:.1}ms", mean);
    println!("  P50:    {:.1}ms", p50);
    println!("  P95:    {:.1}ms", p95);
    println!("  RTF:    {:.3}x (< 1.0 = faster than real-time)", rtf);
    println!(
        "  Speed:  {:.1}x real-time",
        duration_sec as f64 * 1000.0 / mean
    );
}
