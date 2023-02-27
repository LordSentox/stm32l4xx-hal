use crate::pac::rcc::RegisterBlock;
use crate::time::Hertz;
use fugit::RateExtU32;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MsiFreq {
    #[doc = "range 0 around 100 kHz"]
    RANGE100K = 0,
    #[doc = "range 1 around 200 kHz"]
    RANGE200K = 1,
    #[doc = "range 2 around 400 kHz"]
    RANGE400K = 2,
    #[doc = "range 3 around 800 kHz"]
    RANGE800K = 3,
    #[doc = "range 4 around 1 MHz"]
    RANGE1M = 4,
    #[doc = "range 5 around 2 MHz"]
    RANGE2M = 5,
    #[doc = "range 6 around 4 MHz"]
    RANGE4M = 6,
    #[doc = "range 7 around 8 MHz"]
    RANGE8M = 7,
    #[doc = "range 8 around 16 MHz"]
    RANGE16M = 8,
    #[doc = "range 9 around 24 MHz"]
    RANGE24M = 9,
    #[doc = "range 10 around 32 MHz"]
    RANGE32M = 10,
    #[doc = "range 11 around 48 MHz"]
    RANGE48M = 11,
}

impl MsiFreq {
    pub fn to_hertz(self) -> Hertz {
        (match self {
            Self::RANGE100K => 100_000,
            Self::RANGE200K => 200_000,
            Self::RANGE400K => 400_000,
            Self::RANGE800K => 800_000,
            Self::RANGE1M => 1_000_000,
            Self::RANGE2M => 2_000_000,
            Self::RANGE4M => 4_000_000,
            Self::RANGE8M => 8_000_000,
            Self::RANGE16M => 16_000_000,
            Self::RANGE24M => 24_000_000,
            Self::RANGE32M => 32_000_000,
            Self::RANGE48M => 48_000_000,
        })
        .Hz()
    }

    pub fn freeze(self, rcc: &RegisterBlock, use_lse_calibration: bool) {
        unsafe {
            rcc.cr.modify(|_, w| {
                w.msirange()
                    .bits(self as u8)
                    .msirgsel()
                    .set_bit()
                    .msion()
                    .set_bit();

                // Use LSE to automatically calibrate MSI
                if use_lse_calibration {
                    w.msipllen().set_bit();
                }

                w
            });
        }

        // Wait until MSI is running with the correct configuration
        while rcc.cr.read().msirdy().bit_is_clear() {}
    }
}
