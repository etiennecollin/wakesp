# wakesp

Wakesp (from the words `wake` and `esp`) is a firmware project for the ESP32-C3 microcontroller, written in Rust. This project allows the ESP32-C3 to update a dynamic DNS (DDNS) through manual TCP HTTP requests and wake up devices on your network using Wake-on-LAN through a custom UDP request interface.

## Features

- Dynamic DNS Updates: Manually update your DDNS provider with the latest IP address.
- Wake-on-LAN: Send WOL packets easily with a custom UDP interface to wake up devices on your network.
- Completely async without an OS thanks to [embassy](https://github.com/embassy-rs/embassy)
- Rust Implementation: Benefit from the safety and performance of Rust.
- ESP32-C3 Compatible: Designed specifically for the ESP32-C3 microcontroller.

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
DNS_ENABLE="true"
DNS_HOST="dynamicdns.park-your-domain.com"
DNS_HTTP_REQUEST="GET /update?host=<HOST>&domain=<DOMAIN>&password=<PASSWORD>&ip= HTTP/1.1\r\nHost: dynamicdns.park-your-domain.com\r\nConnection: close\r\n\r\n"
DNS_UPDATE_DELAY_HOURS="12"

# UDP socket
UDP_ENABLE="true"
UDP_LISTEN_PORT="12345"
WOL_BROADCAST_ADDR="255.255.255.255"
WOL_MAC_ADDR="12:34:56:78:9a:bc"
```

and run

```bash
cargo run --release
```
