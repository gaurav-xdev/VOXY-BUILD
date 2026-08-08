# PHANTOM Desktop Runtime — Benchmark Report

## Environment

- **OS**: Windows 11 (x86_64)
- **CPU**: Intel i5-13420H (8 cores / 12 threads)
- **Mode**: Debug (unoptimized) — release mode is 10-50x faster
- **Tool**: `cargo test --lib benchmarks -- --nocapture`

---

## Benchmark Results (Debug Mode)

| Benchmark | Iterations | Total Time | Per-Operation | Throughput |
|-----------|-----------|------------|---------------|------------|
| Runtime init | 1,000 | 77ms | **77μs/init** | 13,000/s |
| Tray creation | 1,000 | 636μs | **636ns/iter** | 1,572,000/s |
| Notification send | 1,000 | 3.2ms | **3.2μs/iter** | 312,500/s |
| Shortcut register | 1,000 | 1.4ms | **1.4μs/iter** | 714,000/s |
| Settings read | 1,000 | 4.3ms | **4.3μs/iter** | 232,500/s |
| Settings validate | 1,000 | 3.7ms | **3.7μs/iter** | 270,000/s |
| EventBus publish | 1,000 | 19ms | **19μs/iter** | 52,600/s |
| Push-to-talk toggle | 1,000 | 26ms | **26μs/iter** | 38,500/s |
| DownloadManager create | 1,000 | 301ms | **301μs/iter** | 3,322/s |
| Clipboard read | 100 | 10ms | **102μs/iter** | 9,800/s |

---

## Performance Analysis

### Fast (< 10μs)
- **Tray creation**: 636ns — negligible overhead
- **Shortcut register**: 1.4μs — vector push + string alloc
- **Notification send**: 3.2μs — atomic counter + RwLock write
- **Settings validate**: 3.7μs — field comparison only
- **Settings read**: 4.3μs — RwLock read + clone

### Moderate (10-100μs)
- **EventBus publish**: 19μs — broadcast channel + serialization
- **Push-to-talk toggle**: 26μs — RwLock + EventBus publish
- **Runtime init**: 77μs — full struct initialization

### I/O Bound (> 100μs)
- **Clipboard read**: 102μs — Win32 OpenClipboard/GetClipboardData
- **DownloadManager create**: 301μs — directory creation + path resolution

---

## Release Mode Estimates

Based on typical debug→release ratios for these operation types:

| Benchmark | Debug | Estimated Release | Speedup |
|-----------|-------|-------------------|---------|
| Runtime init | 77μs | ~5μs | 15x |
| Tray creation | 636ns | ~100ns | 6x |
| Notification send | 3.2μs | ~200ns | 16x |
| Shortcut register | 1.4μs | ~100ns | 14x |
| Settings read | 4.3μs | ~300ns | 14x |
| Settings validate | 3.7μs | ~200ns | 18x |
| EventBus publish | 19μs | ~2μs | 10x |
| Push-to-talk toggle | 26μs | ~3μs | 9x |

---

## Latency Budget (Voice Pipeline)

For push-to-talk to feel instant, the shortcut → voice pipeline path must be < 50ms:

| Step | Estimated Latency |
|------|-------------------|
| Win32 RegisterHotKey → WM_HOTKEY | ~1ms |
| Shortcut handler → EventBus publish | ~26μs |
| EventBus → voice pipeline subscriber | ~2μs |
| Voice pipeline start recording | ~5ms |
| **Total** | **~6ms** |

**Status**: Well within budget. Push-to-talk will feel instant.

---

## Recommendations

1. **Run benchmarks in release mode** for accurate production numbers
2. **Clipboard read** is I/O bound — consider caching for repeated reads
3. **DownloadManager creation** creates directories — acceptable for one-time init
4. **EventBus publish** could benefit from pre-allocated channels for hot paths
5. All operations are well under 1ms — no performance blockers for Desktop UI
