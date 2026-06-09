//! Shared helpers for the EFR32 blinky examples (HAL-free, raw
//! `silabs-metapac`). Board specifics — LED pin, register-block version, LF
//! clock source — are feature-gated; enable one of `mg26` or `mg24`.
#![no_std]

#[cfg(all(feature = "mg26", feature = "mg24"))]
compile_error!("enable exactly one board feature: `mg26` or `mg24`, not both");
#[cfg(not(any(feature = "mg26", feature = "mg24")))]
compile_error!("enable one board feature: `mg26` (brd2713a) or `mg24` (XIAO MG24)");

use silabs_metapac as pac;

#[cfg(feature = "mg24")]
use pac::{cmu_v3 as cmu, gpio_v3 as gpio};
#[cfg(feature = "mg26")]
use pac::{cmu_v7 as cmu, gpio_v7 as gpio};

/// EM01GRPACLK frequency feeding TIMER0 (reset-default ~19 MHz HFRCODPLL)
pub const TIMER0_CLOCK_HZ: u32 = 19_000_000;

#[cfg(feature = "mg26")]
const LED_PIN: u32 = 9; // PA09 (brd2713a)
#[cfg(feature = "mg24")]
const LED_PIN: u32 = 7; // PA07 (XIAO MG24)

pub struct Led;

impl Led {
    pub fn new() -> Self {
        pac::CMU.clken0().modify(|w| w.set_gpio(true));

        let mut led = Self;
        led.set_low();

        let shift = (LED_PIN % 8) * 4;
        if LED_PIN < 8 {
            let cur = pac::GPIO.p_model(0).read().0;
            pac::GPIO.p_model(0).write_value(gpio::regs::PortModel(
                (cur & !(0xF << shift)) | (0x4 << shift),
            ));
        } else {
            let cur = pac::GPIO.p_modeh(0).read().0;
            pac::GPIO.p_modeh(0).write_value(gpio::regs::PortModeh(
                (cur & !(0xF << shift)) | (0x4 << shift),
            ));
        }

        led
    }

    pub fn set_high(&mut self) {
        self.write(true);
    }

    pub fn set_low(&mut self) {
        self.write(false);
    }

    fn write(&mut self, high: bool) {
        let cur = pac::GPIO.p_dout(0).read().0;
        let raw = if high {
            cur | (1 << LED_PIN)
        } else {
            cur & !(1 << LED_PIN)
        };
        pac::GPIO.p_dout(0).write_value(gpio::regs::PortDout(raw));
    }
}

impl Default for Led {
    fn default() -> Self {
        Self::new()
    }
}

/// Bring up the LETIMER's 32.768 kHz low-frequency clock source.
///
/// brd2713a: route EM23GRPACLK to the LFRCO.
#[cfg(feature = "mg26")]
pub fn init_lf_clock() {
    pac::CMU
        .em23grpaclkctrl()
        .modify(|w| w.set_clksel(cmu::vals::Em23grpaclkctrlClksel::Lfrco));
}

/// XIAO MG24: the LFRCO is locked by the stock firmware, so use the on-board
/// 32.768 kHz LFXO crystal.
#[cfg(feature = "mg24")]
pub fn init_lf_clock() {
    use pac::lfxo_v1::vals as lfxo_vals;
    let lfxo = pac::LFXO;

    // LFXO load-cap trim: Silicon Labs manufacturing token, else 63.
    let ctune = {
        let raw = unsafe { core::ptr::read_volatile(0x0FE0_009C as *const u8) };
        if raw <= 0x7F {
            raw
        } else {
            63
        }
    };

    pac::CMU.clken0().modify(|w| w.set_lfxo(true));
    lfxo.lock()
        .write(|w| w.set_lockkey(lfxo_vals::Lockkey::Unlock));

    // Disable so CAL/CFG are writable.
    lfxo.ctrl_set().write(|w| w.set_disondemand(true));
    lfxo.ctrl_clr().write(|w| w.set_forceen(true));
    while lfxo.status().read().ens() {}

    lfxo.cal().write(|w| {
        w.set_gain(1);
        w.set_captune(ctune);
    });
    lfxo.cfg().write(|w| {
        w.set_timeout(lfxo_vals::Timeout::Cycles4k);
        w.set_mode(lfxo_vals::Mode::Xtal);
        w.set_highampl(false);
        w.set_agc(true);
    });

    lfxo.ctrl().write(|w| w.set_forceen(true));
    let mut spins = 0u32;
    while !lfxo.status().read().rdy() && spins < 20_000_000 {
        spins += 1;
    }

    pac::CMU
        .em23grpaclkctrl()
        .modify(|w| w.set_clksel(cmu::vals::Em23grpaclkctrlClksel::Lfxo));
}
