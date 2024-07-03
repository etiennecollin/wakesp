#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

mod dns;
mod udp;
mod utils;

use core::str::FromStr;
use dns::dns_updater_task;
use embassy_executor::Spawner;
use embassy_net::{Config, DhcpConfig, Stack, StackResources};
use embassy_time::{Duration, Timer};
use esp_backtrace as _;
use esp_hal::{
    clock::ClockControl,
    peripherals::Peripherals,
    prelude::*,
    riscv::singleton,
    rng::Rng,
    system::SystemControl,
    timer::{systimer::SystemTimer, timg::TimerGroup},
};
use esp_hal_embassy as embassy;
use esp_wifi::{
    initialize,
    wifi::{
        ClientConfiguration, Configuration, WifiController, WifiDevice, WifiEvent, WifiStaDevice,
        WifiState,
    },
    EspWifiInitFor,
};
use udp::udp_task;

const HOSTNAME: &str = env!("HOSTNAME");
const HOSTNAME_FALLBACK: &str = "wakesp";
const PASSWORD: &str = env!("PASSWORD");
const SSID: &str = env!("SSID");

const DNS_ENABLE: &str = env!("DNS_ENABLE");
const UDP_ENABLE: &str = env!("UDP_ENABLE");

#[main]
async fn main(spawner: Spawner) {
    esp_println::logger::init_logger_from_env();
    log::error!("This is error message");
    log::warn!("This is warn message");
    log::info!("This is info message");

    // Initialize the peripherals
    let peripherals = Peripherals::take();
    let system = SystemControl::new(peripherals.SYSTEM);
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
        peripherals.RADIO_CLK,
        &clocks,
    )
    .unwrap();

    // Set wifi mode
    let (wifi_interface, controller) =
        esp_wifi::wifi::new_with_mode(&init, peripherals.WIFI, WifiStaDevice).unwrap();

    // If hostname is empty or longer than 32 chars (limit from embassy_net),
    // the device will use the fallback hostname
    let hostname: &str;
    let trimmed_hostname = HOSTNAME.trim();
    if trimmed_hostname.is_empty() {
        log::warn!("Falling back to default hostname. No hostname was provided");
        hostname = HOSTNAME_FALLBACK;
    } else if trimmed_hostname.len() > 32 {
        log::warn!("Falling back to default hostname. Hostname has a maximum length of 32 bytes");
        hostname = HOSTNAME_FALLBACK;
    } else {
        hostname = trimmed_hostname
    }

    // Configure DHCPv4
    let mut dhcp_config = DhcpConfig::default();
    dhcp_config.hostname = Some(heapless::String::from_str(hostname).unwrap());
    let config = Config::dhcpv4(dhcp_config);

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
    if DNS_ENABLE == "true" || DNS_ENABLE == "1" {
        spawner.spawn(dns_updater_task(stack)).ok();
    }
    if UDP_ENABLE == "true" || UDP_ENABLE == "1" {
        spawner.spawn(udp_task(stack)).ok();
    }
}

#[embassy_executor::task]
async fn connection(mut controller: WifiController<'static>) {
    log::info!("SYS | Started connection task");
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
                log::error!("SYS | Failed to connect to wifi: {e:?}");
                Timer::after(Duration::from_millis(5000)).await
            }
        }
    }
}

#[embassy_executor::task]
async fn net_task(stack: &'static Stack<WifiDevice<'static, WifiStaDevice>>) {
    stack.run().await
}
