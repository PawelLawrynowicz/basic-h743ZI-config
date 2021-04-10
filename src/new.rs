#![no_std]
#![no_main]

use cortex_m_rt::{entry, exception};
use cortex_m_semihosting::hprintln;
use panic_semihosting as _;
use stm32h7xx_hal::device::FLASH;
use stm32h7xx_hal::device::FLASH as Flash;
use stm32h7xx_hal::gpio::Speed::VeryHigh;
use stm32h7xx_hal::prelude::*;
use stm32h7xx_hal::pwr::Pwr;
use stm32h7xx_hal::stm32::*;

use stm32h7xx_hal::{
    delay::{Delay, DelayFromCountDownTimer},
    hal::digital::v2::OutputPin,
};
use stm32h7xx_hal::{pac, prelude::*};

#[entry]
fn main() -> ! {
    let cp = cortex_m::Peripherals::take().unwrap();
    let dp = pac::Peripherals::take().unwrap();

    let pwr = dp.PWR.constrain();
    let pwrcfg = pwr.vos0(&dp.SYSCFG).freeze();
    flash.bank1.opt
    let rcc = dp.RCC.constrain();
    let ccdr = rcc;
}

fn locked(flash: Flash) -> bool {
    match flash.bank1().cr.read().lock().bit_is_clear() {
        false => return true,
        true => return false,
    }
}

fn unlock() {
    if locked(Flash) {
        self.flash
            .bank1_mut()
            .keyr
            .write(|w| unsafe { w.keyr().bits(0x4567_0123) });
        self.flash
            .bank1_mut()
            .keyr
            .write(|w| unsafe { w.keyr().bits(0xCDEF_89AB) });
    } else {
        panic!("MEMORY UNLOCKED!");
    }
}
