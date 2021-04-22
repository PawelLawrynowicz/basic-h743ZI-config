#![no_std]
#![no_main]

use cortex_m_rt::entry;
use cortex_m_semihosting::hprintln;
use dice_hub75::{DrawTarget, Hub75, Pixel, Point, Rgb565};
use embedded_graphics::primitives::Triangle;
use embedded_graphics::{
    egtext,
    fonts::{Font6x6, Font6x8, Text},
    style::TextStyle,
    image::Image,
    drawable::Drawable,
    pixelcolor::Rgb888,
    prelude::Primitive,
    primitives::{Circle, Line},
    style::PrimitiveStyle,
    text_style,
};
use embedded_hal::blocking::delay::DelayUs;
use panic_semihosting as _;
use stm32h7xx_hal::gpio::Speed::VeryHigh;
use stm32h7xx_hal::prelude::*;
use stm32h7xx_hal::{
    delay::{Delay, DelayFromCountDownTimer},
    hal::digital::v2::OutputPin,
};
use stm32h7xx_hal::{device, prelude::*};
use tinytga::Tga;

#[entry]
fn main() -> ! {
    let cp = cortex_m::Peripherals::take().unwrap();
    let dp = device::Peripherals::take().unwrap();

    let pwr = dp.PWR.constrain();
    let pwrcfg = pwr.vos0(&dp.SYSCFG).freeze();

    let rcc = dp.RCC.constrain();
    let ccdr = rcc
        .sys_ck(480.mhz())
        .hclk(240.mhz())
        .pclk1(120.mhz())
        .pclk2(120.mhz())
        .pclk3(120.mhz())
        .pclk4(120.mhz())
        .per_ck(480.mhz())
        .pll1_strategy(stm32h7xx_hal::rcc::PllConfigStrategy::Iterative)
        .pll1_q_ck(480.mhz())
        .freeze(pwrcfg, &dp.SYSCFG);

    let gpioa = dp.GPIOA.split(ccdr.peripheral.GPIOA);
    let gpiob = dp.GPIOB.split(ccdr.peripheral.GPIOB);
    let gpioc = dp.GPIOC.split(ccdr.peripheral.GPIOC);
    let gpiod = dp.GPIOD.split(ccdr.peripheral.GPIOD);
    let gpioe = dp.GPIOE.split(ccdr.peripheral.GPIOE);
    let gpiog = dp.GPIOG.split(ccdr.peripheral.GPIOG);
    let gpiof = dp.GPIOF.split(ccdr.peripheral.GPIOF);

    let R1 = gpiof.pf0.into_push_pull_output().set_speed(VeryHigh);
    let R2 = gpiof.pf3.into_push_pull_output().set_speed(VeryHigh);
    let G1 = gpiof.pf1.into_push_pull_output().set_speed(VeryHigh);
    let G2 = gpiof.pf4.into_push_pull_output().set_speed(VeryHigh);
    let B1 = gpiof.pf2.into_push_pull_output().set_speed(VeryHigh);
    let B2 = gpiof.pf5.into_push_pull_output().set_speed(VeryHigh);
    let A = gpiod.pd0.into_push_pull_output().set_speed(VeryHigh);
    let B = gpiod.pd1.into_push_pull_output().set_speed(VeryHigh);
    let C = gpiog.pg0.into_push_pull_output().set_speed(VeryHigh);
    let D = gpiob.pb1.into_push_pull_output().set_speed(VeryHigh);
    let STROBE = gpioe.pe12.into_push_pull_output().set_speed(VeryHigh);
    let CLK = gpiob.pb10.into_push_pull_output().set_speed(VeryHigh);
    let OE = gpioe.pe6.into_push_pull_output().set_speed(VeryHigh);

    let mut display = Hub75::<_, 128>::new((A, B, C, CLK, STROBE, OE), 3, unsafe {&mut *(0x58021414 as *mut u8)});

    let timer2 = dp.TIM2.timer(2.ms(), ccdr.peripheral.TIM2, &ccdr.clocks);

    let mut delay = DelayFromCountDownTimer::new(timer2);

    let data = include_bytes!("../grappa.tga");
    let tga = Tga::from_slice(data).unwrap();
    let image: Image<Tga, Rgb888> = Image::new(&tga, Point::zero());
    display.draw_image(&image);

    // let triangle = Triangle::new(Point::new(24, 0), Point::new(30, 0), Point::new(27, 4))
    //     .into_styled(PrimitiveStyle::with_fill(Rgb888::new(255, 0, 0)))
    //     .draw(&mut display);

    // let triangle2 = Triangle::new(Point::new(24, 20), Point::new(30, 20), Point::new(27, 16))
    //     .into_styled(PrimitiveStyle::with_fill(Rgb888::new(0, 255, 0)))
    //     .draw(&mut display);

    // let text = Text::new("BTC\n-3,2%", Point::new(0, 0))
    //     .into_styled(TextStyle::new(Font6x6, Rgb888::new(255, 255, 255)))
    //     .draw(&mut display);

    // let text2 = Text::new("ETH\n+2,1%", Point::new(0, 16))
    //     .into_styled(TextStyle::new(Font6x6, Rgb888::new(255, 255, 255)))
    //     .draw(&mut display);

    // let line = Line::new(Point::new(0, 0), Point::new(63, 31))
    //     .into_styled(PrimitiveStyle::with_stroke(Rgb888::new(255,255,255), 1)).draw(&mut display);

    //display.clear(Rgb888::new(255,255,255));

    //display.clear(Rgb888::new(32,32,32));

    

    loop {
        display.output_bcm(&mut delay, 1);
    }
}
