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

`espflash` to flash your board

```bash
cargo install espflash
```

## Installation

Set the following environment variables (modify the values to match your needs):

```bash
# For WIFI
SSID=""
PASSWORD=""

# For DNS update
DNS_HOST="dynamicdns.park-your-domain.com"
DNS_HTTP_REQUEST="GET /update?host=<HOST>&domain=<DOMAIN>&password=<PASSWORD>&ip= HTTP/1.1\r\nHost: dynamicdns.park-your-domain.com\r\nConnection: close\r\n\r\n"
DNS_UPDATE_DELAY_HOURS="12"

# UDP socket
UDP_LISTEN_PORT="12345"
WOL_BROADCAST_ADDR="255.255.255.255"
WOL_MAC_ADDR="12:34:56:78:9a:bc"
```

and run

```bash
cargo run --release
```
