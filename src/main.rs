#![no_main]
#![no_std]

extern crate panic_halt;
use cortex_m::asm;
use cortex_m_rt::{entry, exception};
use stm32h7xx_hal as hal;

#[entry]
fn main() -> ! {
    let mut _x = 2;
    let mut _y = _x + 1;
    let mut _z = _y;

    loop {
        _z = 2 + _y
    }
}
