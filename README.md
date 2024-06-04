# wol-esp32

WOL using ESP32-C3

## Requirements

Rust nightly toolchain

```bash
rustup toolchain install nightly --component rust-src
```

Target for ESP32-C3

```bash
rustup target add riscv32imc-unknown-none-elf
```

Also install espflash to flash your board

```bash
cargo install espflash
```

## Installation

Set the following environment variables:

```bash
SSID=""
PASSWORD=""
```

and run

```bash
cargo run --release
```
