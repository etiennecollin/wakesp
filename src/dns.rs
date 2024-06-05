use embassy_net::{dns::DnsQueryType, tcp::TcpSocket, IpAddress, IpEndpoint, Stack};
use embassy_time::{Duration, Timer};
use esp_backtrace as _;
use esp_println::println;
use esp_wifi::wifi::{WifiDevice, WifiStaDevice};

const DNS_HOST: &str = env!("DNS_HOST");
const DNS_HTTP_REQUEST: &[u8] = env!("DNS_HTTP_REQUEST").as_bytes();
const DNS_UPDATE_DELAY_HOURS: &str = env!("DNS_UPDATE_DELAY_HOURS");
const DNS_UPDATE_FALLBACK_DELAY_SECONDS: u64 = 43_200; // 12 hours

#[embassy_executor::task]
pub async fn dns_updater_task(stack: &'static Stack<WifiDevice<'static, WifiStaDevice>>) {
    loop {
        loop {
            if stack.is_link_up() {
                break;
            }
            Timer::after(Duration::from_millis(500)).await;
        }

        log::info!("DNS | Waiting to get IP address...");
        loop {
            if let Some(config) = stack.config_v4() {
                log::info!("DNS | Got IP: {}", config.address);
                break;
            }
            Timer::after(Duration::from_millis(500)).await;
        }

        loop {
            Timer::after(Duration::from_millis(1_000)).await;

            // Resolve the IP of the remote endpoint
            let ip_list = match stack.dns_query(DNS_HOST, DnsQueryType::A).await {
                Ok(v) => v,
                Err(e) => {
                    log::error!("DNS | DNS query error: {:?}", e);
                    continue;
                }
            };

            // Get the first IPv4 address in the list
            let remote_endpoint;
            if let Some(ipv4_addr) = ip_list.iter().find(|x| matches!(x, IpAddress::Ipv4(_))) {
                remote_endpoint = IpEndpoint::new(*ipv4_addr, 80);
                log::info!("DNS | Found IP for {}: {:?}", DNS_HOST, ipv4_addr);
            } else {
                log::error!("DNS | No IP found for {}", DNS_HOST);
                continue;
            }

            // Setup TCP socket
            let mut rx_buffer = [0; 4096];
            let mut tx_buffer = [0; 4096];
            let mut socket_tcp = TcpSocket::new(stack, &mut rx_buffer, &mut tx_buffer);
            socket_tcp.set_timeout(Some(embassy_time::Duration::from_secs(10)));

            // Connect to the remote endpoint
            log::info!("DNS | Connecting to {}...", DNS_HOST);
            if let Err(e) = socket_tcp.connect(remote_endpoint).await {
                log::error!("DNS | Connection error: {:?}", e);
                socket_tcp.close();
                continue;
            }
            log::info!("DNS | Connected!");

            // Send the HTTP request to update the IP address
            let mut buf = [0; 1024];
            log::info!("DNS | Writing HTTP request...");
            if let Err(e) = write_tcp_buf(&mut socket_tcp, DNS_HTTP_REQUEST).await {
                log::error!("DNS | Error writing HTTP request: {:?}", e);
                socket_tcp.close();
                continue;
            }

            // Get response length
            let response_len = match socket_tcp.read(&mut buf).await {
                Ok(0) => {
                    log::info!("DNS | Response EOF");
                    socket_tcp.close();
                    continue;
                }
                Ok(n) => n,
                Err(e) => {
                    log::warn!(
                        "DNS | Response error, make sure request went through: {:?}",
                        e
                    );
                    socket_tcp.close();
                    continue;
                }
            };

            // Print the response
            let response = core::str::from_utf8(&buf[..response_len]);
            match response {
                Ok(v) => println!("{}", v),
                Err(e) => log::warn!("DNS | Response was not UTF8: {:?}", e),
            }

            socket_tcp.close();
            break;
        }

        // Wait for the next DNS update
        let delay_seconds = match DNS_UPDATE_DELAY_HOURS.parse::<u64>() {
            Ok(v) => v * 3_600, // Convert hours to seconds
            Err(e) => {
                log::error!(
                    "DNS | Error parsing DNS_UPDATE_DELAY_HOURS to u64 -> {}: {}",
                    e,
                    DNS_UPDATE_DELAY_HOURS
                );

                log::error!(
                    "DNS | Using fallback DNS update delay: {} seconds",
                    DNS_UPDATE_FALLBACK_DELAY_SECONDS
                );
                DNS_UPDATE_FALLBACK_DELAY_SECONDS
            }
        };

        Timer::after(Duration::from_secs(delay_seconds)).await;
    }
}

async fn write_tcp_buf(
    socket: &mut TcpSocket<'_>,
    buf: &[u8],
) -> Result<(), embassy_net::tcp::Error> {
    let mut buf = buf;
    while !buf.is_empty() {
        match socket.write(buf).await {
            Ok(0) => log::warn!("TCP buffer writer wrote 0 bytes to the buffer"),
            Ok(n) => buf = &buf[n..],
            Err(e) => return Err(e),
        }
    }
    Ok(())
}
