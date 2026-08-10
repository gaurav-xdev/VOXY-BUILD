use std::time::Instant;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let test = args.get(1).map(|s| s.as_str()).unwrap_or("all");

    println!("=== Whisper + espeak Coexistence Diagnostic ===");
    println!("Test: {test}");
    println!();

    match test {
        "whisper" => test_whisper_only(),
        "sequential" => test_sequential(),
        "all" => {
            test_whisper_only();
            test_sequential();
        }
        _ => {
            eprintln!("Usage: whisper-espeak-diag [whisper|sequential|all]");
            std::process::exit(1);
        }
    }

    println!();
    println!("=== All tests passed ===");
}

fn test_whisper_only() {
    println!("--- Test 1: whisper-only (baseline) ---");
    let start = Instant::now();

    let model_path = find_whisper_model();
    println!("  Model: {model_path}");

    let t0 = Instant::now();
    let ctx = whisper_rs::WhisperContext::new_with_params(
        &model_path,
        whisper_rs::WhisperContextParameters::default(),
    )
    .expect("whisper context creation failed");
    println!("  WhisperContext created ({})", t0.elapsed().as_millis());

    let t1 = Instant::now();
    let _state = ctx.create_state().expect("whisper state creation failed");
    println!("  WhisperState created ({})", t1.elapsed().as_millis());

    println!("  PASSED ({})", start.elapsed().as_millis());
}

fn test_sequential() {
    println!("--- Test 2: espeak init FIRST, then whisper load ---");
    let start = Instant::now();

    // Step 1: Init espeak first
    let t0 = Instant::now();
    unsafe {
        let path = std::ffi::CString::new(".").unwrap();
        let result = espeak_rs_sys::espeak_Initialize(
            espeak_rs_sys::espeak_AUDIO_OUTPUT_AUDIO_OUTPUT_RETRIEVAL,
            0,
            path.as_ptr(),
            0,
        );
        assert!(result >= 0, "espeak_Initialize failed: {result}");
    }
    println!("  espeak_Initialize OK ({})", t0.elapsed().as_millis());

    // Step 2: Load whisper model
    let model_path = find_whisper_model();
    let t1 = Instant::now();
    println!("  Loading whisper model...");
    let ctx = whisper_rs::WhisperContext::new_with_params(
        &model_path,
        whisper_rs::WhisperContextParameters::default(),
    )
    .expect("whisper context creation failed");
    let _state = ctx.create_state().expect("whisper state creation failed");
    println!("  Whisper loaded ({})", t1.elapsed().as_millis());

    unsafe {
        let _ = espeak_rs_sys::espeak_Terminate();
    }
    println!("  PASSED ({})", start.elapsed().as_millis());
}

fn find_whisper_model() -> String {
    let candidates = [
        "models/ggml-base.en.bin",
        "../models/ggml-base.en.bin",
        "../../models/ggml-base.en.bin",
        "../../../models/ggml-base.en.bin",
    ];
    for path in &candidates {
        if std::path::Path::new(path).exists() {
            return path.to_string();
        }
    }
    panic!("Could not find ggml-base.en.bin model file in any of: {candidates:?}");
}
