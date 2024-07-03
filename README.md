# wakesp

<!-- vim-markdown-toc GFM -->

- [Features](#features)
- [Requirements](#requirements)
- [Installation](#installation)
  - [Automatically Setting Environment Variables](#automatically-setting-environment-variables)
- [UDP Requests](#udp-requests)
  - [`ping`](#ping)
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

This repository

```bash
git clone https://github.com/etiennecollin/wakesp
```

## Installation

Set the following environment variables. These variables are used to configure the ESP32-C3 at compile time. Modify the values to match your needs:

**WiFi Configuration**

- `HOSTNAME`: The hostname for your ESP32-C3 device on the network.
- `SSID`: The SSID (name) of the WiFi network your ESP32-C3 will connect to.
- `PASSWORD`: The password for the WiFi network.

**DNS Update Configuration**

- `DNS_ENABLE`: A flag to enable or disable DNS updates. Set to "true" or "1" to enable.
- `DNS_CHECK_DELAY`: The interval, in seconds, at which the DDNS should be updated.
- `DNS_HOST`: The hostname of the DDNS provider's update service.
- `DNS_HTTP_REQUEST`: The HTTP request format for updating the DDNS. Customize with your host, domain, and password details.

**UDP Socket Configuration**

- `UDP_ENABLE`: A flag to enable or disable the UDP socket interface. Set to "true" or "1" to enable.
- `UDP_LISTEN_PORT`: The port on which the ESP32-C3 will listen for UDP packets.
- `WOL_BROADCAST_ADDR`: The broadcast address to send Wake-on-LAN packets to. Typically set to "255.255.255.255" to broadcast to all devices on the local network.
- `WOL_MAC_ADDR`: The MAC address of the device you want to wake up using Wake-on-LAN.

Here is an example of setting these variables:

```bash
# For WIFI
export HOSTNAME="myesp32"
export SSID="MyWiFiNetwork"
export PASSWORD="mywifipassword"

# For DNS update
export DNS_ENABLE="true"
export DNS_CHECK_DELAY="60"
export DNS_HOST="dynamicdns.park-your-domain.com"
export DNS_HTTP_REQUEST="GET /update?host=<HOST>&domain=<DOMAIN>&password=<PASSWORD>&ip= HTTP/1.1\r\nHost: dynamicdns.park-your-domain.com\r\nConnection: close\r\n\r\n"

# UDP socket
export UDP_ENABLE="true"
export UDP_LISTEN_PORT="12345"
export WOL_BROADCAST_ADDR="255.255.255.255"
export WOL_MAC_ADDR="12:34:56:78:9a:bc"
```

Now, make sure that your current working directory (output of `pwd` command) is the root of the cloned repository.
You can then flash your ESP32-C3 with:

```bash
cargo run --release
```

> Run this last command in the same shell session you set the environment variables in.

Follow the on-screen instructions. Note the IP address given to the ESP32-C3 by the router. It should be printed in the opened terminal as a log. You will be able to send UDP requests to it.

Once the board is flashed, you may close your terminal and unplug your ESP32-C3. Power it via USB anywhere that is reached by the chosen WIFI network. The board will start working immediately once powered by USB.

### Automatically Setting Environment Variables

If you plan on flashing your board more than once, you could edit the file `./.cargo/config.toml` such that it sets the variables automatically. To do so, modify the `[env]` section of the file as follows to add your environment variables:

```toml
# ...
[env]
# ...

# For WIFI
HOSTNAME="myesp32"
SSID="MyWiFiNetwork"
PASSWORD="mywifipassword"

# For DNS update
DNS_ENABLE="true"
DNS_CHECK_DELAY="60"
DNS_HOST="dynamicdns.park-your-domain.com"
DNS_HTTP_REQUEST="GET /update?host=<HOST>&domain=<DOMAIN>&password=<PASSWORD>&ip= HTTP/1.1\r\nHost: dynamicdns.park-your-domain.com\r\nConnection: close\r\n\r\n"

# UDP socket
UDP_ENABLE="true"
UDP_LISTEN_PORT="12345"
WOL_BROADCAST_ADDR="255.255.255.255"
WOL_MAC_ADDR="12:34:56:78:9a:bc"

# ...
```

## UDP Requests

You may use `netcat` to send UDP requests:

```bash
echo "<REQUEST>" | nc -uw1 <ESP32-C3_IP_ADDRESS> <UDP_LISTEN_PORT>
```

> It can be useful to create an alias or a function in your shell config for these requests!

The `"<REQUEST>"` has the following format: `"<COMMAND>,<ARG>"`.
Here are the available commands:

### `ping`

- This command will simply ping the ESP32-C3 and return a status message. It can be used to test that the ESP32-C3 is working.
  - Example: `echo "ping" | nc -uw1 <ESP32-C3_IP_ADDRESS> <UDP_LISTEN_PORT>"`

### `wol`

- This command will send a Wake-on-LAN packet to the MAC address specified by `WOL_MAC_ADDR`.
  - Example: `echo "wol" | nc -uw1 <ESP32-C3_IP_ADDRESS> <UDP_LISTEN_PORT>"`
- It is possible to override the MAC address using the argument of the command.
  - Example: `echo "wol,00:b0:d0:63:c2:26" | nc -uw1 <ESP32-C3_IP_ADDRESS> <UDP_LISTEN_PORT>"`
