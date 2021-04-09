#![no_main]
#![no_std]

extern crate panic_halt;
use core::cell::RefCell;

use crate::stm32h7xx_hal::{device, prelude::*};
use cortex_m::asm;
use cortex_m_rt::{entry, exception};
use stm32h7xx_hal;

pub enum Error {
    FlashError,
}
pub type Result<T> = core::result::Result<T, Error>;

pub const FLASH_SECTOR_ADDRESSES: [u32; 16] = [
    // Bank 1, sectors 0-7 128Kb each
    0x0800_0000,
    0x0802_0000,
    0x0804_8000,
    0x0806_0000,
    0x0808_0000,
    0x080A_0000,
    0x080C_0000,
    0x080E_0000,
    // Bank 2, sectors 0-7 128Kb each
    0x0810_0000,
    0x0812_0000,
    0x0814_8000,
    0x0816_0000,
    0x0818_0000,
    0x081A_0000,
    0x081C_0000,
    0x081E_0000,
];
// str 173
pub const FLASH_END: u32 = 0x081F_FFFF;
pub const FLASH_USER: u32 = 0x0810_0000;
static mut FLASH: Option<device::FLASH> = None;

pub fn init(flash: device::FLASH) {
    unsafe { FLASH = Some(flash) };
}

#[entry]
fn main() -> ! {
    let mut _x = 2;
    let mut _y = _x + 1;
    let mut _z = _y;

    loop {
        _z = 2 + _y
    }
}

fn get_flash() -> Result<&'static mut device::FLASH> {
    match unsafe { FLASH.as_mut() } {
        Some(flash) => Ok(flash),
        None => Err(Error::FlashError),
    }
}

fn unlock(flash: &mut device::FLASH) -> Result<()> {
    // bit_is_clear returns True is value of the bit is 1
    while flash.bank2().sr.read().bsy().bit_is_set() {}

    //attempt unlock
    flash
        .bank2()
        .keyr
        .write(|w| unsafe { w.keyr().bits(0x45670123) });
    flash
        .bank2()
        .keyr
        .write(|w| unsafe { w.keyr().bits(0xCDEF89AB) });

    // Verify success - bit_is_clear returns True is value of the bit is 0
    match flash.bank2().cr.read().lock().bit_is_clear() {
        true => Ok(()),
        false => Err(Error::FlashError),
    }
}

fn lock(flash: &mut device::FLASH) {
    flash.bank2().cr.write(|w| w.lock().set_bit())
}

fn check_address_valid(address: u32, length: usize) -> Result<()> {
    if address > (FLASH_END - length as u32 + 1) {
        Err(Error::FlashError)
    } else {
        Ok(())
    }
}

fn check_length_valid(length: usize) -> Result<()> {
    if length % 4 != 0 {
        Err(Error::FlashError)
    } else if length > 2048 {
        Err(Error::FlashError)
    } else {
        Ok(())
    }
}

fn check_length_correct(length: usize, data: &[u8]) -> Result<()> {
    if length != data.len() {
        Err(Error::FlashError)
    } else {
        Ok(())
    }
}

fn erase(address: u32, length: usize) -> Result<()> {
    check_address_valid(address, length)?;
    let address_start = address;
    let address_end = address + length as u32;
    for (index, sector_start) in FLASH_SECTOR_ADDRESSES.iter().enumerate() {
        let sector_start = *sector_start;
        let sector_end = match FLASH_SECTOR_ADDRESSES.get(index + 1) {
            Some(adr) => *adr - 1,
            None => FLASH_END,
        };
        if (address_start >= sector_start && address_start <= sector_end)
            || (address_end >= sector_start && address_end <= sector_end)
            || (address_start <= sector_start && address_end >= sector_end)
        {
            erase_sector(index as u8)?;
        }
    }
    Ok(())
}

fn erase_sector(sector: u8) -> Result<()> {
    if (sector as usize) >= FLASH_SECTOR_ADDRESSES.len() {
        return Err(Error::FlashError);
    }
    let flash = get_flash()?;
    unlock(flash)?;

    while flash.bank2().sr.read().bsy().bit_is_set() {}

    flash.bank2().cr.modify(|_, w| {
        w.ser().set_bit();
        unsafe { w.snb().bits(sector) }
    });

    flash.bank2().cr.modify(|_, w| w.start().set_bit());

    while flash.bank2().sr.read().bsy().bit_is_set() {}

    flash.bank2().cr.modify(|_, w| w.start().clear_bit());

    if flash.bank2().sr.read().wrperr().bit_is_set() {
        Err(Error::FlashError)
    } else {
        Ok(())
    }
}

fn write(address: u32, length: usize, data: &[u8]) -> Result<()> {
    check_address_valid(address, length)?;
    check_length_valid(length)?;
    check_length_correct(length, data)?;

    let flash = get_flash()?;
    unlock(flash)?;

    // Set parallelism to write in 32 bit chunks, and enable programming.
    // Note reset value has 1 for lock so we need to explicitly clear it.

    while flash.bank2().sr.read().bsy().bit_is_set() {}

    for index in 0..(length / 4) {
        let offset = index * 4;
        let word: u32 = (data[offset] as u32)
            | (data[offset + 1] as u32) << 8
            | (data[offset + 2] as u32) << 16
            | (data[offset + 3] as u32) << 24;
        let write_address = (address + offset as u32) as *mut u32;
        unsafe { core::ptr::write_volatile(write_address, word) };

        while flash.bank2().sr.read().bsy().bit_is_set() {}

        let sr = flash.bank2().sr.read();
        if sr.wrperr().bit_is_set() {
            lock(flash);
            return Err(Error::FlashError);
        }
    }

    lock(flash);
    Ok(())
}
fn read(address: u32, length: usize) -> Result<()> {
    check_address_valid(address, length)?;
    let ptr = Flash::get_address(self, offset, size) as *const u8;
    return Ok(unsafe { core::ptr::read(ptr as *const _) });
}
pub struct Address(pub u32);
