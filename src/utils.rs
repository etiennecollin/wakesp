use core::str::FromStr;
use embassy_futures::select::{Either, select};
use embassy_net::{IpAddress, Stack, tcp::TcpSocket};
use embassy_time::{Duration, Timer};
use heapless::String;

/// 12 bytes for the chars and 5 bytes for the colons
pub const MAC_LEN: usize = 17;

/// Converts the string representation of a MAC address to the correct format.
/// (e.g. "00%3A00%3A00%3A00%3A00%3A00" -> "00:00:00:00:00:00")
pub fn convert_mac_address(addr: &str) -> Result<String<MAC_LEN>, ()> {
    // Check if the address is already in the correct format
    if !addr.contains("%3A") {
        return String::from_str(addr);
    }

    // Allocate a buffer to store the parsed address
    let mut addr_parsed: String<MAC_LEN> = String::new();
    let mut parts = addr.split("%3A");

    // Iterate over the parts of the address and push them to the
    // buffer, adding a colon between each part
    let status = parts.try_for_each(|part| {
        if addr_parsed.push_str(part).is_err() {
            log::error!(
                "Could not parse MAC address. Tried pushing {} in {}",
                part,
                addr_parsed
            );
            return Err(());
        };

        if addr_parsed.len() < addr_parsed.capacity() && addr_parsed.push(':').is_err() {
            log::error!(
                "Could not parse MAC address. Tried pushing \":\" in {}",
                addr_parsed
            );
            return Err(());
        }

        Ok(())
    });

    match status {
        Ok(_) => Ok(addr_parsed),
        Err(_) => Err(()),
    }
}

/// Parse an IP address from a string
pub fn parse_ip_address(ip_str: &str) -> Result<IpAddress, &str> {
    // Take a string of the form "000.000.000.000" and return an IpAddress
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

/// Wait for the wifi device to connect to the network and until it gets an IP address
pub async fn wait_for_connection(stack: Stack<'_>) {
    while !stack.is_link_up() {
        Timer::after(Duration::from_millis(500)).await;
    }

    if stack.config_v4().is_none() {
        log::info!("SYS | Waiting to get IP address...");
        while stack.config_v4().is_none() {
            Timer::after(Duration::from_millis(500)).await;
        }
    }
    log::info!("SYS | Device IP: {}", stack.config_v4().unwrap().address);
}

/// Writes a buffer to a TCP socket.
pub async fn write_tcp_buf(socket: &mut TcpSocket<'_>, mut buf: &[u8]) -> Result<(), ()> {
    while !buf.is_empty() {
        match socket.write(buf).await {
            Ok(0) => log::warn!("SYS | TCP buffer writer wrote 0 bytes to the buffer"),
            Ok(n) => buf = &buf[n..],
            Err(_) => return Err(()),
        }
    }

    flush_wrapper(socket, 500).await?;

    Ok(())
}

/// Abort the connection and flush the socket.
pub async fn abort_connection(socket: &mut TcpSocket<'_>) {
    socket.abort();
    let _ = flush_wrapper(socket, 500).await;
}

/// Flush the socket and return an error if it takes too long
/// `max_time` is in milliseconds
async fn flush_wrapper(socket: &mut TcpSocket<'_>, max_time: u64) -> Result<(), ()> {
    match select(
        socket.flush(),
        Timer::after(Duration::from_millis(max_time)),
    )
    .await
    {
        Either::First(v) => {
            if v.is_err() {
                log::error!("SYS | Error flushing TCP socket: {:?}", v);
                return Err(());
            }
        }
        Either::Second(_) => {
            log::error!("SYS | TCP socket took too long to flush");
            return Err(());
        }
    }
    Ok(())
}
