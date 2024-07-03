use crate::utils::{parse_ip_address, wait_for_connection};

use embassy_net::{dns::DnsQueryType, tcp::TcpSocket, IpAddress, IpEndpoint, Stack};
use embassy_time::{Duration, Timer};
use esp_backtrace as _;
use esp_println::println;
use esp_wifi::wifi::{WifiDevice, WifiStaDevice};
use heapless::{String, Vec};

const DNS_CHECK_DELAY: &str = env!("DNS_CHECK_DELAY");
const DNS_CHECK_FALLBACK_DELAY: u64 = 60;
const DNS_HOST: &str = env!("DNS_HOST");
const DNS_HTTP_REQUEST: &[u8] = env!("DNS_HTTP_REQUEST").as_bytes();

const PUBLIC_IP_PROVIDER_HOST: &str = "api.ipify.org";
const PUBLIC_IP_PROVIDER_REQUEST: &[u8] =
    "GET / HTTP/1.1\r\nHost: api.ipify.org\r\nConnection: close\r\n\r\n".as_bytes();

const TCP_BUFFER_SIZE: usize = 4096;

#[embassy_executor::task]
pub async fn dns_updater_task(stack: &'static Stack<WifiDevice<'static, WifiStaDevice>>) {
    // Parse the DNS check delay and fallback to the fallback delay if there is an error
    let delay_seconds = match DNS_CHECK_DELAY.parse::<u64>() {
        Ok(v) => v,
        Err(e) => {
            log::error!(
                "DNS | Error parsing DNS_CHECK_DELAY to u64 -> {}: {}",
                e,
                DNS_CHECK_DELAY
            );

            log::error!(
                "DNS | Using fallback DNS check delay: {} seconds",
                DNS_CHECK_FALLBACK_DELAY
            );
            DNS_CHECK_FALLBACK_DELAY
        }
    };

    // Keep track of the previous public IP address
    let mut prev_public_ip = None;
    loop {
        wait_for_connection(stack).await;

        // Get the public IP address
        let public_ip_response =
            match send_http_request(stack, PUBLIC_IP_PROVIDER_HOST, PUBLIC_IP_PROVIDER_REQUEST)
                .await
            {
                Ok(Some(v)) => {
                    log::info!("DNS | Got response from {}:", PUBLIC_IP_PROVIDER_HOST);
                    println!("{}", v);
                    v
                }
                Ok(None) => {
                    log::error!("DNS | Got empty response from public IP provider");
                    continue;
                }
                Err(_) => continue,
            };

        // Remove the HTTP headers
        let public_ip_str = match public_ip_response.split("\r\n\r\n").last() {
            Some(v) => v,
            None => {
                log::error!("DNS | Public IP address not found in response");
                continue;
            }
        };

        // Parse the public IP address
        let public_ip = match parse_ip_address(public_ip_str) {
            Ok(v) => {
                log::info!("DNS | Public IP address: {}", v);
                v
            }
            Err(e) => {
                log::error!("DNS | Public IP address not found in response -> {}", e);
                continue;
            }
        };

        // Check if the public IP address has changed
        // We only update the DNS if the IP address has changed
        if Some(public_ip) == prev_public_ip {
            log::info!(
                "DNS | Public IP address has not changed. Next check in {} seconds",
                delay_seconds
            );
            Timer::after(Duration::from_secs(delay_seconds)).await;
            continue;
        }

        // Update the DNS
        match send_http_request(stack, DNS_HOST, DNS_HTTP_REQUEST).await {
            Ok(Some(v)) => {
                log::info!("DNS | Got response from {}:", DNS_HOST);
                println!("{}", v);
                log::info!("DNS | DNS updated. Next check in {} seconds", delay_seconds);
                prev_public_ip = Some(public_ip);
                Timer::after(Duration::from_secs(delay_seconds)).await;
            }
            Ok(None) => {
                continue;
            }
            Err(_) => continue,
        };
    }
}

/// Sends an HTTP request to the target host and returns its response.
async fn send_http_request(
    stack: &'static Stack<WifiDevice<'static, WifiStaDevice>>,
    target_host: &'static str,
    request: &'static [u8],
) -> Result<Option<String<TCP_BUFFER_SIZE>>, ()> {
    // Get public IP address
    let remote_endpoint = get_dns_address(stack, target_host).await?;

    // Setup TCP socket
    let mut rx_buffer = [0; TCP_BUFFER_SIZE];
    let mut tx_buffer = [0; TCP_BUFFER_SIZE];
    let mut socket_tcp = TcpSocket::new(stack, &mut rx_buffer, &mut tx_buffer);
    socket_tcp.set_timeout(Some(embassy_time::Duration::from_secs(10)));

    // Connect to the remote endpoint
    log::info!("DNS | Connecting to {}...", target_host);
    if let Err(e) = socket_tcp.connect(remote_endpoint).await {
        socket_tcp.close();
        log::error!("DNS | Error connecting to {}: {:?}", target_host, e);
        return Err(());
    }
    log::info!("DNS | Connected to {}!", target_host);

    // Send the HTTP request to update the IP address
    log::info!("DNS | Writing HTTP request to {}...", target_host);
    if let Err(e) = write_tcp_buf(&mut socket_tcp, request).await {
        socket_tcp.close();
        log::error!("DNS | Error writing request to {}: {:?}", target_host, e);
        return Err(());
    }

    // Get response length
    let mut response_buf = [0; TCP_BUFFER_SIZE];
    let response_len = match socket_tcp.read(&mut response_buf).await {
        Ok(n) => n,
        Err(e) => {
            socket_tcp.close();
            log::error!("DNS | Error reading response from {}: {:?}", target_host, e);
            return Err(());
        }
    };

    // Parse the response as UTF8
    let response = match String::from_utf8(Vec::from_slice(&response_buf[..response_len]).unwrap())
    {
        Ok(v) => Ok(Some(v)),
        Err(e) => {
            log::warn!("DNS | Response was not UTF8: {:?}", e);
            Ok(None)
        }
    };

    socket_tcp.close();
    return response;
}

/// Queries the DNS server for the IP address of the target host.
async fn get_dns_address(
    stack: &'static Stack<WifiDevice<'static, WifiStaDevice>>,
    target_host: &'static str,
) -> Result<IpEndpoint, ()> {
    // Resolve the IP of the remote endpoint
    log::info!("DNS | Resolving IP for {}...", target_host);
    let ip_list = match stack.dns_query(target_host, DnsQueryType::A).await {
        Ok(v) => v,
        Err(e) => {
            log::error!("DNS | Error querying DNS server: {:?}", e);
            return Err(());
        }
    };

    // Get the first IPv4 address in the list
    let remote_endpoint;
    if let Some(ipv4_addr) = ip_list.iter().find(|x| matches!(x, IpAddress::Ipv4(_))) {
        remote_endpoint = IpEndpoint::new(*ipv4_addr, 80);
        log::info!("DNS | Found IP for {}: {}", target_host, ipv4_addr);
    } else {
        log::error!("DNS | No IP found for {}", target_host);
        return Err(());
    }

    return Ok(remote_endpoint);
}

/// Writes a buffer to a TCP socket.
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
