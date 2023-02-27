//! CPU main clock, provided by the SYSCLK divided by a configurable factor

use crate::pac::rcc::RegisterBlock;
use crate::time::Hertz;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum HclkDivider {
    Div1,
    Div2,
    Div4,
    Div8,
    Div16,
    Div64,
    Div128,
    Div256,
    Div512,
}

impl HclkDivider {
    pub fn from_ratio(source: Hertz, target: Hertz) -> Self {
        assert!(source.raw() % target.raw() == 0);

        match source / target {
            0 => panic!("SYSCLK must not be 0. Unable to create HCLK Divider"),
            1 => Self::Div1,
            2 => Self::Div2,
            4 => Self::Div4,
            8 => Self::Div8,
            16 => Self::Div16,
            64 => Self::Div64,
            128 => Self::Div128,
            256 => Self::Div256,
            512 => Self::Div512,
	    _ => panic!("HCLK can only be set to a value that is SYSCLK divided by a power of 2 less or equals to 512 and not 32")
        }
    }

    pub fn bits(self) -> u8 {
        match self {
            Self::Div1 => 0b0000,
            Self::Div2 => 0b1000,
            Self::Div4 => 0b1001,
            Self::Div8 => 0b1010,
            Self::Div16 => 0b1011,
            Self::Div64 => 0b1100,
            Self::Div128 => 0b1101,
            Self::Div256 => 0b1110,
            Self::Div512 => 0b1111,
        }
    }

    pub fn div_factor(self) -> u16 {
        match self {
            Self::Div1 => 1,
            Self::Div2 => 2,
            Self::Div4 => 4,
            Self::Div8 => 8,
            Self::Div16 => 16,
            Self::Div64 => 64,
            Self::Div128 => 128,
            Self::Div256 => 256,
            Self::Div512 => 512,
        }
    }
}

#[derive(Copy, Clone)]
pub struct HclkConfig {
    freq: Hertz,
}

impl HclkConfig {
    pub fn new(freq: Hertz) -> Self {
        Self { freq }
    }

    pub fn freeze(self, sysclk_freq: Hertz, rcc: &RegisterBlock) -> Hertz {
        let divider = HclkDivider::from_ratio(sysclk_freq, self.freq);

        rcc.cfgr
            .modify(|_, w| unsafe { w.hpre().bits(divider.bits()) });

        self.freq
    }
}
