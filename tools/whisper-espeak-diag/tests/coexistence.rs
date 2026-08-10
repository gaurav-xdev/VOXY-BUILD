//! Regression test: Whisper + espeak must load together without deadlock.
//!
//! Root cause: espeak-rs-sys historically linked msvcrtd (debug CRT) on Windows
//! debug builds, causing heap corruption when whisper's release-CRT whisper.cpp
//! tried to allocate. Fixed in crates/espeak-rs-sys-patch.
//!
//! IMPORTANT: espeak-ng is not thread-safe for concurrent voice setting.
//! These tests must run with `cargo test --test coexistence -- --test-threads=1`.

use std::sync::Once;
use std::time::Instant;

static ESPEAK_INIT: Once = Once::new();

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

/// Trigger espeak-ng lazy init the same way piper-rs does: via `text_to_phonemes`.
/// Uses `Once` to ensure this is safe even if called from multiple tests.
fn ensure_espeak_init() {
    ESPEAK_INIT.call_once(|| {
        let result = espeak_rs::text_to_phonemes("test", "en-US", None);
        assert!(
            result.is_ok(),
            "espeak init via text_to_phonemes failed: {result:?}"
        );
    });
}

#[test]
fn whisper_loads_alone() {
    let start = Instant::now();
    let model_path = find_whisper_model();

    let ctx = whisper_rs::WhisperContext::new_with_params(
        &model_path,
        whisper_rs::WhisperContextParameters::default(),
    )
    .expect("whisper context creation failed");

    let _state = ctx.create_state().expect("whisper state creation failed");

    println!(
        "whisper_loads_alone: OK in {}ms",
        start.elapsed().as_millis()
    );
}

#[test]
fn espeak_then_whisper_sequential() {
    let start = Instant::now();

    ensure_espeak_init();

    let model_path = find_whisper_model();
    let ctx = whisper_rs::WhisperContext::new_with_params(
        &model_path,
        whisper_rs::WhisperContextParameters::default(),
    )
    .expect("whisper context creation failed after espeak init");

    let _state = ctx
        .create_state()
        .expect("whisper state creation failed after espeak init");

    println!(
        "espeak_then_whisper_sequential: OK in {}ms",
        start.elapsed().as_millis()
    );
}

#[test]
fn whisper_then_espeak_sequential() {
    let start = Instant::now();

    let model_path = find_whisper_model();
    let ctx = whisper_rs::WhisperContext::new_with_params(
        &model_path,
        whisper_rs::WhisperContextParameters::default(),
    )
    .expect("whisper context creation failed");

    let _state = ctx.create_state().expect("whisper state creation failed");

    ensure_espeak_init();

    println!(
        "whisper_then_espeak_sequential: OK in {}ms",
        start.elapsed().as_millis()
    );
}

#[test]
fn both_engines_load_together() {
    let start = Instant::now();

    let model_path = find_whisper_model();
    let ctx = whisper_rs::WhisperContext::new_with_params(
        &model_path,
        whisper_rs::WhisperContextParameters::default(),
    )
    .expect("whisper context creation failed");

    let _state = ctx.create_state().expect("whisper state creation failed");

    ensure_espeak_init();

    println!(
        "both_engines_load_together: OK in {}ms",
        start.elapsed().as_millis()
    );
}

#[tokio::test]
async fn both_engines_on_async_runtime() {
    let start = Instant::now();

    let model_path = find_whisper_model();
    let ctx = tokio::task::spawn_blocking(move || {
        whisper_rs::WhisperContext::new_with_params(
            &model_path,
            whisper_rs::WhisperContextParameters::default(),
        )
        .expect("whisper context creation failed on async runtime")
    })
    .await
    .expect("spawn_blocking failed");

    let _state = ctx.create_state().expect("whisper state creation failed");

    ensure_espeak_init();

    println!(
        "both_engines_on_async_runtime: OK in {}ms",
        start.elapsed().as_millis()
    );
}
