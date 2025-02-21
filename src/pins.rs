use core::cell::RefCell;
use embassy_sync::blocking_mutex::{CriticalSectionMutex, Mutex, raw::CriticalSectionRawMutex};
use esp_hal::gpio::OutputOpenDrain;

pub static GPIO2: Mutex<CriticalSectionRawMutex, RefCell<Option<OutputOpenDrain<'_>>>> =
    CriticalSectionMutex::new(RefCell::new(None));
pub static GPIO3: Mutex<CriticalSectionRawMutex, RefCell<Option<OutputOpenDrain<'_>>>> =
    CriticalSectionMutex::new(RefCell::new(None));
pub static GPIO4: Mutex<CriticalSectionRawMutex, RefCell<Option<OutputOpenDrain<'_>>>> =
    CriticalSectionMutex::new(RefCell::new(None));
pub static GPIO5: Mutex<CriticalSectionRawMutex, RefCell<Option<OutputOpenDrain<'_>>>> =
    CriticalSectionMutex::new(RefCell::new(None));
pub static GPIO6: Mutex<CriticalSectionRawMutex, RefCell<Option<OutputOpenDrain<'_>>>> =
    CriticalSectionMutex::new(RefCell::new(None));
pub static GPIO7: Mutex<CriticalSectionRawMutex, RefCell<Option<OutputOpenDrain<'_>>>> =
    CriticalSectionMutex::new(RefCell::new(None));
pub static GPIO8: Mutex<CriticalSectionRawMutex, RefCell<Option<OutputOpenDrain<'_>>>> =
    CriticalSectionMutex::new(RefCell::new(None));
pub static GPIO9: Mutex<CriticalSectionRawMutex, RefCell<Option<OutputOpenDrain<'_>>>> =
    CriticalSectionMutex::new(RefCell::new(None));
