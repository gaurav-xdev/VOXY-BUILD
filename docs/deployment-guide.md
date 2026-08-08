# VOXY Deployment Guide

## System Requirements

### Minimum
- OS: Windows 10/11, Linux (Ubuntu 20.04+), macOS 11+
- RAM: 4GB
- CPU: 4 cores, AVX2 support
- Storage: 2GB free
- Audio: Microphone and speakers

### Recommended
- OS: Windows 11
- RAM: 8GB+
- CPU: 8 cores, AVX2 + SSE2 + FMA
- GPU: NVIDIA with CUDA (optional, for Whisper acceleration)
- Storage: 4GB+ SSD
- Audio: Low-latency audio interface

## Building for Production

```bash
# Build optimized binary
cargo build --release --workspace

# Binary location
./target/release/voxy.exe
```

## Configuration

### Environment Variables
```powershell
# Required for build
$env:LIBCLANG_PATH="C:\tools\llvm\bin"
$env:CMAKE="C:\tools\cmake2\cmake-3.31.6-windows-x86_64\bin\cmake.exe"

# Optional: Enable GPU acceleration
$env:VOXY_GPU_ENABLED="true"
```

### Configuration File
Create `config.toml`:
```toml
[voice]
wake_word_enabled = true
stt_provider = "whisper"
tts_provider = "kokoro"

[llm]
provider = "openai"
model = "gpt-4"

[memory]
max_capacity = 10000
forgetting_rate = 0.1

[security]
consent_required = true
audit_logging = true
```

## Deployment Steps

### 1. Build
```powershell
$env:LIBCLANG_PATH="C:\tools\llvm\bin"; cargo build --release --workspace
```

### 2. Verify
```powershell
cargo test --workspace --lib
```

### 3. Configure
Copy `config.toml` to application directory.

### 4. Run
```powershell
./target/release/voxy.exe
```

## Docker Deployment (Experimental)

```dockerfile
FROM rust:1.75 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bullseye-slim
RUN apt-get update && apt-get install -y libclang-dev cmake
COPY --from=builder /app/target/release/voxy /usr/local/bin/
CMD ["voxy"]
```

## Monitoring

### Health Check
```bash
# Check if system is responsive
curl http://localhost:8080/health
```

### Logs
```bash
# View logs
tail -f logs/voxy.log
```

### Metrics
- Event bus throughput: events/sec
- Memory usage: MB
- CPU usage: %
- Active tasks: count
- Error rate: errors/min

## Scaling

### Vertical Scaling
- Increase RAM for larger memory capacity
- Use faster CPU for lower latency
- Add GPU for faster Whisper inference

### Horizontal Scaling
- Run multiple instances with different voice channels
- Share memory through SQLite database
- Coordinate via event bus

## Backup and Recovery

### Backup Memory
```bash
cp voxy_memory.db voxy_memory.db.bak
```

### Restore Memory
```bash
cp voxy_memory.db.bak voxy_memory.db
```

### Reset System
```bash
rm voxy_memory.db
rm -rf config/
```

## Troubleshooting

### High Latency
1. Check CPU usage
2. Reduce memory capacity
3. Use faster LLM provider
4. Enable GPU acceleration

### Memory Leaks
1. Monitor memory usage over time
2. Check for unbounded collections
3. Review event bus buffer sizes
4. Verify forgetting algorithm is running

### Audio Issues
1. Check microphone permissions
2. Verify WASAPI device availability
3. Test with different audio devices
4. Check for exclusive mode conflicts
