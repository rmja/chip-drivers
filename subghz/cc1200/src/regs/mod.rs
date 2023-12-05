use core::mem::transmute;

use crate::gpio::{Gpio0Output, Gpio1Output, Gpio2Output, Gpio3Output, GpioOutput};

mod generated;
mod marc_state;
mod register_address;
pub use generated::*;
pub use marc_state::MarcStateValue;

use self::pri::{FifoCfg, Iocfg0, Iocfg1, Iocfg2, Iocfg3};

pub trait Iocfg {
    /// Analog transfer enable
    ///
    /// # Values
    ///
    /// - false: Standard digital pad
    /// - true: Pad in analog mode (digital GPIO input and output disabled)
    fn gpio_atran(&self) -> bool;
    fn set_gpio_atran(&mut self, value: bool);

    /// Invert output enable
    ///
    /// # Values
    ///
    /// - false: Invert output disabled
    /// - true: Invert output enable
    fn gpio_inv(&self) -> bool;
    fn set_gpio_inv(&mut self, value: bool);

    /// Output selection
    fn gpio_cfg(&self) -> Option<GpioOutput>;
    fn set_gpio_cfg(&mut self, value: GpioOutput);
}

impl Iocfg3 {
    pub fn gpio3_cfg_value(&self) -> Gpio3Output {
        unsafe { transmute(self.gpio3_cfg()) }
    }
}

impl Iocfg for Iocfg3 {
    fn gpio_atran(&self) -> bool {
        self.gpio3_atran()
    }

    fn set_gpio_atran(&mut self, value: bool) {
        self.set_gpio3_atran(value);
    }

    fn gpio_inv(&self) -> bool {
        self.gpio3_inv()
    }

    fn set_gpio_inv(&mut self, value: bool) {
        self.set_gpio3_inv(value);
    }

    fn gpio_cfg(&self) -> Option<GpioOutput> {
        self.gpio3_cfg().try_into().ok()
    }

    fn set_gpio_cfg(&mut self, value: GpioOutput) {
        self.set_gpio3_cfg(value as u8);
    }
}

impl Iocfg2 {
    pub fn gpio2_cfg_value(&self) -> Gpio2Output {
        unsafe { transmute(self.gpio2_cfg()) }
    }
}

impl Iocfg for Iocfg2 {
    fn gpio_atran(&self) -> bool {
        self.gpio2_atran()
    }

    fn set_gpio_atran(&mut self, value: bool) {
        self.set_gpio2_atran(value);
    }

    fn gpio_inv(&self) -> bool {
        self.gpio2_inv()
    }

    fn set_gpio_inv(&mut self, value: bool) {
        self.set_gpio2_inv(value);
    }

    fn gpio_cfg(&self) -> Option<GpioOutput> {
        self.gpio2_cfg().try_into().ok()
    }

    fn set_gpio_cfg(&mut self, value: GpioOutput) {
        self.set_gpio2_cfg(value as u8);
    }
}

impl Iocfg1 {
    pub fn gpio1_cfg_value(&self) -> Gpio1Output {
        unsafe { transmute(self.gpio1_cfg()) }
    }
}

impl Iocfg for Iocfg1 {
    fn gpio_atran(&self) -> bool {
        self.gpio1_atran()
    }

    fn set_gpio_atran(&mut self, value: bool) {
        self.set_gpio1_atran(value);
    }

    fn gpio_inv(&self) -> bool {
        self.gpio1_inv()
    }

    fn set_gpio_inv(&mut self, value: bool) {
        self.set_gpio1_inv(value);
    }

    fn gpio_cfg(&self) -> Option<GpioOutput> {
        self.gpio1_cfg().try_into().ok()
    }

    fn set_gpio_cfg(&mut self, value: GpioOutput) {
        self.set_gpio1_cfg(value as u8);
    }
}

impl Iocfg0 {
    pub fn gpio0_cfg_value(&self) -> Gpio0Output {
        unsafe { transmute(self.gpio0_cfg()) }
    }
}

impl Iocfg for Iocfg0 {
    fn gpio_atran(&self) -> bool {
        self.gpio0_atran()
    }

    fn set_gpio_atran(&mut self, value: bool) {
        self.set_gpio0_atran(value);
    }

    fn gpio_inv(&self) -> bool {
        self.gpio0_inv()
    }

    fn set_gpio_inv(&mut self, value: bool) {
        self.set_gpio0_inv(value);
    }

    fn gpio_cfg(&self) -> Option<GpioOutput> {
        self.gpio0_cfg().try_into().ok()
    }

    fn set_gpio_cfg(&mut self, value: GpioOutput) {
        self.set_gpio0_cfg(value as u8);
    }
}

impl FifoCfg {
    pub fn bytes_in_rxfifo(&self) -> u8 {
        self.fifo_thr() + 1
    }

    pub fn set_bytes_in_rxfifo(&mut self, value: u8) {
        assert!((1..=128).contains(&value));
        self.set_fifo_thr(value - 1);
    }

    pub fn bytes_in_txfifo(&self) -> u8 {
        127 - self.fifo_thr()
    }

    pub fn set_bytes_in_txfifo(&mut self, value: u8) {
        assert!(value <= 127);
        self.set_fifo_thr(127 - value);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fifo_thr_rx() {
        let mut fifocfg = FifoCfg(0);

        fifocfg.set_bytes_in_rxfifo(1);
        assert_eq!(1, fifocfg.bytes_in_rxfifo());
        assert_eq!(0, fifocfg.fifo_thr());

        fifocfg.set_bytes_in_rxfifo(2);
        assert_eq!(2, fifocfg.bytes_in_rxfifo());
        assert_eq!(1, fifocfg.fifo_thr());

        fifocfg.set_bytes_in_rxfifo(127);
        assert_eq!(127, fifocfg.bytes_in_rxfifo());
        assert_eq!(126, fifocfg.fifo_thr());

        fifocfg.set_bytes_in_rxfifo(128);
        assert_eq!(128, fifocfg.bytes_in_rxfifo());
        assert_eq!(127, fifocfg.fifo_thr());
    }

    #[test]
    fn fifo_thr_tx() {
        let mut fifocfg = FifoCfg(0);

        fifocfg.set_bytes_in_txfifo(127);
        assert_eq!(127, fifocfg.bytes_in_txfifo());
        assert_eq!(0, fifocfg.fifo_thr());

        fifocfg.set_bytes_in_txfifo(126);
        assert_eq!(126, fifocfg.bytes_in_txfifo());
        assert_eq!(1, fifocfg.fifo_thr());

        fifocfg.set_bytes_in_txfifo(1);
        assert_eq!(1, fifocfg.bytes_in_txfifo());
        assert_eq!(126, fifocfg.fifo_thr());

        fifocfg.set_bytes_in_txfifo(0);
        assert_eq!(0, fifocfg.bytes_in_txfifo());
        assert_eq!(127, fifocfg.fifo_thr());
    }
}
