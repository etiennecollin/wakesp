use crate::utils::{abort_connection, parse_ip_address, wait_for_connection, write_tcp_buf};

use embassy_net::{IpAddress, IpEndpoint, Stack, dns::DnsQueryType, tcp::TcpSocket};
use embassy_time::{Duration, Timer};
use esp_backtrace as _;
use heapless::{String, Vec};

/// The interval in seconds between the DNS update checks.
const DNS_CHECK_DELAY: &str = env!("DNS_CHECK_DELAY");
/// The fallback interval in seconds between the DNS update checks.
const DNS_CHECK_DELAY_FALLBACK: u64 = 60;
/// The hostname of the update service of your DNS provider.
const DNS_HOST: &str = env!("DNS_HOST");
/// The HTTP request format for updating the DNS.
const DNS_HTTP_REQUEST: &[u8] = env!("DNS_HTTP_REQUEST").as_bytes();

/// The hostname of the API provider for getting the public IP address.
const PUBLIC_IP_PROVIDER_HOST: &str = "api.ipify.org";
/// The HTTP request format for getting the public IP address.
const PUBLIC_IP_PROVIDER_REQUEST: &[u8] =
    b"GET / HTTP/1.1\r\nHost: api.ipify.org\r\nConnection: close\r\n\r\n";

/// The buffer size for the TCP socket.
/// It should be big enough to contain the HTTP requests and responses.
const TCP_BUFFER_SIZE: usize = 1024;

/// The embassy task that handles the DNS updater.
#[embassy_executor::task]
pub async fn dns_updater_task(stack: Stack<'static>) {
    let delay_seconds = get_dns_check_delay(DNS_CHECK_DELAY);
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
                log::error!("{}", public_ip_response);
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
                log::error!("{}", public_ip_response);
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
                let (_, tail) = v.split_once("\r\n\r\n").unwrap_or((v.as_str(), ""));
                if tail.is_empty() {
                    log::warn!("DNS | Response was empty");
                } else {
                    log::info!("...\r\n{}", tail);
                }
                log::info!("DNS | DNS updated. Next check in {} seconds", delay_seconds);
                prev_public_ip = Some(public_ip);
                Timer::after(Duration::from_secs(delay_seconds)).await;
            }
            Ok(None) => log::warn!("DNS | Response was empty"),
            Err(_) => log::error!("DNS | Error updating DNS"),
        };
    }
}

/// Parse the DNS check delay and fallback to the fallback delay if there is an error
fn get_dns_check_delay(delay: &str) -> u64 {
    match delay.parse::<u64>() {
        Ok(v) => v,
        Err(e) => {
            log::error!(
                "DNS | Error parsing DNS_CHECK_DELAY to u64 -> {}: {}",
                e,
                delay
            );

            log::error!(
                "DNS | Using fallback DNS check delay: {} seconds",
                DNS_CHECK_DELAY_FALLBACK
            );
            DNS_CHECK_DELAY_FALLBACK
        }
    }
}

/// Sends an HTTP request to the target host and returns its response.
async fn send_http_request(
    stack: Stack<'_>,
    target_host: &'static str,
    request: &'static [u8],
) -> Result<Option<String<TCP_BUFFER_SIZE>>, ()> {
    // Get public IP address
    let remote_endpoint = get_dns_address(stack, target_host).await?;

    // Setup TCP socket
    let mut rx_buffer = [0; TCP_BUFFER_SIZE];
    let mut tx_buffer = [0; TCP_BUFFER_SIZE];
    let mut socket = TcpSocket::new(stack, &mut rx_buffer, &mut tx_buffer);
    socket.set_timeout(Some(embassy_time::Duration::from_secs(10)));

    // Connect to the remote endpoint
    log::info!("DNS | Connecting to {}...", target_host);
    if let Err(e) = socket.connect(remote_endpoint).await {
        abort_connection(&mut socket).await;
        log::error!("DNS | Error connecting to {}: {:?}", target_host, e);
        return Err(());
    }
    log::info!("DNS | Connected to {}!", target_host);

    // Send the HTTP request to update the IP address
    log::info!("DNS | Writing HTTP request to {}...", target_host);
    if (write_tcp_buf(&mut socket, request).await).is_err() {
        abort_connection(&mut socket).await;
        log::error!("DNS | Error writing request to {}", target_host);
        return Err(());
    }

    // Get response length
    let mut response_buf = [0; TCP_BUFFER_SIZE];
    let response_len = match socket.read(&mut response_buf).await {
        Ok(n) => n,
        Err(e) => {
            abort_connection(&mut socket).await;
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

    socket.close();
    Timer::after(Duration::from_millis(500)).await;
    abort_connection(&mut socket).await;
    response
}

/// Queries the DNS server for the IP address of the target host.
async fn get_dns_address(stack: Stack<'_>, target_host: &'static str) -> Result<IpEndpoint, ()> {
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

    Ok(remote_endpoint)
}
