#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_net::{
    dns::{DnsQueryType, DnsSocket},
    tcp::TcpSocket,
    Config, IpAddress, IpEndpoint, Stack, StackResources,
};
use embassy_time::{Duration, Timer};
use esp_backtrace as _;
use esp_hal::{
    clock::ClockControl, embassy, peripherals::Peripherals, riscv::singleton, rng::Rng,
    system::SystemExt, systimer::SystemTimer, timer::TimerGroup,
};
use esp_println::println;
use esp_wifi::{
    initialize,
    wifi::{
        ClientConfiguration, Configuration, WifiController, WifiDevice, WifiEvent, WifiStaDevice,
        WifiState,
    },
    EspWifiInitFor,
};
mod udp;
use udp::udp_task;

const SSID: &str = env!("SSID");
const PASSWORD: &str = env!("PASSWORD");
const DNS_HTTP_REQUEST: &[u8] = env!("DNS_HTTP_REQUEST").as_bytes();

#[embassy_executor::main(entry = "esp_hal::entry")]
async fn main(spawner: Spawner) {
    esp_println::logger::init_logger_from_env();
    log::error!("This is error message");
    log::warn!("This is warn message");
    log::info!("This is info message");

    // Initialize the peripherals
    let peripherals = Peripherals::take();
    let system = peripherals.SYSTEM.split();
    let clocks = ClockControl::max(system.clock_control).freeze();
    let mut rng = Rng::new(peripherals.RNG);

    // Generate a seed for the wifi stack
    let mut seed_buf = [0u8; 8];
    rng.read(&mut seed_buf);
    let seed: u64 = u64::from_ne_bytes(seed_buf);

    // Initialize the wifi
    let timer = SystemTimer::new(peripherals.SYSTIMER).alarm0;
    let init = initialize(
        EspWifiInitFor::Wifi,
        timer,
        rng,
        system.radio_clock_control,
        &clocks,
    )
    .unwrap();

    // Set wifi mode
    let (wifi_interface, controller) =
        esp_wifi::wifi::new_with_mode(&init, peripherals.WIFI, WifiStaDevice).unwrap();

    // Configure DHCPv4
    let config = Config::dhcpv4(Default::default());

    // Create the wifi stack
    let stack = &*singleton!(:Stack<WifiDevice<'static, WifiStaDevice>> = Stack::new(
        wifi_interface,
        config,
        singleton!(:StackResources<8> = StackResources::new()).unwrap(),
        seed
    ))
    .unwrap();

    // Initialize embassy for async tasks
    let timg0 = TimerGroup::new_async(peripherals.TIMG0, &clocks);
    embassy::init(&clocks, timg0);

    spawner.spawn(connection(controller)).ok();
    spawner.spawn(net_task(stack)).ok();
    spawner.spawn(dns_updater_task(stack)).ok();
    spawner.spawn(udp_task(stack)).ok();
}

#[embassy_executor::task]
async fn connection(mut controller: WifiController<'static>) {
    log::info!("SYS | Start connection task");
    log::info!(
        "SYS | Device capabilities: {:?}",
        controller.get_capabilities()
    );
    loop {
        if esp_wifi::wifi::get_wifi_state() == WifiState::StaConnected {
            // Wait until we're no longer connected
            controller.wait_for_event(WifiEvent::StaDisconnected).await;
            Timer::after(Duration::from_millis(5000)).await
        }

        if !matches!(controller.is_started(), Ok(true)) {
            let client_config = Configuration::Client(ClientConfiguration {
                ssid: SSID.try_into().unwrap(),
                password: PASSWORD.try_into().unwrap(),
                ..Default::default()
            });
            controller.set_configuration(&client_config).unwrap();
            log::info!("SYS | Starting wifi...");
            controller.start().await.unwrap();
            log::info!("SYS | Wifi started!");
        }
        log::info!("SYS | About to connect...");

        match controller.connect().await {
            Ok(_) => log::info!("SYS | Wifi connected!"),
            Err(e) => {
                log::warn!("SYS | Failed to connect to wifi: {e:?}");
                Timer::after(Duration::from_millis(5000)).await
            }
        }
    }
}

#[embassy_executor::task]
async fn net_task(stack: &'static Stack<WifiDevice<'static, WifiStaDevice>>) {
    stack.run().await
}

#[embassy_executor::task]
async fn dns_updater_task(stack: &'static Stack<WifiDevice<'static, WifiStaDevice>>) {
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

            // Create a DNS socket and resolve the IP of dynamicdns.park-your-domain.com
            let socket_dns = DnsSocket::new(stack);
            let dns_query_result = socket_dns
                .query("dynamicdns.park-your-domain.com", DnsQueryType::A)
                .await;
            let ip_list = match dns_query_result {
                Ok(v) => v,
                Err(e) => {
                    log::warn!("DNS | DNS query error: {:?}", e);
                    continue;
                }
            };

            // Get the first IPv4 address in the list
            let remote_endpoint;
            if let Some(ipv4_addr) = ip_list.iter().find(|x| matches!(x, IpAddress::Ipv4(_))) {
                remote_endpoint = IpEndpoint::new(*ipv4_addr, 80);
                log::info!(
                    "DNS | Found IP for dynamicdns.park-your-domain.com: {:?}",
                    ipv4_addr
                );
            } else {
                log::warn!("DNS | No IP found for dynamicdns.park-your-domain.com");
                continue;
            }

            // Setup TCP socket
            let mut rx_buffer = [0; 4096];
            let mut tx_buffer = [0; 4096];
            let mut socket_tcp = TcpSocket::new(stack, &mut rx_buffer, &mut tx_buffer);
            socket_tcp.set_timeout(Some(embassy_time::Duration::from_secs(10)));

            // Connect to the remote endpoint
            log::info!("DNS | Connecting to dynamicdns.park-your-domain.com...");
            let r = socket_tcp.connect(remote_endpoint).await;
            if let Err(e) = r {
                log::warn!("DNS | Connection error: {:?}", e);
                continue;
            }
            log::info!("DNS | Connected!");

            // Send the HTTP request to update the IP address
            let mut buf = [0; 1024];
            log::info!("DNS | Writing HTTP request...");
            let request = write_tcp_buf(&mut socket_tcp, DNS_HTTP_REQUEST).await;

            if let Err(e) = request {
                log::info!("DNS | Error writing HTTP request: {:?}", e);
                continue;
            }

            // Get response length
            let response_len = match socket_tcp.read(&mut buf).await {
                Ok(0) => {
                    log::info!("DNS | Response EOF");
                    continue;
                }
                Ok(n) => n,
                Err(e) => {
                    log::warn!("DNS | Response error: {:?}", e);
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

        // This should run every 24 hours
        Timer::after(Duration::from_secs(86_400)).await;
    }
}

async fn write_tcp_buf(
    socket: &mut TcpSocket<'_>,
    buf: &[u8],
) -> Result<(), embassy_net::tcp::Error> {
    let mut buf = buf;
    while !buf.is_empty() {
        match socket.write(buf).await {
            Ok(0) => log::warn!("write() returned Ok(0)"),
            Ok(n) => buf = &buf[n..],
            Err(e) => return Err(e),
        }
    }
    Ok(())
}
