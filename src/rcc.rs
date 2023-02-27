//! Reset and Clock Control

pub mod cfgr;
pub mod clocks;
mod enable;
pub mod hclk;
pub mod hse;
pub mod msi;
pub mod pclk;
pub mod pll;

pub use cfgr::CFGR;
pub use clocks::Clocks;
pub use hclk::HclkConfig;
pub use hse::HseConfig;
pub use msi::MsiFreq;

use crate::pac::rcc::RegisterBlock;
use crate::stm32::{rcc, RCC};
use crate::time::Hertz;

pub const MAX_CLOCK_SPEED: Hertz = Hertz::MHz(80);
const HSI16_FREQ: Hertz = Hertz::MHz(16);

/// Extension trait that constrains the `RCC` peripheral
pub trait RccExt {
    /// Constrains the `RCC` peripheral so it plays nicely with the other abstractions
    fn constrain(self) -> Rcc;
}

impl RccExt for RCC {
    fn constrain(self) -> Rcc {
        Rcc {
            ahb1: AHB1::new(),
            ahb2: AHB2::new(),
            ahb3: AHB3::new(),
            apb1r1: APB1R1::new(),
            apb1r2: APB1R2::new(),
            apb2: APB2::new(),
            bdcr: BDCR { _0: () },
            csr: CSR { _0: () },
            crrcr: CRRCR { _0: () },
            ccipr: CCIPR { _0: () },
            cfgr: CFGR::default(),
        }
    }
}

/// Constrained RCC peripheral
pub struct Rcc {
    /// AMBA High-performance Bus (AHB1) registers
    pub ahb1: AHB1,
    /// AMBA High-performance Bus (AHB2) registers
    pub ahb2: AHB2,
    /// AMBA High-performance Bus (AHB3) registers
    pub ahb3: AHB3,
    /// Advanced Peripheral Bus 1 (APB1) registers
    pub apb1r1: APB1R1,
    /// Advanced Peripheral Bus 1 (APB2) registers
    pub apb1r2: APB1R2,
    /// Advanced Peripheral Bus 2 (APB2) registers
    pub apb2: APB2,
    /// Clock configuration register
    pub cfgr: CFGR,
    /// Backup domain control register
    pub bdcr: BDCR,
    /// Control/Status Register
    pub csr: CSR,
    /// Clock recovery RC register
    pub crrcr: CRRCR,
    /// Peripherals independent clock configuration register
    pub ccipr: CCIPR,
}

/// CSR Control/Status Register
pub struct CSR {
    _0: (),
}

impl CSR {
    // TODO remove `allow`
    #[allow(dead_code)]
    pub(crate) fn csr(&mut self) -> &rcc::CSR {
        // NOTE(unsafe) this proxy grants exclusive access to this register
        unsafe { &(*RCC::ptr()).csr }
    }
}

/// Clock recovery RC register
pub struct CRRCR {
    _0: (),
}

impl CRRCR {
    // TODO remove `allow`
    #[allow(dead_code)]
    pub(crate) fn crrcr(&mut self) -> &rcc::CRRCR {
        // NOTE(unsafe) this proxy grants exclusive access to this register
        unsafe { &(*RCC::ptr()).crrcr }
    }

    /// Checks if the 48 MHz HSI is enabled
    pub fn is_hsi48_on(&mut self) -> bool {
        self.crrcr().read().hsi48on().bit()
    }

    /// Checks if the 48 MHz HSI is ready
    pub fn is_hsi48_ready(&mut self) -> bool {
        self.crrcr().read().hsi48rdy().bit()
    }
}

/// Peripherals independent clock configuration register
pub struct CCIPR {
    _0: (),
}

impl CCIPR {
    #[allow(dead_code)]
    pub(crate) fn ccipr(&mut self) -> &rcc::CCIPR {
        // NOTE(unsafe) this proxy grants exclusive access to this register
        unsafe { &(*RCC::ptr()).ccipr }
    }
}

/// BDCR Backup domain control register registers
pub struct BDCR {
    _0: (),
}

impl BDCR {
    // TODO remove `allow`
    #[allow(dead_code)]
    pub(crate) fn enr(&mut self) -> &rcc::BDCR {
        // NOTE(unsafe) this proxy grants exclusive access to this register
        unsafe { &(*RCC::ptr()).bdcr }
    }
}

macro_rules! bus_struct {
    ($($busX:ident => ($EN:ident, $en:ident, $SMEN:ident, $smen:ident, $RST:ident, $rst:ident, $doc:literal),)+) => {
        $(
            #[doc = $doc]
            pub struct $busX {
                _0: (),
            }

            impl $busX {
                pub(crate) fn new() -> Self {
                    Self { _0: () }
                }

                #[allow(unused)]
                pub(crate) fn enr(&self) -> &rcc::$EN {
                    // NOTE(unsafe) this proxy grants exclusive access to this register
                    unsafe { &(*RCC::ptr()).$en }
                }

                #[allow(unused)]
                pub(crate) fn smenr(&self) -> &rcc::$SMEN {
                    // NOTE(unsafe) this proxy grants exclusive access to this register
                    unsafe { &(*RCC::ptr()).$smen }
                }

                #[allow(unused)]
                pub(crate) fn rstr(&self) -> &rcc::$RST {
                    // NOTE(unsafe) this proxy grants exclusive access to this register
                    unsafe { &(*RCC::ptr()).$rst }
                }
            }
        )+
    };
}

bus_struct! {
    AHB1 => (AHB1ENR, ahb1enr, AHB1SMENR, ahb1smenr, AHB1RSTR, ahb1rstr, "Advanced High-performance Bus 1 (AHB1) registers"),
    AHB2 => (AHB2ENR, ahb2enr, AHB2SMENR, ahb2smenr, AHB2RSTR, ahb2rstr, "Advanced High-performance Bus 2 (AHB2) registers"),
    AHB3 => (AHB3ENR, ahb3enr, AHB3SMENR, ahb3smenr, AHB3RSTR, ahb3rstr, "Advanced High-performance Bus 3 (AHB3) registers"),
    APB1R1 => (APB1ENR1, apb1enr1, APB1SMENR1, apb1smenr1, APB1RSTR1, apb1rstr1, "Advanced Peripheral Bus 1 (APB1) registers"),
    APB1R2 => (APB1ENR2, apb1enr2, APB1SMENR2, apb1smenr2, APB1RSTR2, apb1rstr2, "Advanced Peripheral Bus 1 (APB1) registers"),
    APB2 => (APB2ENR, apb2enr, APB2SMENR, apb2smenr, APB2RSTR, apb2rstr, "Advanced Peripheral Bus 2 (APB2) registers"),
}

/// Bus associated to peripheral
pub trait RccBus: crate::Sealed {
    /// Bus type;
    type Bus;
}

/// Enable/disable peripheral
pub trait Enable: RccBus {
    /// Enables peripheral
    fn enable(bus: &mut Self::Bus);

    /// Disables peripheral
    fn disable(bus: &mut Self::Bus);

    /// Check if peripheral enabled
    fn is_enabled() -> bool;

    /// Check if peripheral disabled
    fn is_disabled() -> bool;

    /// # Safety
    ///
    /// Enables peripheral. Takes access to RCC internally
    unsafe fn enable_unchecked();

    /// # Safety
    ///
    /// Disables peripheral. Takes access to RCC internally
    unsafe fn disable_unchecked();
}

/// Enable/disable peripheral in sleep mode
pub trait SMEnable: RccBus {
    /// Enables peripheral
    fn enable_in_sleep_mode(bus: &mut Self::Bus);

    /// Disables peripheral
    fn disable_in_sleep_mode(bus: &mut Self::Bus);

    /// Check if peripheral enabled
    fn is_enabled_in_sleep_mode() -> bool;

    /// Check if peripheral disabled
    fn is_disabled_in_sleep_mode() -> bool;

    /// # Safety
    ///
    /// Enables peripheral. Takes access to RCC internally
    unsafe fn enable_in_sleep_mode_unchecked();

    /// # Safety
    ///
    /// Disables peripheral. Takes access to RCC internally
    unsafe fn disable_in_sleep_mode_unchecked();
}

/// Reset peripheral
pub trait Reset: RccBus {
    /// Resets peripheral
    fn reset(bus: &mut Self::Bus);

    /// # Safety
    ///
    /// Resets peripheral. Takes access to RCC internally
    unsafe fn reset_unchecked();
}

#[derive(Debug, PartialEq)]
/// LSE Configuration
struct LseConfig {
    /// If the clock driving circuitry is bypassed i.e. using an oscillator, not a crystal or
    /// resonator
    bypass: CrystalBypass,
    /// Clock Security System enable/disable
    css: ClockSecuritySystem,
}

/// Crystal bypass selector
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CrystalBypass {
    /// If the clock driving circuitry is bypassed i.e. using an oscillator
    Enable,
    /// If the clock driving circuitry is not bypassed i.e. using a crystal or resonator
    Disable,
}

/// Clock Security System (CSS) selector
///
/// When this is enabled on HSE it will fire of the NMI interrupt on failure and for the LSE the
/// MCU will be woken if in Standby and then the LSECSS interrupt will fire. See datasheet on how
/// to recover for CSS failures.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ClockSecuritySystem {
    /// Enable the clock security system to detect clock failures
    Enable,
    /// Leave the clock security system disabled
    Disable,
}

#[derive(Debug, PartialEq)]
pub struct SysclkConfig {
    pub speed: Hertz,
    pub source_clock: SysclkSource,
}

#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u8)]
pub enum SysclkSource {
    MSI = 0b00,
    HSI16 = 0b01,
    HSE = 0b10,
    PLL = 0b11,
}
