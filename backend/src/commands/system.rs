use crate::audio::capture::AudioCapture;

#[tauri::command]
pub fn list_audio_devices() -> Result<Vec<String>, String> {
    AudioCapture::list_devices()
}

#[tauri::command]
pub fn get_gpu_info() -> Result<serde_json::Value, String> {
    // Basic GPU detection — parse nvidia-smi output
    let output = std::process::Command::new("nvidia-smi")
        .args([
            "--query-gpu=name,memory.total",
            "--format=csv,noheader,nounits",
        ])
        .output()
        .map_err(|_| "nvidia-smi not found — CUDA may not be available".to_string())?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let parts: Vec<&str> = stdout.trim().split(", ").collect();

    Ok(serde_json::json!({
        "name": parts.first().unwrap_or(&"Unknown"),
        "vram_total_mb": parts.get(1).and_then(|s| s.parse::<u32>().ok()).unwrap_or(0),
        "cuda_available": output.status.success(),
    }))
}
