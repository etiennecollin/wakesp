# wakesp

<!-- vim-markdown-toc GFM -->

- [Features](#features)
- [Requirements](#requirements)
- [Installation](#installation)
- [UDP Requests](#udp-requests)
  - [`wol`](#wol)

<!-- vim-markdown-toc -->

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

Set the following environment variables. These variables are used to configure the ESP32-C3 at compile time. Modify the values to match your needs:

**WiFi Configuration**

- `HOSTNAME`: The hostname for your ESP32-C3 device on the network.
- `SSID`: The SSID (name) of the WiFi network your ESP32-C3 will connect to.
- `PASSWORD`: The password for the WiFi network.

**DNS Update Configuration**

- `DNS_ENABLE`: A flag to enable or disable DNS updates. Set to "true" or "1" to enable.
- `DNS_HOST`: The hostname of the DDNS provider's update service.
- `DNS_HTTP_REQUEST`: The HTTP request format for updating the DDNS. Customize with your host, domain, and password details.
- `DNS_UPDATE_DELAY_HOURS`: The interval, in hours, at which the DDNS should be updated.

**UDP Socket Configuration**

- `UDP_ENABLE`: A flag to enable or disable the UDP socket interface. Set to "true" or "1" to enable.
- `UDP_LISTEN_PORT`: The port on which the ESP32-C3 will listen for UDP packets.
- `WOL_BROADCAST_ADDR`: The broadcast address to send Wake-on-LAN packets to. Typically set to "255.255.255.255" to broadcast to all devices on the local network.
- `WOL_MAC_ADDR`: The MAC address of the device you want to wake up using Wake-on-LAN.

Here is an example of setting these variaples:

```bash
# For WIFI
export HOSTNAME="myesp32"
export SSID="MyWiFiNetwork"
export PASSWORD="mywifipassword"

# For DNS update
export DNS_ENABLE="true"
export DNS_HOST="dynamicdns.park-your-domain.com"
export DNS_HTTP_REQUEST="GET /update?host=<HOST>&domain=<DOMAIN>&password=<PASSWORD>&ip= HTTP/1.1\r\nHost: dynamicdns.park-your-domain.com\r\nConnection: close\r\n\r\n"
export DNS_UPDATE_DELAY_HOURS="12"

# UDP socket
export UDP_ENABLE="true"
export UDP_LISTEN_PORT="12345"
export WOL_BROADCAST_ADDR="255.255.255.255"
export WOL_MAC_ADDR="12:34:56:78:9a:bc"
```

In the same shell session (so that the variables are set), you can then flash and run your ESP32-C3 with:

```bash
cargo run --release
```

Follow the on-screen instructions. Note the IP address given to the ESP32-C3 by the router. It should be printed in the opened terminal as a log. You will be able to send UDP requests to it.

Once the board is flashed, you may close your terminal and unplug your ESP32-C3. Power it via USB anywhere that is reached by the chosen WIFI network. The board will start working immediately once powered by USB.

## UDP Requests

You may use `netcat` to send UDP requests:

```bash
echo "<REQUEST>" | nc -uw1 <ESP32-C3_IP_ADDRESS> <UDP_LISTEN_PORT>
```

> It can be useful to create an alias or a function in your shell config for these requests!

The `"<REQUEST>"` has the following format: `"<COMMAND>,<ARG>"`.
Here are the available commands:

### `wol`

- This command will send a Wake-on-LAN packet to the MAC address specified by `WOL_MAC_ADDR`.
  - Example: `echo "wol" | nc -uw1 <ESP32-C3_IP_ADDRESS> <UDP_LISTEN_PORT>"`
- It is possible to override the MAC address using the argument of the command.
  - Example: `echo "wol,00:b0:d0:63:c2:26" | nc -uw1 <ESP32-C3_IP_ADDRESS> <UDP_LISTEN_PORT>"`
