#![no_std]
#![no_main]

use cortex_m_rt::{entry, exception};
use cortex_m_semihosting::hprintln;
use panic_semihosting as _;
use stm32h7xx_hal::device::FLASH;
use stm32h7xx_hal::gpio::Speed::VeryHigh;
use stm32h7xx_hal::prelude::*;
use stm32h7xx_hal::pwr::Pwr;
use stm32h7xx_hal::stm32::*;
use stm32h7xx_hal::{
    delay::{Delay, DelayFromCountDownTimer},
    hal::digital::v2::OutputPin,
};
use stm32h7xx_hal::{pac, prelude::*};
pub struct Flash {
    flash: FLASH,
    pub sector: u8,
}

#[entry]
fn main() -> ! {
    let cp = cortex_m::Peripherals::take().unwrap();
    let dp = pac::Peripherals::take().unwrap();

    let pwr = dp.PWR.constrain();
    let pwrcfg = pwr.vos0(&dp.SYSCFG).freeze();

    let rcc = dp.RCC.constrain();
    let ccdr = rcc
        .sys_ck(120.mhz())
        .hclk(60.mhz())
        .pll1_strategy(stm32h7xx_hal::rcc::PllConfigStrategy::Iterative)
        .freeze(pwrcfg, &dp.SYSCFG);

    let flash = Flash::new(dp.FLASH, 4);

    let mut value: u32 = 2345;
    let offset = 0;

    flash.erase().unwrap();

    flash.write(offset, &value).unwrap();

    value = 6789;

    value = flash.read(offset);

    hprintln!("{}", value).unwrap();

    loop {}
}

#[derive(Debug, Clone)]
pub struct FlashError {
    status: u16,
}

///All errors contain raw value of the FLASH_SR status register (lower 16b)
impl Flash {
    pub fn new(flash: FLASH, sector: u8) -> Self {
        debug_assert!(sector < 16, "invalid sector {}", sector);

        let flash = Flash { flash, sector };

        flash.init();

        flash
    }

    fn init(&self) {
        self.flash
            .bank1_mut()
            .keyr
            .write(|w| unsafe { w.keyr().bits(0x4567_0123) });
        self.flash
            .bank1_mut()
            .keyr
            .write(|w| unsafe { w.keyr().bits(0xCDEF_89AB) });

        self.flash
            .bank1_mut()
            .cr
            .modify(|_, w| unsafe { w.psize().bits(0b10) });

        cortex_m::asm::dsb();
        cortex_m::asm::isb();

        let ps = self.flash.bank1_mut().cr.read().psize().bits();
        hprintln!("PSIZE: {}", ps);

        match self.flash.bank1_mut().cr.read().lock().bit_is_clear() {
            false => panic!("YIKES"),
            true => (),
        }
    }

    pub fn erase(&self) -> Result<(), u16> {
        while self.flash.bank1_mut().sr.read().bsy().bit_is_set() {}

        self.flash.bank1_mut().cr.modify(|_, w| {
            w.ser().set_bit();
            unsafe { w.snb().bits(self.sector) }
        });
        self.flash.bank1_mut().cr.modify(|_, w| w.start().set_bit());

        while self.flash.bank1_mut().sr.read().bsy().bit_is_set() {}

        self.flash
            .bank1_mut()
            .cr
            .modify(|_, w| w.start().clear_bit());

        let status = self.flash.bank1_mut().sr.read();
        if status.wrperr().bit_is_set() {
            self.flash
                .bank1_mut()
                .sr
                .modify(|_, w| w.wrperr().set_bit());
            return Err(status.bits() as u16);
        }

        self.flash.bank1_mut().cr.modify(|_, w| w.ser().clear_bit());
        Ok(())
    }

    fn get_address(&self, offset: usize, access_size: usize) -> usize {
        let (size, address) = match self.sector {
            //RM0090 Rev 18 page 75
            0..=15 => (0x20000, 0x0800_0000 + self.sector as usize * 0x20000),
            _ => panic!("invalid sector {}", self.sector),
        };

        debug_assert!(offset + access_size < size, "access beyond sector limits");

        address + offset
    }

    pub fn write<T>(&self, offset: usize, data: &T) -> Result<(), u16> {
        let size = core::mem::size_of::<T>();
        hprintln!("SIZE: {}", size);
        let src_ptr = (data as *const T) as *const u32;
        let dest_ptr = Flash::get_address(self, offset, size) as *mut u32;

        debug_assert!(size % 4 == 0, "data size not 4-byte aligned");
        debug_assert!(src_ptr as usize % 4 == 0, "data address not 4-byte aligned");

        while self.flash.bank1_mut().sr.read().bsy().bit_is_set() {}

        //check if register operations can be moved out of the loop
        for i in 0..size as isize / 4 {
            self.flash.bank1_mut().cr.modify(|_, w| w.pg().set_bit());
            self.flash.bank1_mut().cr.modify(|_, w| w.fw().set_bit());

            hprintln!("WSPN: {:?}", self.flash.bank1().wpsn_curr.read().bits());

            if !self.flash.bank1_mut().cr.read().lock().bit_is_clear() {
                panic!("YIKES BUT IN WRITE")
            }

            unsafe {
                let zmienna = src_ptr.offset(i);
                *dest_ptr.offset(i) = *zmienna;
                //core::ptr::write_volatile(dest_ptr.offset(i), *zmienna);
            }
            while self.flash.bank1_mut().sr.read().bsy().bit_is_set() {}

            let status = self.flash.bank1_mut().sr.read();
            if status.wrperr().bit_is_set()
                || status.pgserr().bit_is_set()
                || status.operr().bit_is_set()
                || status.incerr().bit_is_set()
                || status.strberr().bit_is_set()
            {
                self.flash
                    .bank1_mut()
                    .sr
                    .write(|w| unsafe { w.bits(0xFFFF) });
                return Err(status.bits() as u16);
            }
        }

        if !self.flash.bank1_mut().cr.read().lock().bit_is_clear() {
            panic!("YIKES BUT IN WRITE");
        }

        self.flash.bank1_mut().cr.modify(|_, w| w.pg().clear_bit());

        Ok(())
    }

    pub fn read<T>(&self, offset: usize) -> T {
        let size = core::mem::size_of::<T>();
        let ptr = Flash::get_address(self, offset, size) as *const u8;
        unsafe { core::ptr::read(ptr as *const _) }
    }

    pub fn read_into<T>(&self, offset: usize, dest: &mut T) {
        let size = core::mem::size_of::<T>();
        let ptr = Flash::get_address(self, offset, size) as *const u8;
        unsafe {
            core::ptr::copy_nonoverlapping(ptr as *const _, dest, 1);
        };
    }
}
