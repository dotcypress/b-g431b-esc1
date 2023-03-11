use crate::*;

pub type PwmChannel<C> = Pwm<stm32::TIM1, C, ComplementaryEnabled, ActiveHigh, ActiveHigh>;

pub struct PWM {
    u: PwmChannel<C1>,
    v: PwmChannel<C2>,
    w: PwmChannel<C3>,
    steps: [(u16, u16, u16); 6],
}

impl PWM {
    pub fn new(u: PwmChannel<C1>, v: PwmChannel<C2>, w: PwmChannel<C3>) -> Self {
        let h = u.get_max_duty() / 2;
        let l = 1;
        let steps = [
            (0, h, l),
            (l, h, 0),
            (l, 0, h),
            (0, l, h),
            (h, l, 0),
            (h, 0, l),
        ];
        Self { u, v, w, steps }
    }

    pub fn set_step(&mut self, step: usize) {
        let (u, v, w) = self.steps[step % 6];
        self.u.set_duty(u);
        if u > 0 {
            self.u.enable();
        } else {
            self.u.disable();
        }

        self.v.set_duty(v);
        if v > 0 {
            self.v.enable();
        } else {
            self.v.disable();
        }

        self.w.set_duty(w);
        if w > 0 {
            self.w.enable();
        } else {
            self.w.disable();
        }
    }
}
