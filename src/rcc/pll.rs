use crate::pac::rcc::RegisterBlock;
use crate::rcc::{HSI16_FREQ, MAX_CLOCK_SPEED};
use crate::time::Hertz;
use fugit::RateExtU32;

use super::CFGR;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum PllOutputDivider {
    Div2,
    Div4,
    Div6,
    Div8,
}
impl PllOutputDivider {
    pub fn bits(self) -> u8 {
        match self {
            Self::Div2 => 0b00,
            Self::Div4 => 0b01,
            Self::Div6 => 0b10,
            Self::Div8 => 0b11,
        }
    }

    pub fn div_factor(self) -> u8 {
        match self {
            Self::Div2 => 2,
            Self::Div4 => 4,
            Self::Div6 => 6,
            Self::Div8 => 8,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// PLL Source
pub enum PllSource {
    /// Multi-speed internal clock
    MSI,
    /// High-speed 16 MHz internal clock
    HSI16,
    /// High-speed external clock
    HSE,
}

impl PllSource {
    pub fn source_bits(self) -> u8 {
        match self {
            Self::MSI => 0b01,
            Self::HSI16 => 0b10,
            Self::HSE => 0b11,
        }
    }
}

pub struct PllConfig {
    source: PllSource,
    target_freq: Hertz,
    in_div: u8,
    out_mul: u8,
    out_div: PllOutputDivider,
}

impl PllConfig {
    pub fn new(
        source: PllSource,
        target_freq: Hertz,
        in_div: u8,
        out_mul: u8,
        out_div: PllOutputDivider,
    ) -> Self {
        assert!(in_div >= 1);
        assert!(in_div <= 8);
        assert!(out_mul >= 8);
        assert!(out_mul <= 86);
        assert!(target_freq <= MAX_CLOCK_SPEED);

        Self {
            source,
            target_freq,
            in_div,
            out_mul,
            out_div,
        }
    }

    pub fn speed(&self) -> Hertz {
        self.target_freq
    }

    pub fn freeze(&self, cfgr: &CFGR, rcc: &RegisterBlock) -> Hertz {
        let clock_freq = match self.source {
            PllSource::HSE => cfgr
                .hse()
                .expect("Please enable the HSE when selecting it as the PLL input clock")
                .speed(),
            PllSource::HSI16 => HSI16_FREQ,
            PllSource::MSI => cfgr
                .msi()
                .expect("Please enable the MSI when selecting it as the PLL input clock")
                .to_hertz(),
        };

        // The clock frequency gets divided before it gets put into the PLL VCO input.
        let source_freq: Hertz = (clock_freq.raw() / self.in_div as u32).Hz();

        assert!(source_freq >= Hertz::MHz(4));
        assert!(source_freq <= Hertz::MHz(16));
        let vco_freq: Hertz = (source_freq.raw() * self.out_mul as u32).Hz();
        assert!(vco_freq >= Hertz::MHz(64));
        assert!(vco_freq <= Hertz::MHz(344));
        let out_clock: Hertz = (vco_freq.raw() / self.out_div.div_factor() as u32).Hz();
        assert!(out_clock <= MAX_CLOCK_SPEED);

        assert_eq!(
            out_clock, self.target_freq,
            "PLL configuration parameters do not match the target frequency you want to achieve"
        );

        // Enable on PLLR
        rcc.pllcfgr.modify(|_, w| unsafe {
            w.pllsrc()
                .bits(self.source.source_bits())
                .pllm()
                .bits(self.in_div - 1)
                .pllr()
                .bits(self.out_div.bits())
                .plln()
                .bits(self.out_mul)
        });

        rcc.cr.modify(|_, w| w.pllon().set_bit());
        while rcc.cr.read().pllrdy().bit_is_clear() {}
        rcc.pllcfgr.modify(|_, w| w.pllren().set_bit());

        out_clock
    }
}
