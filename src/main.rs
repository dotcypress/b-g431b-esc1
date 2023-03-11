#![no_std]
#![no_main]

extern crate panic_halt;
extern crate rtic;
extern crate stm32g4xx_hal as hal;

mod pwm;

use defmt_rtt as _;

use hal::gpio::*;
use hal::prelude::*;
use hal::pwm::*;
use hal::stm32;
use hal::syscfg::SysCfgExt;
use hal::timer::*;
use pwm::*;

#[rtic::app(device = hal::stm32, peripherals = true)]
mod app {
    use super::*;

    #[shared]
    struct Shared {}

    #[local]
    struct Local {
        frame: usize,
        pwm: PWM,
        exti: stm32::EXTI,
        button: gpioc::PC10<Input<PullDown>>,
        timer: CountDownTimer<stm32::TIM4>,
    }

    #[init]
    fn init(ctx: init::Context) -> (Shared, Local, init::Monotonics) {
        defmt::info!("init");

        let mut exti = ctx.device.EXTI;
        let mut rcc = ctx.device.RCC.constrain();
        let mut syscfg = ctx.device.SYSCFG.constrain();

        let port_a = ctx.device.GPIOA.split(&mut rcc);
        let port_b = ctx.device.GPIOB.split(&mut rcc);
        let port_c = ctx.device.GPIOC.split(&mut rcc);

        let mut button = port_c.pc10.into_pull_down_input();
        button.make_interrupt_source(&mut syscfg);
        button.trigger_on_edge(&mut exti, SignalEdge::Rising);
        button.enable_interrupt(&mut exti);

        let tim = ctx
            .device
            .TIM1
            .pwm_advanced(
                (
                    port_a.pa8.into_alternate(),
                    port_a.pa9.into_alternate(),
                    port_a.pa10.into_alternate(),
                ),
                &mut rcc,
            )
            .prescaler(1)
            .period(8_500 - 1)
            .with_deadtime(1500.ns())
            .center_aligned();

        let (_, (t1c1, t1c2, t1c3)) = tim.finalize();

        let u = t1c1.into_complementary(port_c.pc13.into_alternate());
        let v = t1c2.into_complementary(port_a.pa12.into_alternate());
        let w = t1c3.into_complementary(port_b.pb15.into_alternate());
        let pwm = PWM::new(u, v, w);

        let timer = Timer::new(ctx.device.TIM4, &rcc.clocks);
        let mut timer = timer.start_count_down(150.hz());
        timer.listen(Event::TimeOut);

        defmt::info!("init completed");

        (
            Shared {},
            Local {
                button,
                exti,
                timer,
                pwm,
                frame: 0,
            },
            init::Monotonics(),
        )
    }

    #[task(binds = TIM4, local = [timer, frame, pwm])]
    fn timer_tick(ctx: timer_tick::Context) {
        let timer_tick::LocalResources { timer, frame, pwm } = ctx.local;
        pwm.set_step(*frame);
        *frame += 1;
        timer.clear_interrupt(Event::TimeOut);
    }

    #[task(binds = EXTI15_10, local = [exti, button])]
    fn button_click(ctx: button_click::Context) {
        defmt::info!("click");
        ctx.local.exti.unpend(hal::exti::Event::GPIO10);
        ctx.local.button.clear_interrupt_pending_bit();
    }

    #[idle]
    fn idle(_: idle::Context) -> ! {
        loop {
            rtic::export::nop();
        }
    }
}
