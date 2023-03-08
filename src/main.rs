#![no_std]
#![no_main]
#![deny(warnings)]

extern crate panic_halt;
extern crate rtic;
extern crate stm32g4xx_hal as hal;

use defmt_rtt as _;

use hal::gpio::{gpioc::*, *};
use hal::prelude::*;
use hal::stm32;
use hal::timer::*;

#[rtic::app(device = hal::stm32, peripherals = true)]
mod app {
    use super::*;

    #[shared]
    struct Shared {}

    #[local]
    struct Local {
        frame: usize,
        led: PC6<Output<PushPull>>,
        timer: CountDownTimer<stm32::TIM4>,
    }

    #[init]
    fn init(ctx: init::Context) -> (Shared, Local, init::Monotonics) {
        defmt::info!("init");

        let mut rcc = ctx.device.RCC.constrain();

        let port_c = ctx.device.GPIOC.split(&mut rcc);
        let led = port_c.pc6.into_push_pull_output();

        let timer = Timer::new(ctx.device.TIM4, &rcc.clocks);
        let mut timer = timer.start_count_down(20.hz());
        timer.listen(Event::TimeOut);

        defmt::info!("init completed");

        (
            Shared {},
            Local {
                timer,
                led,
                frame: 0,
            },
            init::Monotonics(),
        )
    }

    #[task(binds = TIM4, local = [timer, led, frame])]
    fn timer_tick(ctx: timer_tick::Context) {
        let timer_tick::LocalResources { timer, led, frame } = ctx.local;

        let mask = 0b1001;
        if *frame & mask == mask {
            led.set_low().ok();
        } else {
            led.set_high().ok();
        }

        *frame += 1;
        timer.clear_interrupt(Event::TimeOut);
    }

    #[idle]
    fn idle(_: idle::Context) -> ! {
        loop {
            rtic::export::nop();
        }
    }
}
