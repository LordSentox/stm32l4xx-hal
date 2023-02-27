use crate::pac::rcc::RegisterBlock;
use crate::time::Hertz;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Prescaler {
    Div1,
    Div2,
    Div4,
    Div8,
    Div16,
}

impl Prescaler {
    pub fn from_ratio(source: Hertz, target: Hertz) -> Self {
        assert!(source.raw() % target.raw() == 0);

        match source / target {
	    0 => panic!("HCLK must not be 0. Unable to create APB prescaler."),
	    1 => Self::Div1,
            2 => Self::Div2,
            4 => Self::Div4,
            8 => Self::Div8,
            16 => Self::Div16,
	    _ => panic!("APB prescalers can only be set to a value that is HCLK divided by a power of 2 less or equals to 16")
	}
    }

    pub fn bits(self) -> u8 {
        match self {
            Self::Div1 => 0b000,
            Self::Div2 => 0b100,
            Self::Div4 => 0b101,
            Self::Div8 => 0b110,
            Self::Div16 => 0b111,
        }
    }

    pub fn div_factor(self) -> u16 {
        match self {
            Self::Div1 => 1,
            Self::Div2 => 2,
            Self::Div4 => 4,
            Self::Div8 => 8,
            Self::Div16 => 16,
        }
    }
}

macro_rules! pclk_config {
    ($pclk:ident, $num:literal, $div_bits:ident) => {
        #[derive(Copy, Clone)]
        pub struct $pclk {
            freq: Hertz,
        }

        impl $pclk {
            pub fn new(freq: Hertz) -> Self {
                Self { freq }
            }

            pub fn freeze(self, hclk_freq: Hertz, rcc: &RegisterBlock) -> (Hertz, Hertz) {
                let divider = Prescaler::from_ratio(hclk_freq, self.freq);

                rcc.cfgr
                    .modify(|_, w| unsafe { w.$div_bits().bits(divider.bits()) });

                let timclk_freq = match divider {
                    Prescaler::Div1 => self.freq,
                    _ => 2 * self.freq,
                };

                (self.freq, timclk_freq)
            }
        }
    };
}

pclk_config!(Pclk1Config, 1, ppre1);
pclk_config!(Pclk2Config, 2, ppre2);
