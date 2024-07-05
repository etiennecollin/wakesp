mod html_responses;
mod wol_utils;

use crate::utils::{wait_for_connection, write_tcp_buf};

use embassy_net::{tcp::TcpSocket, IpListenEndpoint, Stack};
use embassy_time::{Duration, Timer};
use esp_backtrace as _;
use esp_println::println;
use esp_wifi::wifi::{WifiDevice, WifiStaDevice};
use heapless::FnvIndexMap;
use wol_utils::wol_command;

/// The port on which the device will listen for HTTP requests.
const HTTP_LISTEN_PORT: &str = env!("HTTP_LISTEN_PORT");
/// The fallback port on which the device will listen for HTTP requests.
const HTTP_LISTEN_PORT_FALLBACK: u16 = 8080;
/// The buffer size for the TCP socket.
/// It should be big enough to contain the HTTP requests and responses.
const TCP_BUFFER_SIZE: usize = 4096;
/// The enable flag for the WOL feature.
const WOL_ENABLE: &str = env!("WOL_ENABLE");

/// The embassy task that handles the HTTP server.
#[embassy_executor::task]
pub async fn http_server_task(stack: &'static Stack<WifiDevice<'static, WifiStaDevice>>) {
    let listening_port = match HTTP_LISTEN_PORT.parse::<u16>() {
        Ok(v) => v,
        Err(e) => {
            log::error!("HTTP | Could not parse port number: {:?}", e);
            log::error!("HTTP | Using default port {}", HTTP_LISTEN_PORT_FALLBACK);
            HTTP_LISTEN_PORT_FALLBACK
        }
    };

    let listening_endpoint = IpListenEndpoint {
        addr: None,
        port: listening_port,
    };

    // Setup TCP socket
    let mut rx_buffer = [0; TCP_BUFFER_SIZE];
    let mut tx_buffer = [0; TCP_BUFFER_SIZE];
    let response_buffer = &mut [0; TCP_BUFFER_SIZE];
    let mut socket = TcpSocket::new(stack, &mut rx_buffer, &mut tx_buffer);
    socket.set_timeout(Some(embassy_time::Duration::from_secs(10)));

    loop {
        wait_for_connection(stack).await;

        // Wait for incoming connection
        log::info!(
            "HTTP | Waiting for connection on port {}...",
            listening_endpoint.port
        );
        if let Err(e) = socket.accept(listening_endpoint).await {
            log::error!("HTTP | Error accepting connection: {:?}", e);
            abort_connection(&mut socket).await;
            continue;
        };
        log::info!("HTTP | Accepted connection");

        let mut read_buffer = [0u8; TCP_BUFFER_SIZE];
        match socket.read(&mut read_buffer).await {
            Ok(0) => log::info!("HTTP | Connection closed"),
            Ok(len) => {
                // Parse the query as UTF8 and print it
                let query = match core::str::from_utf8(&read_buffer[..len]) {
                    Ok(v) => {
                        println!("{}", v);
                        v
                    }
                    Err(e) => {
                        log::error!("HTTP | Query was not UTF8: {:?}", e);
                        abort_connection(&mut socket).await;
                        continue;
                    }
                };

                let html = match handle_http_query(stack, query).await {
                    Ok(v) => v,
                    Err(_) => html_responses::ERROR,
                };

                let response_len = match generate_http_response(html, response_buffer) {
                    Ok(v) => v,
                    Err(e) => {
                        log::error!("HTTP | Error generating response: {:?}", e);
                        abort_connection(&mut socket).await;
                        continue;
                    }
                };

                if let Err(e) = write_tcp_buf(&mut socket, &response_buffer[..response_len]).await {
                    log::error!("DNS | Error writing response: {:?}", e);
                    abort_connection(&mut socket).await;
                    continue;
                }
            }
            Err(e) => log::error!("HTTP | Error reading response: {:?}", e),
        };

        socket.close();
        Timer::after(Duration::from_millis(1_000)).await;
        abort_connection(&mut socket).await;
    }
}

/// Handle the HTTP query and return the appropriate response.
async fn handle_http_query(
    stack: &'static Stack<WifiDevice<'static, WifiStaDevice>>,
    query: &str,
) -> Result<&'static str, ()> {
    // Parse the command and arguments
    let full_command = query.split_whitespace().nth(1).unwrap_or("/");
    let (command, full_args) = full_command.split_once('?').unwrap_or((full_command, ""));

    // Collect args in a hashmap
    let mut args = FnvIndexMap::<_, _, 4>::new();
    if !full_args.is_empty() {
        full_args.split('&').for_each(|v| {
            let (key, value) = v.split_once('=').unwrap_or((v, ""));
            if let Err(e) = args.insert(key, value) {
                log::warn!("HTTP | Query sepecified too many arguments: {:?}", e);
            }
        });
    }

    match command {
        "/wol" => {
            if WOL_ENABLE != "true" && WOL_ENABLE != "1" {
                return Ok(html_responses::NOT_ENABLED);
            }
            match args.get("mac_addr") {
                Some(v) => {
                    wol_command(stack, v).await?;
                    Ok(html_responses::WOL_SUCCESS)
                }
                None => Ok(html_responses::WOL_INPUT),
            }
        }
        _ => Ok(html_responses::HOME),
    }
}

/// Abort the connection and flush the socket.
async fn abort_connection(socket: &mut TcpSocket<'_>) {
    socket.abort();
    if let Err(e) = socket.flush().await {
        log::error!("HTTP | Flush error: {:?}", e);
    }
}

// Generate a HTTP response with the given html and write it to the buffer.
// The total length of the response must be less than the buffer size.
// Returns the total length of the response.
fn generate_http_response(
    html: &str,
    buffer: &mut [u8; TCP_BUFFER_SIZE],
) -> Result<usize, &'static str> {
    let headers = b"HTTP/1.1 200 OK\r\nConnection: close\r\n\r\n<!DOCTYPE html><html><body>\r\n";
    let tail = b"\r\n</body></html>\r\n";

    let total_length = headers.len() + html.len() + tail.len();

    if total_length > TCP_BUFFER_SIZE {
        return Err("Response does not fit in TCP buffer");
    }

    let mut offset = 0;

    // Copy headers into buffer
    buffer[offset..offset + headers.len()].copy_from_slice(headers);
    offset += headers.len();

    // Copy html into buffer
    buffer[offset..offset + html.len()].copy_from_slice(html.as_bytes());
    offset += html.len();

    // Copy tail into buffer
    buffer[offset..offset + tail.len()].copy_from_slice(tail);

    Ok(total_length)
}