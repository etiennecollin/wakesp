#![no_std]
#![no_main]
#![feature(impl_trait_in_assoc_type)]

mod dns;
mod http_server;
mod pins;
mod utils;

use core::str::FromStr;
use dns::dns_updater_task;
use embassy_executor::Spawner;
use embassy_net::{Config, DhcpConfig, Runner, StackResources};
use embassy_time::{Duration, Timer};
use esp_alloc as _;
use esp_backtrace as _;
use esp_hal::{
    clock::CpuClock,
    gpio::{Level, OutputOpenDrain, Pull},
    riscv::singleton,
    rng::Rng,
    timer::{systimer::SystemTimer, timg::TimerGroup},
};
use esp_hal_embassy as embassy;
use esp_wifi::{
    init,
    wifi::{
        ClientConfiguration, Configuration, WifiController, WifiDevice, WifiEvent, WifiStaDevice,
        WifiState,
    },
};
use http_server::http_server_task;
use log::info;
use pins::*;

/// The hostname of the device.
const HOSTNAME: &str = env!("HOSTNAME");
/// The fallback hostname of the device.
const HOSTNAME_FALLBACK: &str = "wakesp";
/// The password of the wifi network.
const PASSWORD: &str = env!("PASSWORD");
/// The SSID of the wifi network.
const SSID: &str = env!("SSID");

/// The DNS enable flag.
const DNS_ENABLE: &str = env!("DNS_ENABLE");
/// The HTTP server enable flag.
const HTTP_SERVER_ENABLE: &str = env!("HTTP_SERVER_ENABLE");

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    esp_println::logger::init_logger_from_env();
    log::error!("This is error message");
    log::warn!("This is warn message");
    log::info!("This is info message");

    // Initialize the peripherals
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);
    let timg0 = TimerGroup::new(peripherals.TIMG0);
    let systimer = SystemTimer::new(peripherals.SYSTIMER);
    let mut rng = Rng::new(peripherals.RNG);

    esp_alloc::heap_allocator!(72 * 1024);

    // Initialize GPIO pins
    let gpio2 = OutputOpenDrain::new(peripherals.GPIO2, Level::High, Pull::Up);
    let gpio3 = OutputOpenDrain::new(peripherals.GPIO3, Level::High, Pull::Up);
    let gpio4 = OutputOpenDrain::new(peripherals.GPIO4, Level::High, Pull::Up);
    let gpio5 = OutputOpenDrain::new(peripherals.GPIO5, Level::High, Pull::Up);
    let gpio6 = OutputOpenDrain::new(peripherals.GPIO6, Level::High, Pull::Up);
    let gpio7 = OutputOpenDrain::new(peripherals.GPIO7, Level::High, Pull::Up);
    let gpio8 = OutputOpenDrain::new(peripherals.GPIO8, Level::High, Pull::Up);
    let gpio9 = OutputOpenDrain::new(peripherals.GPIO9, Level::High, Pull::Up);
    GPIO2.lock(|x| x.borrow_mut().replace(gpio2));
    GPIO3.lock(|x| x.borrow_mut().replace(gpio3));
    GPIO4.lock(|x| x.borrow_mut().replace(gpio4));
    GPIO5.lock(|x| x.borrow_mut().replace(gpio5));
    GPIO6.lock(|x| x.borrow_mut().replace(gpio6));
    GPIO7.lock(|x| x.borrow_mut().replace(gpio7));
    GPIO8.lock(|x| x.borrow_mut().replace(gpio8));
    GPIO9.lock(|x| x.borrow_mut().replace(gpio9));

    // Initialize the wifi
    let wifi_controller = &*singleton!(: esp_wifi::EspWifiController<'static>  = 
    init(timg0.timer0, rng, peripherals.RADIO_CLK).unwrap())
    .unwrap();

    // Set wifi mode
    let (wifi_interface, controller) =
        esp_wifi::wifi::new_with_mode(wifi_controller, peripherals.WIFI, WifiStaDevice).unwrap();

    // If hostname is empty or longer than 32 chars (limit from embassy_net),
    // the device will use the fallback hostname
    let hostname: &str;
    let trimmed_hostname = HOSTNAME.trim();
    if trimmed_hostname.is_empty() {
        log::warn!(
            "Falling back to default hostname '{}'. No hostname was provided",
            HOSTNAME_FALLBACK
        );
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

    // Generate a seed for the wifi stack
    let mut seed_buf = [0u8; 8];
    rng.read(&mut seed_buf);
    let seed: u64 = u64::from_ne_bytes(seed_buf);

    // Create the wifi stack
    let (stack, runner) = embassy_net::new(
        wifi_interface,
        config,
        singleton!(:StackResources<8> = StackResources::new()).unwrap(),
        seed,
    );

    // Initialize embassy for async tasks
    embassy::init(systimer.alarm0);

    spawner.spawn(connection(controller)).ok();
    spawner.spawn(net_task(runner)).ok();
    if DNS_ENABLE == "true" || DNS_ENABLE == "1" {
        spawner.spawn(dns_updater_task(stack)).ok();
    }
    if HTTP_SERVER_ENABLE == "true" || HTTP_SERVER_ENABLE == "1" {
        spawner.spawn(http_server_task(stack)).ok();
    }
}

#[embassy_executor::task]
async fn connection(mut controller: WifiController<'static>) {
    log::info!("SYS | Started connection task");
    log::info!("SYS | Device capabilities: {:?}", controller.capabilities());
    loop {
        if esp_wifi::wifi::wifi_state() == WifiState::StaConnected {
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
            controller.start().unwrap();
            log::info!("SYS | Wifi started!");
        }
        log::info!("SYS | About to connect...");

        match controller.connect() {
            Ok(_) => log::info!("SYS | Wifi connected!"),
            Err(e) => {
                log::error!("SYS | Failed to connect to wifi: {e:?}");
                Timer::after(Duration::from_millis(5000)).await
            }
        }
    }
}

#[embassy_executor::task]
async fn net_task(mut runner: Runner<'static, WifiDevice<'static, WifiStaDevice>>) {
    runner.run().await
}
