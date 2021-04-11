#![no_std]
#![no_main]

use panic_semihosting as _;
use cortex_m_semihosting::hprintln;
use cortex_m_rt::entry;
use dice_hub75::{DrawTarget, Hub75, Pixel, Point, Rgb565};
use embedded_graphics::{prelude::Primitive, primitives::{Circle, Line}, style::PrimitiveStyle};
use stm32h7xx_hal::{delay::{Delay, DelayFromCountDownTimer}, hal::digital::v2::OutputPin};
use stm32h7xx_hal::{device, prelude::*};
use embedded_hal::blocking::delay::DelayUs;
use stm32h7xx_hal::gpio::Speed::VeryHigh;
use stm32h7xx_hal::prelude::*;

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

    let R1 = gpiob.pb8.into_push_pull_output().set_speed(VeryHigh);
    let R2 = gpioa.pa6.into_push_pull_output().set_speed(VeryHigh);
    let G1 = gpiob.pb9.into_push_pull_output().set_speed(VeryHigh);
    let G2 = gpiob.pb5.into_push_pull_output().set_speed(VeryHigh);
    let B1 = gpioa.pa5.into_push_pull_output().set_speed(VeryHigh);
    let B2 = gpiod.pd14.into_push_pull_output().set_speed(VeryHigh);
    let A = gpioa.pa3.into_push_pull_output().set_speed(VeryHigh);
    let B = gpioc.pc0.into_push_pull_output().set_speed(VeryHigh);
    let C = gpioc.pc3.into_push_pull_output().set_speed(VeryHigh);
    let D = gpiob.pb1.into_push_pull_output().set_speed(VeryHigh);
    let STROBE = gpioe.pe12.into_push_pull_output().set_speed(VeryHigh);
    let CLK = gpiob.pb10.into_push_pull_output().set_speed(VeryHigh);
    let OE = gpioe.pe6.into_push_pull_output().set_speed(VeryHigh);

    let mut display = Hub75::<_>::new((R1, G1, B1, R2, G2, B2, A, B, C, CLK, STROBE, OE), 6);

    let mut x=0;

    //let mut delay = Delay::new(cp.SYST, ccdr.clocks);

    let timer2 = dp
    .TIM2
    .timer(100.ms(), ccdr.peripheral.TIM2, &ccdr.clocks);

    let mut delay = DelayFromCountDownTimer::new(timer2);

    let line = Line::new(Point::new(0, 0), Point::new(31, 31))
    .into_styled(PrimitiveStyle::with_stroke(Rgb565::new(255,255,255), 1));

    let circle = Circle::new(Point::new(10, 10), 5)
    .into_styled(PrimitiveStyle::with_fill(Rgb565::new(255,0,0)));

    //display.draw_line(&line);

    //display.clear(Rgb565::new(255,255,255));

    fn translator(x: i32, y: i32)->Point{
        let panel_rows = 32;
        let panel_cols = 32;

        let is_top_stripe: bool = (y % (panel_rows/2)) < panel_rows/4;

        let new_x;
        let new_y;

        if is_top_stripe{
            new_x=x+panel_cols;
        }
        else {
            new_x=x;
        }

        new_y=(y / (panel_rows/2))*(panel_rows/4)+y%(panel_rows/4);

        Point::new(new_x, new_y)
    }

    let line1 = Line::new(translator(0, 0), translator(7, 7))
    .into_styled(PrimitiveStyle::with_stroke(Rgb565::new(255,255,255), 1));

    let line2 = Line::new(translator(8, 8), translator(15, 15))
    .into_styled(PrimitiveStyle::with_stroke(Rgb565::new(255,255,255), 1));

    let line3 = Line::new(translator(16, 16), translator(23, 23))
    .into_styled(PrimitiveStyle::with_stroke(Rgb565::new(255,255,255), 1));

    let line4 = Line::new(translator(24, 24), translator(31, 31))
    .into_styled(PrimitiveStyle::with_stroke(Rgb565::new(255,255,255), 1));

    loop {
        //let pixel = Pixel(Point::new(32, x), Rgb565::new(125, 125, 0));
        //x+=1;
        
       display.draw_line(&line1);
       display.draw_line(&line2);
       display.draw_line(&line3);
       display.draw_line(&line4);

       // display.draw_circle(&circle).unwrap();

        //display.draw_pixel(pixel).unwrap();

        //hprintln!("x: {}", x);
        
        display.output(&mut delay).ok();
    }
}
