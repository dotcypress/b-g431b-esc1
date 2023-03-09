#![no_std]
#![no_main]

extern crate panic_halt;
extern crate rtic;
extern crate stm32g4xx_hal as hal;

use defmt_rtt as _;

use hal::gpio::*;
use hal::prelude::*;
use hal::pwm::*;
use hal::stm32;
use hal::syscfg::SysCfgExt;
use hal::timer::*;

pub struct PWM {
    ch1: Pwm<stm32::TIM1, C1, ComplementaryEnabled, ActiveHigh, ActiveHigh>,
    ch2: Pwm<stm32::TIM1, C2, ComplementaryEnabled, ActiveHigh, ActiveHigh>,
    ch3: Pwm<stm32::TIM1, C3, ComplementaryEnabled, ActiveHigh, ActiveHigh>,
    steps: [(u16, u16, u16); 6],
}

impl PWM {
    pub fn new(
        ch1: Pwm<stm32::TIM1, C1, ComplementaryEnabled, ActiveHigh, ActiveHigh>,
        ch2: Pwm<stm32::TIM1, C2, ComplementaryEnabled, ActiveHigh, ActiveHigh>,
        ch3: Pwm<stm32::TIM1, C3, ComplementaryEnabled, ActiveHigh, ActiveHigh>,
    ) -> Self {
        let h = ch1.get_max_duty();
        let m = h / 4;
        let l = 0;
        let steps = [
            (h, l, m),
            (h, m, l),
            (m, h, l),
            (l, h, m),
            (l, m, h),
            (m, l, h),
        ];
        Self {
            ch1,
            ch2,
            ch3,
            steps,
        }
    }
}

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
            .prescaler(24)
            .period(8_500 - 1)
            .with_deadtime(1500.ns())
            .center_aligned();

        let (_, (t1c1, t1c2, t1c3)) = tim.finalize();

        let mut ch1 = t1c1.into_complementary(port_c.pc13.into_alternate());
        let mut ch2 = t1c2.into_complementary(port_a.pa12.into_alternate());
        let mut ch3 = t1c3.into_complementary(port_b.pb15.into_alternate());

        ch1.set_duty(0);
        ch2.set_duty(0);
        ch3.set_duty(0);

        ch1.enable();
        ch2.enable();
        ch3.enable();

        let pwm = PWM::new(ch1, ch2, ch3);

        let timer = Timer::new(ctx.device.TIM4, &rcc.clocks);
        let mut timer = timer.start_count_down(50.hz());
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
        let (ch1, ch2, ch3) = pwm.steps[*frame % 6];
        pwm.ch1.set_duty(ch1);
        pwm.ch2.set_duty(ch2);
        pwm.ch3.set_duty(ch3);

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
