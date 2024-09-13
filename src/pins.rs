use core::cell::RefCell;
use embassy_sync::blocking_mutex::{raw::CriticalSectionRawMutex, CriticalSectionMutex, Mutex};
use esp_hal::gpio::{GpioPin, OutputOpenDrain};

pub static GPIO2: Mutex<CriticalSectionRawMutex, RefCell<Option<OutputOpenDrain<'_, GpioPin<2>>>>> =
    CriticalSectionMutex::new(RefCell::new(None));
pub static GPIO3: Mutex<CriticalSectionRawMutex, RefCell<Option<OutputOpenDrain<'_, GpioPin<3>>>>> =
    CriticalSectionMutex::new(RefCell::new(None));
pub static GPIO4: Mutex<CriticalSectionRawMutex, RefCell<Option<OutputOpenDrain<'_, GpioPin<4>>>>> =
    CriticalSectionMutex::new(RefCell::new(None));
pub static GPIO5: Mutex<CriticalSectionRawMutex, RefCell<Option<OutputOpenDrain<'_, GpioPin<5>>>>> =
    CriticalSectionMutex::new(RefCell::new(None));
pub static GPIO6: Mutex<CriticalSectionRawMutex, RefCell<Option<OutputOpenDrain<'_, GpioPin<6>>>>> =
    CriticalSectionMutex::new(RefCell::new(None));
pub static GPIO7: Mutex<CriticalSectionRawMutex, RefCell<Option<OutputOpenDrain<'_, GpioPin<7>>>>> =
    CriticalSectionMutex::new(RefCell::new(None));
pub static GPIO8: Mutex<CriticalSectionRawMutex, RefCell<Option<OutputOpenDrain<'_, GpioPin<8>>>>> =
    CriticalSectionMutex::new(RefCell::new(None));
pub static GPIO9: Mutex<CriticalSectionRawMutex, RefCell<Option<OutputOpenDrain<'_, GpioPin<9>>>>> =
    CriticalSectionMutex::new(RefCell::new(None));
