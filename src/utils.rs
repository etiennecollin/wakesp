use embassy_net::{IpAddress, Stack};
use embassy_time::{Duration, Timer};
use esp_wifi::wifi::{WifiDevice, WifiStaDevice};

/// Parse an IP address from a string
pub fn parse_ip_address(ip_str: &str) -> Result<IpAddress, &str> {
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

/// Wait for the wifi device to connect to the network and until it gets an IP address
pub async fn wait_for_connection(stack: &'static Stack<WifiDevice<'static, WifiStaDevice>>) {
    while !stack.is_link_up() {
        Timer::after(Duration::from_millis(500)).await;
    }

    if stack.config_v4() == None {
        log::info!("SYS | Waiting to get IP address...");
        while stack.config_v4() == None {
            Timer::after(Duration::from_millis(500)).await;
        }
    }
    log::info!("SYS | Device IP: {}", stack.config_v4().unwrap().address);
}
