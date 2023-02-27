use crate::time::Hertz;
use fugit::RateExtU32;

use super::MsiFreq;

/// Frozen clock frequencies
///
/// The existence of this value indicates that the clock configuration can no longer be changed
#[derive(Clone, Copy, Debug)]
pub struct Clocks {
    pub(super) hclk: Hertz,
    pub(super) hsi48: bool,
    pub(super) msi: Option<MsiFreq>,
    pub(super) lsi: bool,
    pub(super) lse: bool,
    pub(super) hse: Option<Hertz>,
    pub(super) pclk1: Hertz,
    pub(super) pclk2: Hertz,
    pub(super) ppre1: u8,
    pub(super) ppre2: u8,
    pub(super) sysclk: Hertz,
    pub(super) timclk1: Hertz,
    pub(super) timclk2: Hertz,
    pub(super) pll: Option<Hertz>,
}

impl Clocks {
    /// Returns the frequency of the AHB
    pub fn hclk(&self) -> Hertz {
        self.hclk
    }

    /// Returns status of HSI48
    pub fn hsi48(&self) -> bool {
        self.hsi48
    }

    // Returns the status of the MSI
    pub fn msi(&self) -> Option<MsiFreq> {
        self.msi
    }

    /// Returns status of the LSI
    pub fn lsi(&self) -> bool {
        self.lsi
    }

    // Return the status of the LSE
    pub fn lse(&self) -> bool {
        self.lse
    }

    /// Returns the frequency of the APB1
    pub fn pclk1(&self) -> Hertz {
        self.pclk1
    }

    /// Returns the frequency of the APB2
    pub fn pclk2(&self) -> Hertz {
        self.pclk2
    }

    /// Get PLL output frequency, if it is active
    pub fn pll(&self) -> Option<Hertz> {
        self.pll
    }

    // TODO remove `allow`
    #[allow(dead_code)]
    pub(crate) fn ppre1(&self) -> u8 {
        self.ppre1
    }
    // TODO remove `allow`
    #[allow(dead_code)]
    pub(crate) fn ppre2(&self) -> u8 {
        self.ppre2
    }

    /// Returns the system (core) frequency
    pub fn sysclk(&self) -> Hertz {
        self.sysclk
    }

    /// Returns the frequency for timers on APB1
    pub fn timclk1(&self) -> Hertz {
        self.timclk1
    }

    /// Returns the frequency for timers on APB2
    pub fn timclk2(&self) -> Hertz {
        self.timclk2
    }
}

impl Default for Clocks {
    fn default() -> Self {
        Self {
            hclk: 4.MHz(),
            hsi48: false,
            msi: Some(MsiFreq::RANGE4M),
            lsi: false,
            lse: false,
            hse: None,
            pclk1: 4.MHz(),
            pclk2: 4.MHz(),
            ppre1: 1,
            ppre2: 1,
            sysclk: 4.MHz(),
            timclk1: 4.MHz(),
            timclk2: 4.MHz(),
            pll: None,
        }
    }
}
