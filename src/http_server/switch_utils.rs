use crate::pins::*;
use core::cell::RefCell;
use embassy_sync::blocking_mutex::{raw::CriticalSectionRawMutex, Mutex};
use embassy_time::{Duration, Timer};
use esp_hal::gpio::{InputPin, Level, OutputOpenDrain, OutputPin};

/// Triggers a GPIO pin based on the provided pin number.
pub async fn switch_command(pin_str: &str) -> Result<(), ()> {
    // Parse the pin number as a u8
    let pin = match pin_str.parse::<u8>() {
        Ok(v) => v,
        Err(_) => {
            log::error!("Switch | Error parsing pin number");
            return Err(());
        }
    };

    // Toggle the pin based on the number
    let result = match pin {
        2 => toggle_pin(&GPIO2, false).await,
        3 => toggle_pin(&GPIO3, false).await,
        4 => toggle_pin(&GPIO4, false).await,
        5 => toggle_pin(&GPIO5, false).await,
        6 => toggle_pin(&GPIO6, false).await,
        7 => toggle_pin(&GPIO7, false).await,
        8 => toggle_pin(&GPIO8, false).await,
        9 => toggle_pin(&GPIO9, false).await,
        _ => {
            log::warn!("Switch | Invalid pin number");
            return Err(());
        }
    };

    // Check if the pin was toggled successfully
    if result.is_err() {
        log::error!("Switch | Error toggling pin GPIO{}", pin_str);
        return Err(());
    }

    log::info!("SWITCH | Triggered pin GPIO{}", pin_str);
    Ok(())
}

/// Toggle a GPIO pin behind a Mutex and a RefCell.
/// This function will toggle the pin for 500ms, then toggle it back.
/// The parameter `toggle_high` determines how the pin will be toggled:
///     - `true`: High -> 500ms -> Low
///     - `false`: Low -> 500ms -> High
pub async fn toggle_pin<T>(
    gpio: &Mutex<CriticalSectionRawMutex, RefCell<Option<OutputOpenDrain<'_, T>>>>,
    toggle_high: bool,
) -> Result<(), ()>
where
    T: InputPin + OutputPin,
{
    let level_0;
    let level_1;

    if toggle_high {
        level_0 = Level::High;
        level_1 = Level::Low;
    } else {
        level_0 = Level::Low;
        level_1 = Level::High;
    }

    set_pin(gpio, level_0)?;
    Timer::after(Duration::from_millis(500)).await;
    set_pin(gpio, level_1)?;

    Ok(())
}

/// Sets a GPIO pin behind a Mutex and a RefCell to the provided level.
pub fn set_pin<T>(
    gpio: &Mutex<CriticalSectionRawMutex, RefCell<Option<OutputOpenDrain<'_, T>>>>,
    level: Level,
) -> Result<(), ()>
where
    T: InputPin + OutputPin,
{
    let mut triggered = false;

    gpio.lock(|pin_locked| {
        if let Ok(mut pin_option) = pin_locked.try_borrow_mut() {
            if let Some(pin) = pin_option.as_mut() {
                pin.set_level(level);
                triggered = true;
            }
        }
    });

    if !triggered {
        return Err(());
    }

    Ok(())
}
