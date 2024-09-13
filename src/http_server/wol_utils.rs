use crate::pins::*;
use crate::utils::{convert_mac_address, parse_ip_address};
use embassy_net::{
    udp::{PacketMetadata, UdpSocket},
    IpAddress, IpEndpoint, Stack,
};
use embassy_time::{Duration, Timer};
use esp_wifi::wifi::{WifiDevice, WifiStaDevice};

/// The port on which the device will listen for UDP requests.
const UDP_BIND_PORT: u16 = 9;
/// The buffer size for the UDP socket.
/// It should be big enough to contain the WOL packets.
const UDP_BUFFER_SIZE: usize = 128;
/// The broadcast address to send the WOL packet to.
const WOL_BROADCAST_ADDR: &str = env!("WOL_BROADCAST_ADDR");
/// The fallback broadcast address to send the WOL packet to.
const WOL_BROADCAST_ADDR_FALLBACK: IpAddress = IpAddress::v4(255, 255, 255, 255);

/// Triggers a GPIO pin set as an open drain
pub async fn switch_command(pin_str: &str) -> Result<(), ()> {
    // Parse the pin number as a u8
    let pin = match pin_str.parse::<u8>() {
        Ok(v) => v,
        Err(_) => {
            log::error!("Switch | Error parsing pin number");
            return Err(());
        }
    };

    let mut triggered = false;
    match pin {
        2 => {
            GPIO2.lock(|x| {
                if let Some(v) = x.borrow_mut().as_mut() {
                    v.toggle();
                    triggered = true;
                } else {
                    triggered = false;
                }
            });
            Timer::after(Duration::from_millis(500)).await;
            GPIO2.lock(|x| {
                if let Some(v) = x.borrow_mut().as_mut() {
                    v.toggle();
                    triggered = true;
                } else {
                    triggered = false;
                }
            });
        }
        3 => {
            GPIO3.lock(|x| {
                if let Some(v) = x.borrow_mut().as_mut() {
                    v.toggle();
                    triggered = true;
                } else {
                    triggered = false;
                }
            });
            Timer::after(Duration::from_millis(500)).await;
            GPIO3.lock(|x| {
                if let Some(v) = x.borrow_mut().as_mut() {
                    v.toggle();
                    triggered = true;
                } else {
                    triggered = false;
                }
            });
        }
        4 => {
            GPIO4.lock(|x| {
                if let Some(v) = x.borrow_mut().as_mut() {
                    v.toggle();
                    triggered = true;
                } else {
                    triggered = false;
                }
            });
            Timer::after(Duration::from_millis(500)).await;
            GPIO4.lock(|x| {
                if let Some(v) = x.borrow_mut().as_mut() {
                    v.toggle();
                    triggered = true;
                } else {
                    triggered = false;
                }
            });
        }
        5 => {
            GPIO5.lock(|x| {
                if let Some(v) = x.borrow_mut().as_mut() {
                    v.toggle();
                    triggered = true;
                } else {
                    triggered = false;
                }
            });
            Timer::after(Duration::from_millis(500)).await;
            GPIO5.lock(|x| {
                if let Some(v) = x.borrow_mut().as_mut() {
                    v.toggle();
                    triggered = true;
                } else {
                    triggered = false;
                }
            });
        }
        6 => {
            GPIO6.lock(|x| {
                if let Some(v) = x.borrow_mut().as_mut() {
                    v.toggle();
                    triggered = true;
                } else {
                    triggered = false;
                }
            });
            Timer::after(Duration::from_millis(500)).await;
            GPIO6.lock(|x| {
                if let Some(v) = x.borrow_mut().as_mut() {
                    v.toggle();
                    triggered = true;
                } else {
                    triggered = false;
                }
            });
        }
        7 => {
            GPIO7.lock(|x| {
                if let Some(v) = x.borrow_mut().as_mut() {
                    v.toggle();
                    triggered = true;
                } else {
                    triggered = false;
                }
            });
            Timer::after(Duration::from_millis(500)).await;
            GPIO7.lock(|x| {
                if let Some(v) = x.borrow_mut().as_mut() {
                    v.toggle();
                    triggered = true;
                } else {
                    triggered = false;
                }
            });
        }
        8 => {
            GPIO8.lock(|x| {
                if let Some(v) = x.borrow_mut().as_mut() {
                    v.toggle();
                    triggered = true;
                } else {
                    triggered = false;
                }
            });
            Timer::after(Duration::from_millis(500)).await;
            GPIO8.lock(|x| {
                if let Some(v) = x.borrow_mut().as_mut() {
                    v.toggle();
                    triggered = true;
                } else {
                    triggered = false;
                }
            });
        }
        9 => {
            GPIO9.lock(|x| {
                if let Some(v) = x.borrow_mut().as_mut() {
                    v.toggle();
                    triggered = true;
                } else {
                    triggered = false;
                }
            });
            Timer::after(Duration::from_millis(500)).await;
            GPIO9.lock(|x| {
                if let Some(v) = x.borrow_mut().as_mut() {
                    v.toggle();
                    triggered = true;
                } else {
                    triggered = false;
                }
            });
        }
        _ => {
            log::error!("Switch | Invalid pin number");
            return Err(());
        }
    }

    if !triggered {
        log::error!("Switch | Error toggling pin GPIO{}", pin_str);
        return Err(());
    }

    log::info!("SWITCH | Triggered pin GPIO{}", pin_str);
    Ok(())
}

/// Send a Wake-on-LAN command to the specified MAC address.
pub async fn wol_command(
    stack: &'static Stack<WifiDevice<'static, WifiStaDevice>>,
    mac_addr: &str,
) -> Result<(), ()> {
    // Replace "%3A" with ":" in the MAC address
    let mac_addr = match convert_mac_address(mac_addr) {
        Ok(v) => v,
        Err(_) => {
            log::error!("WOL | Error parsing MAC address");
            return Err(());
        }
    };
    let wol_packet = match generate_wol_packet(mac_addr.as_str()) {
        Ok(v) => v,
        Err(e) => {
            log::warn!("WOL | Error creating WOL packet -> {}: \"{}\"", e, mac_addr);
            return Err(());
        }
    };
    let wol_target = get_broadcast_addr(WOL_BROADCAST_ADDR);

    // Setup UDP socket
    let mut rx_meta = [PacketMetadata::EMPTY; 16];
    let mut rx_buffer = [0; UDP_BUFFER_SIZE];
    let mut tx_meta = [PacketMetadata::EMPTY; 16];
    let mut tx_buffer = [0; UDP_BUFFER_SIZE];

    let mut socket = UdpSocket::new(
        stack,
        &mut rx_meta,
        &mut rx_buffer,
        &mut tx_meta,
        &mut tx_buffer,
    );

    if let Err(e) = socket.bind(UDP_BIND_PORT) {
        log::error!("WOL | Error binding UDP socket to port: {:?}", e);
        socket.close();
        return Err(());
    }

    // Try to send the WOL packet 3 times max
    let mut i = 0;
    const MAX_TRIES: u8 = 3;
    while i < MAX_TRIES {
        let status = socket.send_to(&wol_packet, wol_target).await;
        match status {
            Ok(()) => {
                log::info!("WOL | Sent WOL packet to MAC address: {}", mac_addr);
                Timer::after(Duration::from_millis(500)).await; // Wait for packet to be sent
                break;
            }
            Err(e) => {
                i += 1;
                log::warn!("WOL | Error sending WOL packet: {:?}", e);
                log::warn!("WOL | Trying again ({} try remaining)", MAX_TRIES - i);
                Timer::after(Duration::from_millis(500)).await;
            }
        }
    }

    if i == MAX_TRIES {
        log::error!("WOL | Failed to send WOL packet");
        socket.close();
        return Err(());
    }

    socket.close();
    Ok(())
}

/// Create a Wake-on-LAN packet from a MAC address.
/// The packet is a 102-byte array with the first 6 bytes set to 0xFF and the MAC address repeated 16 times.
fn generate_wol_packet(mac_addr: &str) -> Result<[u8; 102], &str> {
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

/// Get the broadcast address from a string and fallback to a constant address if parsing fails.
fn get_broadcast_addr(addr: &str) -> IpEndpoint {
    // Parse the constant broadcast address
    let broadcast_addr = match parse_ip_address(addr) {
        Ok(v) => v,
        Err(e) => {
            log::error!("WOL | Invalid broadcast address -> {}: {}", e, addr);

            log::error!(
                "WOL | Using fallback broadcast address: {}",
                WOL_BROADCAST_ADDR_FALLBACK
            );
            WOL_BROADCAST_ADDR_FALLBACK
        }
    };
    IpEndpoint::new(broadcast_addr, 9)
}
