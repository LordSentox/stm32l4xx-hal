use crate::pac::rcc::RegisterBlock;
use crate::time::Hertz;

use super::{ClockSecuritySystem, CrystalBypass};

#[derive(Debug, PartialEq)]
/// HSE Configuration
pub struct HseConfig {
    /// Clock speed of HSE
    speed: Hertz,
    /// If the clock driving circuitry is bypassed i.e. using an oscillator, not a crystal or
    /// resonator
    bypass: CrystalBypass,
    /// Clock Security System enable/disable
    css: ClockSecuritySystem,
}

impl HseConfig {
    pub fn new(speed: Hertz, bypass: CrystalBypass, css: ClockSecuritySystem) -> Self {
        Self { speed, bypass, css }
    }

    pub fn speed(&self) -> Hertz {
        self.speed
    }

    pub fn freeze(&self, rcc: &RegisterBlock) -> Hertz {
        rcc.cr.write(|w| {
            w.hseon().set_bit();

            if self.bypass == CrystalBypass::Enable {
                w.hsebyp().set_bit();
            }

            w
        });

        while rcc.cr.read().hserdy().bit_is_clear() {}

        // Setup CSS
        if self.css == ClockSecuritySystem::Enable {
            // Enable CSS
            rcc.cr.modify(|_, w| w.csson().set_bit());
        }

        self.speed
    }
}
