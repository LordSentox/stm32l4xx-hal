use crate::rcc::{RegisterBlock, RCC};
use crate::time::Hertz;
use crate::{flash::ACR, pwr::Pwr};

use super::pclk::{Pclk1Config, Pclk2Config};
use super::MsiFreq;
use super::{
    pll::{PllConfig, PllOutputDivider, PllSource},
    LseConfig, SysclkSource,
};
use super::{
    ClockSecuritySystem, Clocks, CrystalBypass, HclkConfig, HseConfig, SysclkConfig, HSI16_FREQ,
};

/// Clock configuration to set clock settings or reconfigure them.
pub struct CFGR {
    hse: Option<HseConfig>,
    lse: Option<LseConfig>,
    msi: Option<MsiFreq>,
    hsi48_on: bool,
    hsi16_on: bool,
    lsi_on: bool,
    hclk: Option<HclkConfig>,
    pclk1: Option<Pclk1Config>,
    pclk2: Option<Pclk2Config>,
    sysclk: Option<SysclkConfig>,
    pll: Option<PllConfig>,
}

impl CFGR {
    /// Add an HSE to the system
    pub fn enable_hse(
        mut self,
        freq: Hertz,
        bypass: CrystalBypass,
        css: ClockSecuritySystem,
    ) -> Self {
        self.hse = Some(HseConfig::new(freq, bypass, css));
        self
    }
    pub(super) fn hse(&self) -> Option<&HseConfig> {
        self.hse.as_ref()
    }

    /// Add an 32.768 kHz LSE to the system
    pub fn enable_lse(mut self, bypass: CrystalBypass, css: ClockSecuritySystem) -> Self {
        self.lse = Some(LseConfig { bypass, css });

        self
    }

    /// Sets a frequency for the AHB bus
    pub fn set_hclk_freq(mut self, freq: Hertz) -> Self {
        self.hclk = Some(HclkConfig::new(freq));
        self
    }

    /// Enable the 48 MHz USB, RNG, SDMMC HSI clock source. Not available on all stm32l4x6 series
    pub fn enable_hsi48(mut self, on: bool) -> Self {
        self.hsi48_on = on;
        self
    }

    pub fn enable_hsi16(mut self, on: bool) -> Self {
        self.hsi16_on = on;
        self
    }

    /// Enables the MSI with the specified speed
    pub fn enable_msi(mut self, range: MsiFreq) -> Self {
        self.msi = Some(range);
        self
    }
    pub(super) fn msi(&self) -> Option<MsiFreq> {
        self.msi
    }

    /// Sets LSI clock on (the default) or off
    pub fn set_lsi(mut self, on: bool) -> Self {
        self.lsi_on = on;
        self
    }

    /// Sets a frequency for the APB1 bus
    pub fn set_pclk1_freq(mut self, freq: Hertz) -> Self {
        self.pclk1 = Some(Pclk1Config::new(freq));
        self
    }

    /// Sets a frequency for the APB2 bus
    pub fn set_pclk2_freq(mut self, freq: Hertz) -> Self {
        self.pclk2 = Some(Pclk2Config::new(freq));
        self
    }

    /// Sets the system (core) frequency
    pub fn set_sysclk(mut self, source: SysclkSource, freq: Hertz) -> Self {
        self.sysclk = Some(SysclkConfig {
            speed: freq,
            source_clock: source,
        });
        self
    }

    /// Sets the PLL source
    pub fn enable_pll(
        mut self,
        source: PllSource,
        target_freq: Hertz,
        in_div: u8,
        out_mul: u8,
        out_div: PllOutputDivider,
    ) -> Self {
        self.pll = Some(PllConfig::new(
            source,
            target_freq,
            in_div,
            out_mul,
            out_div,
        ));

        self
    }

    pub fn enable_pll_autosetting(
        self,
        _source: PllSource,
        _source_freq: Hertz,
        _target_freq: Hertz,
    ) -> Self {
        todo!()
    }

    pub fn freeze(self, acr: &mut ACR, pwr: &mut Pwr) -> Clocks {
        let rcc = unsafe { &*RCC::ptr() };

        reset_clocks(rcc);
        let mut clocks = Clocks::default();
        self.setup_lsi(rcc, &mut clocks);
        self.setup_lse(rcc, pwr, &mut clocks);
        self.setup_hse(rcc, &mut clocks);
        self.setup_hsi48(rcc, &mut clocks);
        self.setup_hsi16(rcc, &mut clocks);
        self.setup_pll(rcc, &mut clocks);

        let sysclk = self.create_sysclk_config();
        let hclk = self.create_hclk_config(&sysclk);

        self.setup_periph_clocks(rcc, &hclk, &mut clocks);
        self.adjust_flash_wait_states(acr, &hclk);

        self.configure_msi(rcc, &mut clocks);

        self.setup_sysclk(&sysclk, rcc, &mut clocks);
        self.setup_hclk(rcc, &hclk, &sysclk, &mut clocks);

        self.clean_msi(rcc);

        clocks
    }

    fn setup_lsi(&self, rcc: &RegisterBlock, clocks: &mut Clocks) {
        if !self.lsi_on {
            return;
        }

        rcc.csr.modify(|_, w| w.lsion().set_bit());
        while rcc.csr.read().lsirdy().bit_is_clear() {}

        clocks.lsi = true;
    }

    fn setup_lse(&self, rcc: &RegisterBlock, pwr: &mut Pwr, clocks: &mut Clocks) {
        if let Some(lse_cfg) = &self.lse {
            // Unlocke the backup domain
            pwr.cr1.reg().modify(|_, w| w.dbp().set_bit());

            rcc.bdcr.modify(|_, w| {
                // Enable the LSE
                w.lseon().set_bit();

                // Set drive strength if we use a crystal, if a complete oscillator is used, set the LSE bypass bit
                match lse_cfg.bypass {
                    CrystalBypass::Enable => w.lsebyp().set_bit(),
                    CrystalBypass::Disable => unsafe { w.lsedrv().bits(0b11) },
                };

                w
            });

            // Wait until LSE is running
            while rcc.bdcr.read().lserdy().bit_is_clear() {}

            // Make sure to have a backup clock signal if the clock security system is enabled and we have to fall back
            if lse_cfg.css == ClockSecuritySystem::Enable {
                if !self.lsi_on {
                    panic!("The clock security system of the LSE uses LSI as a fallback. LSI must be enabled too");
                }

                // Enable CSS and interrupt
                rcc.bdcr.modify(|_, w| w.lsecsson().set_bit());
                rcc.cier.modify(|_, w| w.lsecssie().set_bit());
            }

            clocks.lse = true;
        }
    }

    fn configure_msi(&self, rcc: &RegisterBlock, clocks: &mut Clocks) {
        if let Some(msi) = self.msi {
            msi.freeze(rcc, self.lse.is_some());

            clocks.msi = Some(msi)
        }
    }

    fn setup_hse(&self, rcc: &RegisterBlock, clocks: &mut Clocks) {
        if let Some(hse) = &self.hse {
            clocks.hse = Some(hse.freeze(rcc));
        }
    }

    fn setup_hsi48(&self, rcc: &RegisterBlock, clocks: &mut Clocks) {
        if self.hsi48_on {
            rcc.crrcr.modify(|_, w| w.hsi48on().set_bit());
            while rcc.crrcr.read().hsi48rdy().bit_is_clear() {}

            clocks.hsi48 = true;
        }
    }

    fn setup_hsi16(&self, rcc: &RegisterBlock, _clocks: &mut Clocks) {
        if self.hsi16_on {
            rcc.cr.write(|w| w.hsion().set_bit());
            while rcc.cr.read().hsirdy().bit_is_clear() {}
        }
    }

    fn setup_pll(&self, rcc: &RegisterBlock, clocks: &mut Clocks) {
        if let Some(pll_cfg) = &self.pll {
            clocks.pll = Some(pll_cfg.freeze(&self, rcc));
        }
    }

    fn create_sysclk_config(&self) -> SysclkConfig {
        if let Some(sysclk) = &self.sysclk {
            sysclk.clone()
        } else if let Some(msi) = self.msi {
            // Use MSI as default, as per standard
            SysclkConfig {
                speed: msi.to_hertz(),
                source_clock: SysclkSource::MSI,
            }
        } else {
            panic!("No SYSCLK configuration has been provided and MSI has not been enabled, which is the fallback. Please provide either");
        }
    }
    fn setup_sysclk(&self, config: &SysclkConfig, rcc: &RegisterBlock, clocks: &mut Clocks) {
        // Check that the speed we want is the speed we actually get from our source clock
        let source_speed = match config.source_clock {
            SysclkSource::HSE => self
                .hse
                .as_ref()
                .expect("SYSCLK set up on HSE, but HSE is not enabled.")
                .speed(),
            SysclkSource::HSI16 => {
                if self.hsi16_on {
                    HSI16_FREQ
                } else {
                    panic!("SYSCLK set up on HSI16, but HSI16 is not enabled.")
                }
            }
            SysclkSource::MSI => self
                .msi
                .as_ref()
                .expect("SYSCLK set up on MSI, but MSI is not enabled.")
                .to_hertz(),
            SysclkSource::PLL => self
                .pll
                .as_ref()
                .expect("SYSCLK set up on PLL, but PLL is not enabled.")
                .speed(),
        };
        assert_eq!(source_speed, config.speed, "The clock feeding SYSCLK does not actually have the correct speed to meet the targeted SYSCLK speed.");

        // Set the SYSCLK source
        rcc.cfgr
            .modify(|_, w| unsafe { w.sw().bits(config.source_clock as u8) });
        while rcc.cfgr.read().sws().bits() != config.source_clock as u8 {}

        clocks.sysclk = config.speed;
    }

    fn create_hclk_config(&self, sysclk_config: &SysclkConfig) -> HclkConfig {
        // Use the requested configuration or a sane default for HCLK.
        match self.hclk {
            Some(config) => config,
            None => HclkConfig::new(sysclk_config.speed), // Same speed as the SYSCLK
        }
    }

    fn setup_hclk(
        &self,
        rcc: &RegisterBlock,
        hclk: &HclkConfig,
        sysclk: &SysclkConfig,
        clocks: &mut Clocks,
    ) {
        clocks.hclk = hclk.freq();
        hclk.freeze(sysclk.speed, rcc);
    }

    fn setup_periph_clocks(&self, rcc: &RegisterBlock, hclk: &HclkConfig, clocks: &mut Clocks) {
        // Use the PCLK configurations or default to the same as HCLK
        let pclk1_config = match self.pclk1 {
            Some(config) => config,
            None => Pclk1Config::new(hclk.freq()),
        };
        let pclk2_config = match self.pclk2 {
            Some(config) => config,
            None => Pclk2Config::new(hclk.freq()),
        };

        (clocks.pclk1, clocks.timclk1) = pclk1_config.freeze(hclk.freq(), rcc);
        (clocks.pclk2, clocks.timclk2) = pclk2_config.freeze(hclk.freq(), rcc);
    }

    fn adjust_flash_wait_states(&self, acr: &mut ACR, hclk: &HclkConfig) {
        let hclk = hclk.freq();
        let latency_bits = if hclk.raw() <= 16_000_000 {
            0b000
        } else if hclk.raw() <= 32_000_000 {
            0b001
        } else if hclk.raw() <= 48_000_000 {
            0b010
        } else if hclk.raw() <= 64_000_000 {
            0b011
        } else {
            0b100
        };

        acr.acr()
            .write(|w| unsafe { w.latency().bits(latency_bits) })
    }

    // Disables the MSI, if it is not configured, since it was used during configuration as the backup clock.
    fn clean_msi(&self, rcc: &RegisterBlock) {
        if self.msi.is_none() {
            rcc.cr
                .modify(|_, w| w.msion().clear_bit().msipllen().clear_bit())
        }
    }
}

fn reset_clocks(rcc: &RegisterBlock) {
    // Switch to MSI as fallback default system clock at 4MHz.
    if rcc.cr.read().msion().bit_is_clear() {
        rcc.cr.modify(|_, w| {
            w.msirgsel().set_bit();
            w.msirange().range4m();
            w.msipllen().clear_bit();
            w.msion().set_bit()
        });

        while rcc.cr.read().msirdy().bit_is_clear() {}
    }
    // Reset clock configuration to default
    if rcc.cfgr.read().sws().bits() != SysclkSource::MSI as u8 {
        rcc.cfgr.reset();
        while rcc.cfgr.read().sws().bits() != SysclkSource::MSI as u8 {}
    }
}

impl Default for CFGR {
    fn default() -> Self {
        Self {
            hse: None,
            lse: None,
            msi: None,
            hsi48_on: false,
            hsi16_on: false,
            lsi_on: false,
            hclk: None,
            pclk1: None,
            pclk2: None,
            sysclk: None,
            pll: None,
        }
    }
}
