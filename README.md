# wakesp

<!-- vim-markdown-toc GFM -->

- [Features](#features)
- [Requirements](#requirements)
- [Installation](#installation)
  - [Automatically Setting Environment Variables](#automatically-setting-environment-variables)
- [HTTP Server](#http-server)

<!-- vim-markdown-toc -->

Wakesp (from the words `wake` and `esp`) is a firmware project for the ESP32 microcontroller, written in Rust. This project allows the ESP32 to update a dynamic DNS (DDNS) through manual TCP HTTP requests and wake up devices on your network using Wake-on-LAN through a web interface.

## Features

- Dynamic DNS Updates: Manually update your DDNS provider with the latest IP address.
- Wake-on-LAN: Send WOL packets easily with a web interface to wake up devices on your network.
- Completely async without an OS thanks to [embassy](https://github.com/embassy-rs/embassy)
- Rust Implementation: Benefit from the safety and performance of Rust.
- ESP32 Compatible: Designed specifically for the ESP32 microcontroller.

## Requirements

Rust nightly toolchain

```bash
rustup toolchain install nightly --component rust-src
```

Target for ESP32

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

Set the following environment variables. These variables are used to configure the ESP32 at compile time. Modify the values to match your needs:

**WiFi Configuration**

- `HOSTNAME`: The hostname for your ESP32 device on the network.
- `SSID`: The SSID (name) of the WiFi network your ESP32 will connect to.
- `PASSWORD`: The password for the WiFi network.

**DNS Update Configuration**

- `DNS_ENABLE`: A flag to enable or disable DNS updates. Set to "true" or "1" to enable.
- `DNS_CHECK_DELAY`: The interval in seconds between the DNS update checks.
- `DNS_HOST`: The hostname of the update service of your DNS provider.
- `DNS_HTTP_REQUEST`: The HTTP request format for updating the DNS. Customize with your host, domain, and password details.

**HTTP Server Configuration**

- `HTTP_SERVER_ENABLE`: A flag to enable or disable the HTTP server. Set to "true" or "1" to enable.
- `HTTP_LISTEN_PORT`: The port on which the ESP32 will listen for HTTP requests.

**WOL Configuration**

- `WOL_ENABLE`: A flag to enable or disable the WOL feature of the HTTP server. Set to "true" or "1" to enable.
- `WOL_BROADCAST_ADDR`: The broadcast address to send Wake-on-LAN packets to. Typically set to "255.255.255.255" to broadcast to all devices on the local network.

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

# For HTTP server
export HTTP_SERVER_ENABLE="true"
export HTTP_LISTEN_PORT="80"

# For WOL
export WOL_ENABLE="true"
export WOL_BROADCAST_ADDR="255.255.255.255"
```

Now, make sure that your current working directory (output of `pwd` command) is the root of the cloned repository.
You can then flash your ESP32 with:

```bash
cargo run --release
```

> Run this last command in the same shell session you set the environment variables in.

Follow the on-screen instructions. Note the IP address given to the ESP32 by the router. It should be printed in the opened terminal as a log. You will be able to send UDP requests to it.

Once the board is flashed, you may close your terminal and unplug your ESP32. Power it via USB anywhere that is reached by the chosen WIFI network. The board will start working immediately once powered by USB.

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

# For HTTP server
HTTP_SERVER_ENABLE="true"
HTTP_LISTEN_PORT="80"

# For WOL
WOL_ENABLE="true"
WOL_BROADCAST_ADDR="255.255.255.255"

# ...
```

## HTTP Server

Connect to your device by typing `http://<IP_OF_YOUR_ESP32>:<HTTP_LISTEN_PORT>` in your favourite browser. For example:

- `http://192.168.2.10:80`
