use embassy_net::{
    udp::{PacketMetadata, UdpSocket},
    IpAddress, IpEndpoint, Stack,
};
use embassy_time::{Duration, Timer};
use esp_backtrace as _;
use esp_wifi::wifi::{WifiDevice, WifiStaDevice};

const UDP_LISTEN_PORT: &str = env!("UDP_LISTEN_PORT");
const WOL_BROADCAST_ADDR: &str = env!("WOL_BROADCAST_ADDR");
const WOL_MAC_ADDR: &str = env!("WOL_MAC_ADDR");
const WOL_FALLBACK_BROADCAST_ADDR: IpAddress = IpAddress::v4(255, 255, 255, 255);

#[embassy_executor::task]
pub async fn udp_task(stack: &'static Stack<WifiDevice<'static, WifiStaDevice>>) {
    loop {
        loop {
            if stack.is_link_up() {
                break;
            }
            Timer::after(Duration::from_millis(500)).await;
        }

        log::info!("UDP | Waiting to get IP address...");
        loop {
            if let Some(config) = stack.config_v4() {
                log::info!("UDP | Got IP: {}", config.address);
                break;
            }
            Timer::after(Duration::from_millis(500)).await;
        }

        // Setup UDP socket
        let mut rx_meta = [PacketMetadata::EMPTY; 16];
        let mut rx_buffer = [0; 4096];
        let mut tx_meta = [PacketMetadata::EMPTY; 16];
        let mut tx_buffer = [0; 4096];
        let mut buf = [0; 4096];

        let mut socket = UdpSocket::new(
            stack,
            &mut rx_meta,
            &mut rx_buffer,
            &mut tx_meta,
            &mut tx_buffer,
        );

        if let Err(e) = socket.bind(UDP_LISTEN_PORT.parse::<u16>().unwrap_or(9)) {
            log::error!("UDP | Error binding to port: {:?}", e);
            socket.close();
            continue;
        }

        let broadcast_addr = match parse_ip_address(WOL_BROADCAST_ADDR) {
            Ok(v) => v,
            Err(e) => {
                log::error!(
                    "UDP | Invalid broadcast address -> {}: {}",
                    e,
                    WOL_BROADCAST_ADDR
                );

                log::error!(
                    "UDP | Using fallback broadcast address: {}",
                    WOL_FALLBACK_BROADCAST_ADDR
                );
                WOL_FALLBACK_BROADCAST_ADDR
            }
        };
        let wol_target = IpEndpoint::new(broadcast_addr, 9);

        loop {
            Timer::after(Duration::from_millis(1_000)).await;

            log::info!("UDP | Ready to receive requests");
            let (data_len, remote_end_point) = match socket.recv_from(&mut buf).await {
                Ok(n) => n,
                Err(e) => {
                    log::error!("UDP | Reception error: {:?}", e);
                    socket.close();
                    break;
                }
            };

            let message = match core::str::from_utf8(&buf[..data_len]) {
                Ok(v) => v,
                Err(_) => {
                    log::warn!(
                        "UDP | Received invalid request from {} with bytearray length {}",
                        remote_end_point,
                        data_len
                    );
                    write_udp_response(&mut socket, b"Invalid UDP request", &remote_end_point)
                        .await;
                    Timer::after(Duration::from_millis(1_000)).await;
                    socket.close();
                    break;
                }
            };

            let (command, arg) = parse_udp_request(message).await;

            match command {
                "wol" => {
                    log::info!("UDP | Received WOL request from: {}", remote_end_point);
                    write_udp_response(&mut socket, b"WOL request received", &remote_end_point)
                        .await;
                    if (send_wol_packet(&mut socket, &wol_target, &arg).await).is_err() {
                        write_udp_response(
                            &mut socket,
                            b"Failed to submit packet on target network",
                            &remote_end_point,
                        )
                        .await;
                        Timer::after(Duration::from_millis(1_000)).await;
                        socket.close();
                        break;
                    }
                    write_udp_response(&mut socket, b"WOL request submitted", &remote_end_point)
                        .await;
                }
                _ => {
                    log::warn!(
                        "UDP | Received invalid command in request from {}",
                        remote_end_point,
                    );
                    log::warn!(
                        "UDP | Invalid request (command, arg): (\"{}\", \"{:?}\")",
                        command,
                        arg
                    );

                    write_udp_response(&mut socket, b"UDP request denied", &remote_end_point).await;
                }
            }
        }
        socket.close();
    }
}

async fn send_wol_packet(
    socket: &mut UdpSocket<'_>,
    wol_target: &IpEndpoint,
    args: &Option<&str>,
) -> Result<(), ()> {
    let mac_addr = match args {
        Some(v) => v,
        None => {
            log::warn!(
                "UDP | No argument provided for WOL through request. Using default mac address"
            );
            WOL_MAC_ADDR
        }
    };

    let wol_packet = match create_wol_packet(mac_addr).await {
        Ok(v) => v,
        Err(e) => {
            log::warn!("UDP | Error creating WOL packet -> {}: \"{}\"", e, mac_addr);
            return Err(());
        }
    };

    // Try to send the WOL packet 3 times max
    let mut i = 0;
    const MAX_TRIES: u8 = 3;
    while i < MAX_TRIES {
        let status = socket.send_to(&wol_packet, *wol_target).await;
        match status {
            Ok(()) => {
                log::info!("UDP | Sent WOL packet to MAC address: {}", mac_addr);
                break;
            }
            Err(e) => {
                i += 1;
                log::warn!("UDP | Error sending WOL packet: {:?}", e);
                log::warn!("UDP | Trying again ({} try remaining)", MAX_TRIES - i);
                Timer::after(Duration::from_millis(500)).await;
            }
        }
    }

    if i == MAX_TRIES {
        log::error!("UDP | Failed to send WOL packet");
        return Err(());
    }

    Ok(())
}

/// Write a response to a client over UDP.
/// The message is expected to be a string formatted as an array of bytes.
/// The message has a maximum length of 127 bytes.
async fn write_udp_response(socket: &mut UdpSocket<'_>, message: &[u8], endpoint: &IpEndpoint) {
    let mut message_augmented = [0u8; 128];
    message_augmented[..message.len()].copy_from_slice(message);
    message_augmented[message.len()] = b'\n';
    message_augmented[message.len() + 1] = b'\0';

    let status = socket.send_to(&message_augmented, *endpoint).await;
    match status {
        Ok(()) => {
            let message_str = match core::str::from_utf8(message) {
                Ok(v) => v,
                Err(e) => {
                    log::error!("UDP | Error parsing response message as str: {:?}", e);
                    return;
                }
            };

            log::info!("UDP | Sent response to client: {}", message_str);
        }
        Err(e) => log::error!("UDP | Error sending response: {:?}", e),
    }
}

async fn parse_udp_request(request: &str) -> (&str, Option<&str>) {
    let mut parts = request.split(',');

    // Set the command as the first part and the args as the second part
    // If no command is provided, return ""
    // If no args are provided, return None
    let command = parts.next().unwrap_or_default().trim();

    let arg;
    if let Some(v) = parts.next() {
        if v.is_empty() || v == "\n" {
            arg = None;
        } else {
            arg = Some(v.trim());
        }
    } else {
        arg = None;
    }

    (command, arg)
}

async fn create_wol_packet(mac_addr: &str) -> Result<[u8; 102], &str> {
    // Parse the MAC address
    let mut mac_bytes = [0u8; 6];
    let mut parts = mac_addr.split(':');
    let status = (0..6).try_for_each(|i| {
        let part = match parts.next() {
            Some(v) => v,
            None => return Err("Invalid MAC address size"),
        };
        match u8::from_str_radix(part, 16) {
            Ok(v) => mac_bytes[i] = v,
            Err(_) => return Err("Could not parse MAC address, bad format"),
        }

        Ok(())
    });

    // Return an error if the MAC address parsing failed
    status?;

    let mut wol_packet = [0u8; 102];

    // Fill the first 6 bytes with 0xFF
    (0..6).for_each(|i| {
        wol_packet[i] = 0xFF;
    });

    // Repeat the MAC address 16 times
    (0..16).for_each(|i| {
        let start = 6 + i * 6;
        wol_packet[start..start + 6].copy_from_slice(&mac_bytes);
    });

    Ok(wol_packet)
}

fn parse_ip_address(ip_str: &str) -> Result<IpAddress, &str> {
    // Take a string of the form "192.168.00.11" and return an IpAddress
    let mut ip_buf = [0u8; 4];
    let mut parts = ip_str.split('.');

    let status = (0..4).try_for_each(|i| {
        let part = match parts.next() {
            Some(v) => v,
            None => return Err("Invalid IP address size"),
        };

        match part.parse::<u8>() {
            Ok(v) => ip_buf[i] = v,
            Err(_) => return Err("Could not parse IP address, bad format"),
        }

        Ok(())
    });

    match status {
        Ok(_) => Ok(IpAddress::v4(ip_buf[0], ip_buf[1], ip_buf[2], ip_buf[3])),
        Err(e) => Err(e),
    }
}
