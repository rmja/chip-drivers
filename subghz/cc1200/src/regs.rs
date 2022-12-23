use core::mem::transmute;

use bitfield::bitfield;

use crate::gpio::{Gpio0Output, Gpio1Output, Gpio2Output, Gpio3Output, GpioOutput};

pub trait Iocfg {
    fn gpio_cfg(&self) -> Option<GpioOutput>;
    fn set_gpio_cfg(&mut self, value: GpioOutput);
}

impl Iocfg3 {
    pub fn gpio3_cfg_value(&self) -> Gpio3Output {
        unsafe { transmute(self.gpio3_cfg()) }
    }
}

impl Iocfg for Iocfg3 {
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
        assert!(value >= 1 && value <= 128);
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

impl PktCfg0 {
    pub fn length_config_value(&self) -> LengthConfig {
        unsafe { transmute(self.length_config()) }
    }

    pub fn set_length_config_value(&mut self, value: LengthConfig) {
        self.set_length_config(value as u8);
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LengthConfig {
    /// Fixed packet length mode. Packet length configured through the PKT_LEN register
    FixedPacketLength = 0b00,
    /// Variable packet length mode. Packet length configured by the first byte received after sync word
    VariablePacketLength = 0b01,
    /// Infinite packet length mode
    InfinitePacketLength = 0b10,
    /// Variable packet length mode. Length configured by the 5 LSB of the first byte received after sync word
    AltVariablePacketLength = 0b11,
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

// The bitfields below are generated using generate_regs.cs

bitfield! {
    /// GPIO3 IO Pin Configuration
    ///
    /// # Address
    ///
    /// The address of this register is 0x00
    #[derive(Clone, Copy)]
    pub struct Iocfg3(u8);

    /// Analog transfer enable
    ///
    /// # Values
    ///
    /// - 0b: Standard digital pad
    /// - 1b: Pad in analog mode (digital GPIO input and output disabled)
    ///
    /// The default value is 0x00
    pub gpio3_atran, set_gpio3_atran: 7;

    /// Invert output enable
    ///
    /// # Values
    ///
    /// - 0b: Invert output disabled
    /// - 1b: Invert output enable
    ///
    /// The default value is 0x00
    pub gpio3_inv, set_gpio3_inv: 6;

    /// Output selection. Default: PKT_SYNC_RXTX
    pub gpio3_cfg, set_gpio3_cfg: 5, 0;
}

impl Default for Iocfg3 {
    fn default() -> Self {
        Self(0x06)
    }
}

bitfield! {
    /// GPIO2 IO Pin Configuration
    ///
    /// # Address
    ///
    /// The address of this register is 0x01
    #[derive(Clone, Copy)]
    pub struct Iocfg2(u8);

    /// Analog transfer enable. Refer to IOCFG3
    pub gpio2_atran, set_gpio2_atran: 7;

    /// Invert output enable. Refer to IOCFG3
    pub gpio2_inv, set_gpio2_inv: 6;

    /// Output selection. Default: PKT_CRC_OK
    pub gpio2_cfg, set_gpio2_cfg: 5, 0;
}

impl Default for Iocfg2 {
    fn default() -> Self {
        Self(0x07)
    }
}

bitfield! {
    /// GPIO1 IO Pin Configuration
    ///
    /// # Address
    ///
    /// The address of this register is 0x02
    #[derive(Clone, Copy)]
    pub struct Iocfg1(u8);

    /// Analog transfer enable. Refer to IOCFG3
    pub gpio1_atran, set_gpio1_atran: 7;

    /// Invert output enable. Refer to IOCFG3
    pub gpio1_inv, set_gpio1_inv: 6;

    /// Output selection. Default: HIGHZ. Note that GPIO1 is shared with the SPI and act as SO when CSn is asserted (active low). The system must ensure pull up/down on this pin
    pub gpio1_cfg, set_gpio1_cfg: 5, 0;
}

impl Default for Iocfg1 {
    fn default() -> Self {
        Self(0x30)
    }
}

bitfield! {
    /// GPIO0 IO Pin Configuration
    ///
    /// # Address
    ///
    /// The address of this register is 0x03
    #[derive(Clone, Copy)]
    pub struct Iocfg0(u8);

    /// Analog transfer enable. Refer to IOCFG3
    pub gpio0_atran, set_gpio0_atran: 7;

    /// Invert output enable. Refer to IOCFG3
    pub gpio0_inv, set_gpio0_inv: 6;

    /// Output selection. Default: EXT_OSC_EN
    pub gpio0_cfg, set_gpio0_cfg: 5, 0;
}

impl Default for Iocfg0 {
    fn default() -> Self {
        Self(0x3c)
    }
}

bitfield! {
    /// Sync Word Configuration [31:24]
    ///
    /// # Address
    ///
    /// The address of this register is 0x04
    #[derive(Clone, Copy)]
    pub struct Sync3(u8);

    /// Sync word [31:24]
    pub sync31_24, set_sync31_24: 7, 0;
}

impl Default for Sync3 {
    fn default() -> Self {
        Self(0x93)
    }
}

bitfield! {
    /// Sync Word Configuration [23:16]
    ///
    /// # Address
    ///
    /// The address of this register is 0x05
    #[derive(Clone, Copy)]
    pub struct Sync2(u8);

    /// Sync word [23:16]
    pub sync23_16, set_sync23_16: 7, 0;
}

impl Default for Sync2 {
    fn default() -> Self {
        Self(0x0b)
    }
}

bitfield! {
    /// Sync Word Configuration [15:8]
    ///
    /// # Address
    ///
    /// The address of this register is 0x06
    #[derive(Clone, Copy)]
    pub struct Sync1(u8);

    /// Sync word [15:8]
    pub sync15_8, set_sync15_8: 7, 0;
}

impl Default for Sync1 {
    fn default() -> Self {
        Self(0x51)
    }
}

bitfield! {
    /// Sync Word Configuration [7:0]
    ///
    /// # Address
    ///
    /// The address of this register is 0x07
    #[derive(Clone, Copy)]
    pub struct Sync0(u8);

    /// Sync Word [7:0]
    pub sync7_0, set_sync7_0: 7, 0;
}

impl Default for Sync0 {
    fn default() -> Self {
        Self(0xde)
    }
}

bitfield! {
    /// Sync Word Detection Configuration Reg. 1
    ///
    /// # Address
    ///
    /// The address of this register is 0x08
    #[derive(Clone, Copy)]
    pub struct SyncCfg1(u8);

    /// Sync word configuration. When SYNC_MODE = 000b, all samples (noise or data) received after RX mode is entered will either be put in the RX FIFO or output on a GPIO configured as SERIAL_RX. Note that when 4'ary modulation is used the sync word uses 2'ary modulation (the symbol rate is kept the same)
    ///
    /// # Values
    ///
    /// - 000b: No sync word
    /// - 001b: 11 bits [SYNC15_8[2:0]:SYNC7_0]
    /// - 010b: 16 bits [SYNC15_8:SYNC7_0]
    /// - 011b: 18 bits [SYNC23_16[1:0]:SYNC15_8:SYNC7_0]
    /// - 100b: 24 bits [SYNC23_16:SYNC15_8:SYNC7_0]
    /// - 101b: 32 bits [SYNC31_24:SYNC23_16:SYNC15_8:SYNC7_0]
    /// - 110b: 16H bits [SYNC31_24:SYNC23_16]
    /// - 111b: 16D bits (DualSync search). When this setting is used in TX mode [SYNC15_8:SYNC7_0] is transmitted
    ///
    /// The default value is 0x05
    pub sync_mode, set_sync_mode: 7, 5;

    /// Soft decision sync word threshold. A sync word is accepted when the calculated sync word qualifier value (PQT_SYNC_ERR.SYNC_ERROR) is less than SYNC_THR/2). A low threshold value means a strict sync word qualifier (sync word must be of high quality to be accepted) while a high threshold value will accept sync word of a poorer quality (increased probability of detecting ┬æfalse┬Æ sync words)
    pub sync_thr, set_sync_thr: 4, 0;
}

impl Default for SyncCfg1 {
    fn default() -> Self {
        Self(0xaa)
    }
}

bitfield! {
    /// Sync Word Detection Configuration Reg. 0
    ///
    /// # Address
    ///
    /// The address of this register is 0x09
    #[derive(Clone, Copy)]
    pub struct SyncCfg0(u8);

    pub sync_cfg0_not_used, _: 7, 6;

    /// Auto clear enable. Auto clear of symbol rate offset estimate when TOC_CFG.TOC_LIMIT != 0 and MDMCFG1.CARRIER_SENSE_GATE = 1. The  symbol rate offset estimate will be cleared when CARRIER_SENSE is de-asserted. Auto clear of IQIC coefficient when IQIC.IQIC_EN = 1. The receiver image compensation coefficient is cleared when the image signal dissappears
    ///
    /// # Values
    ///
    /// - 0b: Auto clear disabled
    /// - 1b: Auto clear enabled
    ///
    /// The default value is 0x00
    pub auto_clear, set_auto_clear: 5;

    /// Receiver configuration limitation. The decimation factor is given by CHAN_BW.ADC_CIC_DECFACT. When this bit is set, RX filter BW must be less than 1500 kHz.
    /// When RX_CONFIG_LIMITATION = 1 the AGC_CFG1.AGC_WIN_SIZE should be incremented by 1 and the wait time between AGC gain adjustment programmed through AGC_CFG1.AGC_SETTLE_WAIT should be doubled.
    ///
    /// # Values
    ///
    /// - 0b: Symbol Rate <= RX Filter BW/2 = f_xosc/(Decimation Factor*CHAN_BW.BB_CIC_DECFACT*4)[Hz]
    /// - 1b: Symbol Rate <= RX Filter BW = f_xosc/(Decimation Factor*CHAN_BW.BB_CIC_DECFACT*2)[Hz]
    ///
    /// The default value is 0x00
    pub rx_config_limitation, set_rx_config_limitation: 4;

    /// PQT gating enable
    ///
    /// # Values
    ///
    /// - 0b: PQT gating enable
    /// - 1b: PQT gating enabled. The demodulator will not start to look for a sync word before a preamble is detected (i.e. PQT_REACHED is asserted). The preamble detector must be enabled for this feature to work (PREAMBLE_CFG0.PQT_EN = 1)
    ///
    /// The default value is 0x00
    pub pqt_gating_en, set_pqt_gating_en: 3;

    /// External sync detect can be used in blind mode to make the receiver change modem parameters after a sync word has been detected by the MCU. GPIO2 needs to be configured as SYNC_DETECT (IOCFG2.GPIO2_CFG = HIGHZ (48)) and the MCU should set this input when a sync word is detected. This will make the receiver switch modem parameters from sync search settings to packet receive settings similar to what is done in FIFO mode/normal mode
    pub ext_sync_detect, set_ext_sync_detect: 2;

    /// Strict sync word bit check. This feature is useful in cases where the sync word has weak correlation properties (level 3 is the strictest sync check)
    ///
    /// # Values
    ///
    /// - 00b: Strict sync word check level 1
    /// - 01b: Strict sync word check level 2
    /// - 10b: Strict sync word check level 3
    /// - 11b: Strict sync word check disabled
    ///
    /// The default value is 0x03
    pub strict_sync_check, set_strict_sync_check: 1, 0;
}

impl Default for SyncCfg0 {
    fn default() -> Self {
        Self(0x03)
    }
}

bitfield! {
    /// Frequency Deviation Configuration
    ///
    /// # Address
    ///
    /// The address of this register is 0x0A
    #[derive(Clone, Copy)]
    pub struct DeviationM(u8);

    /// Frequency deviation (mantissa part)<BR/>
    /// DEV_E > 0 => f_dev = f_xosc*(256+DEV_M)*2^DEV_E/2^22 [Hz]<BR/>
    /// DEV_E = 0 => f_dev = f_xosc*DEV_M/2^21 [Hz]
    pub dev_m, set_dev_m: 7, 0;
}

impl Default for DeviationM {
    fn default() -> Self {
        Self(0x06)
    }
}

bitfield! {
    /// Modulation Format and Frequency Deviation Configuration
    ///
    /// # Address
    ///
    /// The address of this register is 0x0B
    #[derive(Clone, Copy)]
    pub struct ModcfgDevE(u8);

    /// Modem mode configuration
    pub modem_mode, set_modem_mode: 7, 6;

    /// Modulation format
    ///
    /// # Values
    ///
    /// - 000b: 2-FSK
    /// - 001b: 2-GFSK
    /// - 010b: Reserved
    /// - 011b: ASK/OOK
    /// - 100b: 4-FSK
    /// - 101b: 4-GFSK
    /// - 110b: Reserved
    /// - 111b: Reserved
    ///
    /// The default value is 0x00
    pub mod_format, set_mod_format: 5, 3;

    /// Frequency deviation (exponent part). See DEVIATION_M
    pub dev_e, set_dev_e: 2, 0;
}

impl Default for ModcfgDevE {
    fn default() -> Self {
        Self(0x03)
    }
}

bitfield! {
    /// Digital DC Removal Configuration
    ///
    /// # Address
    ///
    /// The address of this register is 0x0C
    #[derive(Clone, Copy)]
    pub struct DcfiltCfg(u8);

    pub dcfilt_cfg_not_used, _: 7;

    /// DC filter override
    ///
    /// # Values
    ///
    /// - 0b: DC filter algorithm estimates and compensates for DC error
    /// - 1b: Manual DC compensation through registers DCFILTOFFSET_I1, DCFILTOFFSET_I0, DCFILTOFFSET_Q1, and DCFILTOFFSET_Q0
    ///
    /// The default value is 0x01
    pub dcfilt_freeze_coeff, set_dcfilt_freeze_coeff: 6;

    /// Settling period of high pass DC filter after AGC adjustment<BR/>
    /// Sample Rate = f_xosc/Decimation Factor [Hz]<BR/>
    /// The decimation factor is 12, 24, or 48, depending on the CHAN_BW.ADC_CIC_DECFACT setting
    ///
    /// # Values
    ///
    /// - 000b: 8 samples
    /// - 001b: 16 samples
    /// - 010b: 32 samples
    /// - 011b: 64 samples
    /// - 100b: 128 samples
    /// - 101b: 128 samples
    /// - 110b: 128 samples
    /// - 111b: 128 samples
    ///
    /// The default value is 0x01
    pub dcfilt_bw_settle, set_dcfilt_bw_settle: 5, 3;

    /// Cut-off frequency (f_Cut_Off ) of high pass DC filter<BR/>
    /// DCFILT_BW = 0 - 011b:<BR/>
    /// f_Cut_Off_DC_filter ~= f_xosc/(Decimation Factor*2^(DCFILT_BW+3)) [Hz]<BR/>
    /// DCFILT_BW = 110b - 111b:<BR/>
    /// f_Cut_Off_DC_filter ~= f_xosc/(Decimation Factor*2^(2*DCFILT_BW)) [Hz]<BR/>
    /// The decimation factor is 12, 24, or48, depending on the CHAN_BW.ADC_CIC_DECFACT setting
    pub dcfilt_bw, set_dcfilt_bw: 2, 0;
}

impl Default for DcfiltCfg {
    fn default() -> Self {
        Self(0x4c)
    }
}

bitfield! {
    /// Preamble Length Configuration Reg. 1
    ///
    /// # Address
    ///
    /// The address of this register is 0x0D
    #[derive(Clone, Copy)]
    pub struct PreambleCfg1(u8);

    pub preamble_cfg1_not_used, _: 7, 6;

    /// Sets the minimum number of preamble bits to be transmitted
    ///
    /// # Values
    ///
    /// - 0000b: No preamble
    /// - 0001b: 0.5 byte
    /// - 0010b: 1 byte
    /// - 0011b: 1.5 bytes
    /// - 0100b: 2 bytes
    /// - 0101b: 3 bytes
    /// - 0110b: 4 bytes
    /// - 0111b: 5 bytes
    /// - 1000b: 6 bytes
    /// - 1001b: 7 bytes
    /// - 1010b: 8 bytes
    /// - 1011b: 12 bytes
    /// - 1100b: 24 bytes
    /// - 1101b: 30 bytes
    /// - 1110b: Reserved
    /// - 1111b: Reserved
    ///
    /// The default value is 0x05
    pub num_preamble, set_num_preamble: 5, 2;

    /// Preamble byte configuration. PREAMBLE_WORD determines how a preamble byte looks like. Note that when 4'ary modulation is used the preamble uses 2'are modulation (the baud rate is kept the same)
    ///
    /// # Values
    ///
    /// - 00b: 10101010 (0xAA)
    /// - 01b: 01010101 (0x55)
    /// - 10b: 00110011 (0x33)
    /// - 11b: 11001100 (0xCC)
    ///
    /// The default value is 0x00
    pub preamble_word, set_preamble_word: 1, 0;
}

impl Default for PreambleCfg1 {
    fn default() -> Self {
        Self(0x14)
    }
}

bitfield! {
    /// Preamble Detection Configuration Reg. 0
    ///
    /// # Address
    ///
    /// The address of this register is 0x0E
    #[derive(Clone, Copy)]
    pub struct PreambleCfg0(u8);

    /// Preamble detection enable
    ///
    /// # Values
    ///
    /// - 0b: Preamble detection disabled
    /// - 1b: Preamble detection enabled
    ///
    /// The default value is 0x01
    pub pqt_en, set_pqt_en: 7;

    /// PQT start-up timer. PQT_VALID_TIMEOUT sets the number of symbols that must be received before PQT_VALID is asserted
    ///
    /// # Values
    ///
    /// - 000b: 11 symbols
    /// - 001b: 12 symbols
    /// - 010b: 13 symbols
    /// - 011b: 14 symbols
    /// - 100b: 15 symbols
    /// - 101b: 17 symbols
    /// - 110b: 24 symbols
    /// - 111b: 32 symbols
    ///
    /// The default value is 0x05
    pub pqt_valid_timeout, set_pqt_valid_timeout: 6, 4;

    /// Soft Decision PQT. A preamble is detected when the calculated preamble qualifier value (PQT_SYNC_ERR.PQT_ERROR) is less than PQT. A low threshold value means a strict preamble qualifier (preamble must be of high quality to be accepted) while a high threshold value will accept preamble of a poorer quality (increased probability of detecting ┬æfalse┬Æ preamble)
    pub pqt, set_pqt: 3, 0;
}

impl Default for PreambleCfg0 {
    fn default() -> Self {
        Self(0xda)
    }
}

bitfield! {
    /// Digital Image Channel Compensation Configuration
    ///
    /// # Address
    ///
    /// The address of this register is 0x0F
    #[derive(Clone, Copy)]
    pub struct Iqic(u8);

    /// IQ image compensation enable. When this bit is set the following must be true:<BR/>
    /// f_IF > RX Filter BW<BR/>
    /// (see IF_MIX_CFGCMIX_CFG for how to program f_IF)
    ///
    /// # Values
    ///
    /// - 0b: IQ image compensation disabled
    /// - 1b: IQ image compensation enabled
    ///
    /// The default value is 0x01
    pub iqic_en, set_iqic_en: 7;

    /// IQIC update coefficients enable
    ///
    /// # Values
    ///
    /// - 0b: IQIC update coefficients disabled (IQIE_I1, IQIE_I0, IQIE_Q1, IQIE_Q0 registers are not updated)
    /// - 1b: IQIC update coefficients enabled (IQIE_I1, IQIE_I0, IQIE_Q1, IQIE_Q0 registers are updated)
    ///
    /// The default value is 0x01
    pub iqic_update_coeff_en, set_iqic_update_coeff_en: 6;

    /// IQIC block length when settling. The IQIC module will do a coarse estimation of IQ imbalance coefficients during settling mode. Long block length increases settling time and improves image rejection
    ///
    /// # Values
    ///
    /// - 00b: 8 samples
    /// - 01b: 32 samples
    /// - 10b: 128 samples
    /// - 11b: 256 samples
    ///
    /// The default value is 0x00
    pub iqic_blen_settle, set_iqic_blen_settle: 5, 4;

    /// IQIC block length. Long block length increases settling time and improves image rejection
    ///
    /// # Values
    ///
    /// - 00b: 8 samples
    /// - 01b: 32 samples
    /// - 10b: 128 samples
    /// - 11b: 256 samples
    ///
    /// The default value is 0x01
    pub iqic_blen, set_iqic_blen: 3, 2;

    /// IQIC image channel level threshold. Image rejection will be activated when image carrier is present. The IQIC image channel level threshold is an image carrier detector. High threshold imply that image carrier must be high to enable IQIC compensation module
    ///
    /// # Values
    ///
    /// - 00b: > 256
    /// - 01b: > 512
    /// - 10b: > 1024
    /// - 11b: > 2048
    ///
    /// The default value is 0x00
    pub iqic_imgch_level_thr, set_iqic_imgch_level_thr: 1, 0;
}

impl Default for Iqic {
    fn default() -> Self {
        Self(0xc4)
    }
}

bitfield! {
    /// Channel Filter Configuration
    ///
    /// # Address
    ///
    /// The address of this register is 0x10
    #[derive(Clone, Copy)]
    pub struct ChanBw(u8);

    /// ADC_CIC_DECFACT is a table index which programs the first decimation filter and program the RX filter bandwidth. ADC_CIC_DECFACT table index:
    ///
    /// # Values
    ///
    /// - 00b: Decimation factor 12
    /// - 01b: Decimation factor 24
    /// - 10b: Decimation factor 48
    /// - 11b: Reserved
    ///
    /// The default value is 0x02
    pub adc_cic_decfact, set_adc_cic_decfact: 7, 6;

    /// BB_CIC_DECFACT configures the RX filter BW by changing decimation factor in the second decimation filter
    pub bb_cic_decfact, set_bb_cic_decfact: 5, 0;
}

impl Default for ChanBw {
    fn default() -> Self {
        Self(0x94)
    }
}

bitfield! {
    /// General Modem Parameter Configuration Reg. 1
    ///
    /// # Address
    ///
    /// The address of this register is 0x11
    #[derive(Clone, Copy)]
    pub struct Mdmcfg1(u8);

    /// When CARRIER_SENSE_GATE is 1, the demodulator will not start to look for a sync word before CARRIER_SENSE is asserted
    ///
    /// # Values
    ///
    /// - 0b: Search for sync word regardless of CS
    /// - 1b: Do not start sync search before CARRIER_SENSE is asserted
    ///
    /// The default value is 0x00
    pub carrier_sense_gate, set_carrier_sense_gate: 7;

    /// FIFO enable. Specifies if data to/from modem will be passed through the FIFOs or directly to the serial pin
    ///
    /// # Values
    ///
    /// - 0b: Data in/out through the serial pin(s) (the FIFOs are bypassed)
    /// - 1b: Data in/out through the FIFOs
    ///
    /// The default value is 0x01
    pub fifo_en, set_fifo_en: 6;

    /// Manchester mode enable. Manchester encoding/decoding is only applicable to payload data including optional CRC. Manchester encoding/decoding is not supported for 4-(G)FSK
    ///
    /// # Values
    ///
    /// - 0b: NRZ
    /// - 1b: Manchester encoding/decoding
    ///
    /// The default value is 0x00
    pub manchester_en, set_manchester_en: 5;

    /// Invert data enable. Invert payload data stream in RX and TX (only applicable to payload data including optional CRC)
    ///
    /// # Values
    ///
    /// - 0b: Invert data disabled
    /// - 1b: Invert data enabled
    ///
    /// The default value is 0x00
    pub invert_data_en, set_invert_data_en: 4;

    /// Collision detect enable. After a sync word is detected (SYNC_EVENT asserted), the receiver will always receive a packet. If collision detection is enabled, the receiver will continue to search for preamble. If a new preamble is found (PQT_REACHED asserted) and the RSSI has increased  10 or 16 dB during packet reception (depending on AGC_CFG1.RSSI_STEP_THR) a collision is detected and the COLLISION_FOUND flag will be asserted
    ///
    /// # Values
    ///
    /// - 0b: Collision detect disabled
    /// - 1b: collision detect enabled
    ///
    /// The default value is 0x00
    pub collision_detect_en, set_collision_detect_en: 3;

    /// Fixed DVGA gain configuration. The DVGA configuration has impact on the RSSI  offset
    ///
    /// # Values
    ///
    /// - 00b: 0 dB DVGA (preferred setting for RX filter bandwidth < 100 kHz)
    /// - 01b: -18 dB DVGA (preferred setting for RX filter bandwidth >= 100 kHz)
    /// - 10b: 6 dB DVGA
    /// - 11b: 9 dB DVGA
    ///
    /// The default value is 0x03
    pub dvga_gain, set_dvga_gain: 2, 1;

    /// Configure the number of active receive channels. If this bit is set the power consumption will be reduced but the sensitivity level will be reduced by ~3 dB. Image rejection will not work
    ///
    /// # Values
    ///
    /// - 0b: IQ-channels
    /// - 1b: Only I-channel
    ///
    /// The default value is 0x00
    pub single_adc_en, set_single_adc_en: 0;
}

impl Default for Mdmcfg1 {
    fn default() -> Self {
        Self(0x46)
    }
}

bitfield! {
    /// General Modem Parameter Configuration Reg. 0
    ///
    /// # Address
    ///
    /// The address of this register is 0x12
    #[derive(Clone, Copy)]
    pub struct Mdmcfg0(u8);

    /// For test purposes only, use values from SmartRF Studio.
    pub mdmcfg0_reserved7, set_mdmcfg0_reserved7: 7;

    /// Transparent mode enable
    ///
    /// # Values
    ///
    /// - 0b: Transparent mode disabled
    /// - 1b: Transparent mode enabled
    ///
    /// The default value is 0x00
    pub transparent_mode_en, set_transparent_mode_en: 6;

    /// Transparent signal interpolation factor. The sample rate gives the jitter of the samples and the sample rate is given by:<BR/>
    /// Sample Rate = f_xosc*Interpolation Facor/(Decimation Factor*CHAN_BW.BB_CIC_DECFACT) [Hz]<BR/>
    /// The decimation factor is given by CHAN_BW.ADC_CIC_DECFACT while the interpolation factor is given below
    ///
    /// # Values
    ///
    /// - 00b: 1x transparent signal interpolated one time before output (reset)
    /// - 01b: 2x transparent signal interpolated two times before output
    /// - 10b: 4x transparent signal interpolated four times before output
    /// - 11b: Reserved
    ///
    /// The default value is 0x00
    pub transparent_intfact, set_transparent_intfact: 5, 4;

    /// Transparent data filter and extended data filter enable. Enabling transparent data filter and/or extended data filter might Improve sensitivity. When TRANSPARENT_MODE_EN = 0 this bit should only be set when RX filter bandwidth/symbol rate > 10 and TOC_CFG.TOC_LIMIT = 0. The table below shows the status of the transparent data filter and the extended data filter for all combinations of TRANSPARENT_MODE_EN (MSB) and DATA_FILTER_EN (LSB)
    ///
    /// # Values
    ///
    /// - 00b: Transparent data filter disabled and extended data filter disabled
    /// - 01b: Transparent data filter disabled and extended data filter enabled
    /// - 10b: Transparent data filter disabled and extended data filter disabled
    /// - 11b: Transparent data filter enabled and extended data filter disabled
    ///
    /// The default value is 0x01
    pub data_filter_en, set_data_filter_en: 3;

    /// Viterbi detection enable. Enabling Viterbi detection improves the sensitivity. The latency from the antenna to the signal is available in the RXFIFO or on the GPIO is increased by 5 bits for 2-ary modulation formats and 10 bits for 4-ary modulation formats. Minimum packet length = 2 bytes when Viterbi Detection and 4-(G)FSK is enabled
    ///
    /// # Values
    ///
    /// - 0b: Viterbi detection disabled
    /// - 1b: Viterbi detection enabled
    ///
    /// The default value is 0x01
    pub viterbi_en, set_viterbi_en: 2;

    /// For test purposes only, use values from SmartRF Studio.
    pub mdmcfg0_reserved1, set_mdmcfg0_reserved1: 1;

    /// For test purposes only, use values from SmartRF Studio.
    pub mdmcfg0_reserved0, set_mdmcfg0_reserved0: 0;
}

impl Default for Mdmcfg0 {
    fn default() -> Self {
        Self(0x0d)
    }
}

bitfield! {
    /// Symbol Rate Configuration Exponent and Mantissa [19:16]
    ///
    /// # Address
    ///
    /// The address of this register is 0x13
    #[derive(Clone, Copy)]
    pub struct SymbolRate2(u8);

    /// Symbol rate (exponent part)<BR/>
    /// SRATE_E > 0 => Symbol Rate = f_xosc*(2^20+SRATE_M)*2^SRATE_E/2^39 [ksps]<BR/>
    /// SRATE_E = 0 => Symbol Rate = f_xosc*SRATE_M/2^38 [ksps]
    pub srate_e, set_srate_e: 7, 4;

    /// Symbol rate (mantissa part [19:16]). See SRATE_E
    pub srate_m_19_16, set_srate_m_19_16: 3, 0;
}

impl Default for SymbolRate2 {
    fn default() -> Self {
        Self(0x43)
    }
}

bitfield! {
    /// Symbol Rate Configuration Mantissa [15:8]
    ///
    /// # Address
    ///
    /// The address of this register is 0x14
    #[derive(Clone, Copy)]
    pub struct SymbolRate1(u8);

    /// Symbol rate (mantissa part [15:8]). See SYMBOL_RATE2
    pub srate_m_15_8, set_srate_m_15_8: 7, 0;
}

impl Default for SymbolRate1 {
    fn default() -> Self {
        Self(0xa9)
    }
}

bitfield! {
    /// Symbol Rate Configuration Mantissa [7:0]
    ///
    /// # Address
    ///
    /// The address of this register is 0x15
    #[derive(Clone, Copy)]
    pub struct SymbolRate0(u8);

    /// Symbol rate (mantissa part [7:0]). See SYMBOL_RATE2
    pub srate_m_7_0, set_srate_m_7_0: 7, 0;
}

impl Default for SymbolRate0 {
    fn default() -> Self {
        Self(0x2a)
    }
}

bitfield! {
    /// AGC Reference Level Configuration
    ///
    /// # Address
    ///
    /// The address of this register is 0x16
    #[derive(Clone, Copy)]
    pub struct AgcRef(u8);

    /// AGC reference level. The AGC reference level must be higher than the minimum SNR to the demodulator. The AGC reduces the analog front end gain when the magnitude output from channel filter > AGC reference level. An optimum AGC reference level is given by several conditions, but a rule of thumb is to use the formula:<BR/>
    /// AGC_REFERENCE = 10*log10(RX Filter BW) - 92 - RSSI Offset<BR/>
    /// For Zero-IF configuration, AGC hysteresis > 3 dB, or modem format which needs SNR>15 dB a higher AGC reference value is needed
    /// </br>
    pub agc_reference, set_agc_reference: 7, 0;
}

impl Default for AgcRef {
    fn default() -> Self {
        Self(0x36)
    }
}

bitfield! {
    /// Carrier Sense Threshold Configuration
    ///
    /// # Address
    ///
    /// The address of this register is 0x17
    #[derive(Clone, Copy)]
    pub struct AgcCsThr(u8);

    /// AGC carrier sense threshold. Two's complement number with 1 dB resolution
    pub agc_cs_threshold, set_agc_cs_threshold: 7, 0;
}

impl Default for AgcCsThr {
    fn default() -> Self {
        Self(0x00)
    }
}

bitfield! {
    /// RSSI Offset Configuration
    ///
    /// # Address
    ///
    /// The address of this register is 0x18
    #[derive(Clone, Copy)]
    pub struct AgcGainAdjust(u8);

    /// AGC gain adjustment. This register is used to adjust RSSI[11:0] to the actual carrier input signal level to compensate for interpolation gains (two's complement with 1 dB resolution)
    pub gain_adjustment, set_gain_adjustment: 7, 0;
}

impl Default for AgcGainAdjust {
    fn default() -> Self {
        Self(0x00)
    }
}

bitfield! {
    /// Automatic Gain Control Configuration Reg. 3
    ///
    /// # Address
    ///
    /// The address of this register is 0x19
    #[derive(Clone, Copy)]
    pub struct AgcCfg3(u8);

    /// AGC behavior after sync word detection
    ///
    /// # Values
    ///
    /// - 000b: No AGC gain freeze. Keep computing/updating RSSI
    /// - 001b: AGC gain freeze. Keep computing/updating RSSI
    /// - 010b: No AGC gain freeze. Keep computing/updating RSSI (AGC slow mode enabled)
    /// - 011b: Freeze both AGC gain and RSSI
    /// - 100b: No AGC gain freeze. Keep computing/updating RSSI
    /// - 101b: Freeze both AGC gain and RSSI
    /// - 110b: No AGC gain freeze. Keep computing/updating RSSI (AGC slow mode enabled)
    /// - 111b: Freeze both AGC gain and RSSI
    ///
    /// The default value is 0x05
    pub agc_sync_behaviour, set_agc_sync_behaviour: 7, 5;

    /// AGC minimum gain. Limits the AGC minimum gain compared to the preset gain table range. AGC_MIN_GAIN can have a value in the range<BR/>
    /// 0 to 17 when AGC_CFG2.FE_PERFORMANCE_MODE = 0 or 1,<BR/>
    /// 0 to 13 when AGC_CFG2.FE_PERFORMANCE_MODE = 10b and<BR/>
    /// 0 to 7 when AGC_CFG2.FE_PERFORMANCE_MODE = 11b
    pub agc_min_gain, set_agc_min_gain: 4, 0;
}

impl Default for AgcCfg3 {
    fn default() -> Self {
        Self(0xb1)
    }
}

bitfield! {
    /// Automatic Gain Control Configuration Reg. 2
    ///
    /// # Address
    ///
    /// The address of this register is 0x1A
    #[derive(Clone, Copy)]
    pub struct AgcCfg2(u8);

    ///
    /// # Values
    ///
    /// - 0b: Receiver starts with maximum gain value
    /// - 1b: Receiver starts from previous gain value
    ///
    /// The default value is 0x00
    pub start_previous_gain_en, set_start_previous_gain_en: 7;

    /// Controls which gain tables to be applied
    ///
    /// # Values
    ///
    /// - 00b: Optimized linearity mode
    /// - 01b: Normal operation mode
    /// - 10b: Low power mode with reduced gain range
    /// - 11b: Zero-IF mode
    ///
    /// The default value is 0x01
    pub fe_performance_mode, set_fe_performance_mode: 6, 5;

    /// AGC maximum gain. Limits the AGC maximum gain compared to the preset gain table range. AGC_MAX_GAIN can have a value in the range<BR/>
    /// 0 to 17 when AGC_CFG2.FE_PERFORMANCE_MODE = 0 or 1,<BR/>
    /// 0 to 13 when AGC_CFG2.FE_PERFORMANCE_MODE = 10b and<BR/>
    /// 0 to 7 when AGC_CFG2.FE_PERFORMANCE_MODE = 11b
    pub agc_max_gain, set_agc_max_gain: 4, 0;
}

impl Default for AgcCfg2 {
    fn default() -> Self {
        Self(0x20)
    }
}

bitfield! {
    /// Automatic Gain Control Configuration Reg. 1
    ///
    /// # Address
    ///
    /// The address of this register is 0x1B
    #[derive(Clone, Copy)]
    pub struct AgcCfg1(u8);

    pub agc_cfg1_not_used, _: 7;

    /// AGC has a built in function to signal if there has been a step in the RSSI value. During sync search the difference between the current and the previous RSSI value is compared against the RSSI step (3 or 6 dB), while during packet reception, the difference between the current value and the value at sync found is compared against 10 or 16 dB
    ///
    /// # Values
    ///
    /// - 0b: RSSI step is 3 dB during sync search / RSSI step is 10 dB during packet reception
    /// - 1b: RSSI step is 6 dB during sync search / RSSI step is 16 dB during packet reception
    ///
    /// The default value is 0x01
    pub rssi_step_thr, set_rssi_step_thr: 6;

    /// AGC integration window size for each value. Samples refer to the RX filter sampling frequency, which is programmed to be 4 times the desired RX filter BW
    ///
    /// # Values
    ///
    /// - 000b: 8 samples
    /// - 001b: 16 samples
    /// - 010b: 32 samples
    /// - 011b: 64 samples
    /// - 100b: 128 samples
    /// - 101b: 256 samples
    /// - 110b: Reserved
    /// - 111b: Reserved
    ///
    /// The default value is 0x02
    pub agc_win_size, set_agc_win_size: 5, 3;

    /// Sets the wait time between AGC gain adjustments
    ///
    /// # Values
    ///
    /// - 000b: 24 samples
    /// - 001b: 32 samples
    /// - 010b: 40 samples
    /// - 011b: 48 samples
    /// - 100b: 64 samples
    /// - 101b: 80 samples
    /// - 110b: 96 samples
    /// - 111b: 127 samples
    ///
    /// The default value is 0x02
    pub agc_settle_wait, set_agc_settle_wait: 2, 0;
}

impl Default for AgcCfg1 {
    fn default() -> Self {
        Self(0x52)
    }
}

bitfield! {
    /// Automatic Gain Control Configuration Reg. 0
    ///
    /// # Address
    ///
    /// The address of this register is 0x1C
    #[derive(Clone, Copy)]
    pub struct AgcCfg0(u8);

    /// AGC hysteresis level. The difference between the desired signal level and the actual signal level must be larger than AGC hysteresis level before the AGC changes the front end gain
    ///
    /// # Values
    ///
    /// - 00b: 2 dB
    /// - 01b: 4 dB
    /// - 10b: 7 dB
    /// - 11b: 10 dB
    ///
    /// The default value is 0x03
    pub agc_hyst_level, set_agc_hyst_level: 7, 6;

    /// AGC slew rate limit. Limits the maximum front end gain adjustment
    ///
    /// # Values
    ///
    /// - 00b: 60 dB
    /// - 01b: 30 dB
    /// - 10b: 18 dB
    /// - 11b: 9 dB
    ///
    /// The default value is 0x00
    pub agc_slewrate_limit, set_agc_slewrate_limit: 5, 4;

    /// Gives the number of new input samples to the moving average filter (internal RSSI estimates) that are required before the next update of the RSSI value. The RSSI_VALID signal will be asserted from the first RSSI update. RSSI_VALID is available on a GPIO or can be read from the RSSI0 register
    ///
    /// # Values
    ///
    /// - 00b: 1
    /// - 01b: 2
    /// - 10b: 5
    /// - 11b: 9
    ///
    /// The default value is 0x00
    pub rssi_valid_cnt, set_rssi_valid_cnt: 3, 2;

    /// The OOK/ASK receiver uses a max peak magnitude (logic 1) tracker and low peak magnitude (logic 0) tracker to estimate ASK_THRESHOLD (decision level) as the average of the max and min value. The max peak magnitude value is also used by the AGC to set the gain. AGC_ASK_DECAY controls the max peak magnitude decay steps in OOK/ASK mode and defines the number of samples required for the max peak level to be reduced to 10% when receiving logic 0 after receiving a logic 1.
    /// <BR/> <BR/>
    /// Sample Rate = (f_xosc*Interpolation Factor)/(Decimation Factor*CHAN_BW.BB_CIC_DECFACT)[Hz]
    /// <BR/> <BR/>
    /// The decimation factor is given by CHAN_BW.ADC_CIC_DECFACT and the interpolation factor is given by SYNC_CFG0.RX_CONFIG_LIMITATION as follows:</br>
    ///
    /// # Values
    ///
    /// - 00b: 1200 samples
    /// - 01b: 2400 samples
    /// - 10b: 4700 samples
    /// - 11b: 9500 samples
    ///
    /// The default value is 0x03
    pub agc_ask_decay, set_agc_ask_decay: 1, 0;
}

impl Default for AgcCfg0 {
    fn default() -> Self {
        Self(0xc3)
    }
}

bitfield! {
    /// FIFO Configuration
    ///
    /// # Address
    ///
    /// The address of this register is 0x1D
    #[derive(Clone, Copy)]
    pub struct FifoCfg(u8);

    /// Automatically flushes the last packet received in the RX FIFO if a CRC error occurred. If this bit has been turned off and should be turned on again, an SFRX strobe must first be issued
    pub crc_autoflush, set_crc_autoflush: 7;

    /// Threshold value for the RX and TX FIFO. The threshold value is coded in opposite directions for the two FIFOs to give equal margin to the overflow and underflow conditions when the threshold is reached. I.e.; FIFO_THR = 0 means that there are 127 bytes in the TX FIFO and 1 byte in the RX FIFO, while FIFO_THR = 127 means that there are 0 bytes in the TX FIFO and 128 bytes in the RX FIFO when the thresholds are reached
    pub fifo_thr, set_fifo_thr: 6, 0;
}

impl Default for FifoCfg {
    fn default() -> Self {
        Self(0x80)
    }
}

bitfield! {
    /// Device Address Configuration
    ///
    /// # Address
    ///
    /// The address of this register is 0x1E
    #[derive(Clone, Copy)]
    pub struct DevAddr(u8);

    /// Address used for packet filtering in RX
    pub device_addr, set_device_addr: 7, 0;
}

impl Default for DevAddr {
    fn default() -> Self {
        Self(0x00)
    }
}

bitfield! {
    /// Frequency Synthesizer Calibration and Settling Configuration
    ///
    /// # Address
    ///
    /// The address of this register is 0x1F
    #[derive(Clone, Copy)]
    pub struct SettlingCfg(u8);

    pub settling_cfg_not_used, _: 7, 5;

    /// Auto calibration is performed:
    ///
    /// # Values
    ///
    /// - 00b: Never (manually calibrate using SCAL strobe)
    /// - 01b: When going from IDLE to RX or TX (or FSTXON)
    /// - 10b: When going from RX or TX back to IDLE automatically
    /// - 11b: Every 4th time when going from RX or TX to IDLE automatically
    ///
    /// The default value is 0x01
    pub fs_autocal, set_fs_autocal: 4, 3;

    /// Sets the time for the frequency synthesizer to settle to lock state. The table shows settling after calibration and settling when switching between TX and RX. Use values from SmartRF Studio
    ///
    /// # Values
    ///
    /// - 00b: 50 / 20 us
    /// - 01b: 75 / 30 us
    /// - 10b: 100 / 40 us
    /// - 11b: 150 / 60 us
    ///
    /// The default value is 0x01
    pub lock_time, set_lock_time: 2, 1;

    /// Frequency synthesizer regulator settling time. Use values from SmartRF Studio
    ///
    /// # Values
    ///
    /// - 0b: 30 us
    /// - 1b: 60 us
    ///
    /// The default value is 0x01
    pub fsreg_time, set_fsreg_time: 0;
}

impl Default for SettlingCfg {
    fn default() -> Self {
        Self(0x0b)
    }
}

bitfield! {
    /// Frequency Synthesizer Configuration
    ///
    /// # Address
    ///
    /// The address of this register is 0x20
    #[derive(Clone, Copy)]
    pub struct FsCfg(u8);

    pub fs_cfg_not_used, _: 7, 5;

    /// Out of lock detector enable
    ///
    /// # Values
    ///
    /// - 0b: Out of lock detector disabled
    /// - 1b: Out of lock detector enabled
    ///
    /// The default value is 0x00
    pub fs_lock_en, set_fs_lock_en: 4;

    /// Band select setting for LO divider
    ///
    /// # Values
    ///
    /// - 0000b: Not in use
    /// - 0001b: Not in use
    /// - 0010b: 820.0 - 960.0 MHz band (LO divider = 4)
    /// - 0011b: Not in use
    /// - 0100b: 410.0 - 480.0 MHz band (LO divider = 8)
    /// - 0101b: Not in use
    /// - 0110b: 273.3 - 320.0 MHz band (LO divider = 12)
    /// - 0111b: Not in use
    /// - 1000b: 205.0 - 240.0 MHz band (LO divider = 16)
    /// - 1001b: Not in use
    /// - 1010b: 164.0 - 192.0 MHz band (LO divider = 20)
    /// - 1011b: 136.7 - 160.0 MHz band (LO divider = 24)
    /// - 1100b: Not in use
    /// - 1101b: Not in use
    /// - 1110b: Not in use
    /// - 1111b: Not in use
    ///
    /// The default value is 0x02
    pub fsd_bandselect, set_fsd_bandselect: 3, 0;
}

impl Default for FsCfg {
    fn default() -> Self {
        Self(0x02)
    }
}

bitfield! {
    /// eWOR Configuration Reg. 1
    ///
    /// # Address
    ///
    /// The address of this register is 0x21
    #[derive(Clone, Copy)]
    pub struct WorCfg1(u8);

    /// eWOR timer resolution. Controls the t_Event0 and RX timeout resolution<BR/>
    /// t_EVENT0 =  2^(5*WOR_RES)*EVENT0/f_rcosc [s] and<BR/>
    /// RX Timeout = MAX[1,FLOOR[EVENT0/2^(RFEND_CFG1.RX_TIME+3)]]*2^(4*WOR_RES)*1250/f_xosc [s]
    ///
    /// # Values
    ///
    /// - 00b: High resolution
    /// - 01b: Medium high resolution
    /// - 10b: Medium low resolution
    /// - 11b: Low resolution
    ///
    /// The default value is 0x00
    pub wor_res, set_wor_res: 7, 6;

    /// eWOR mode
    ///
    /// # Values
    ///
    /// - 000b: Feedback mode
    /// - 001b: Normal mode
    /// - 010b: Legacy mode
    /// - 011b: Event1 mask mode
    /// - 100b: Event0 mask mode
    /// - 111b: Reserved
    ///
    /// The default value is 0x01
    pub wor_mode, set_wor_mode: 5, 3;

    /// Event 1 timeout<BR/>
    /// t_EVENT1 = WOR_EVENT1/f_rcosc [s]
    /// </br>
    pub event1, set_event1: 2, 0;
}

impl Default for WorCfg1 {
    fn default() -> Self {
        Self(0x08)
    }
}

bitfield! {
    /// eWOR Configuration Reg. 0
    ///
    /// # Address
    ///
    /// The address of this register is 0x22
    #[derive(Clone, Copy)]
    pub struct WorCfg0(u8);

    /// RX duty cycle mode configuration. eWOR mode and RXDCM cannot be enabled at the same time. Both modes can be used in RX Sniff Mode implementation
    ///
    /// # Values
    ///
    /// - 00b: RXDCM disabled
    /// - 01b: RXDCM 0
    /// - 10b: RXDCM 1
    /// - 11b: RXDCM 2
    ///
    /// The default value is 0x00
    pub rx_duty_cycle_mode, _: 7, 6;

    /// Clock division enable. Enables clock division in SLEEP mode
    /// Setting DIV_256HZ_EN = 1 will lower the current consumption in SLEEP mode. Note that when this bit is set the radio should not be woken from SLEEP by pulling CSn low
    ///
    /// # Values
    ///
    /// - 0b: Clock division disabled
    /// - 1b: Clock division enabled
    ///
    /// The default value is 0x01
    pub div_256hz_en, set_div_256hz_en: 5;

    /// Event 2 timeout<BR/>
    /// t_EVENT2 = 2^WOR_EVENT2/f_rcosc [s]
    pub event2_cfg, set_event2_cfg: 4, 3;

    /// RCOSC calibration mode. Configures when the RCOSC calibration sequence is performed. If calibration is enabled, WOR_CFG0.RC_PD must be 0
    ///
    /// # Values
    ///
    /// - 00b: RCOSC calibration disabled
    /// - 01b: RCOSC calibration disabled
    /// - 10b: RCOSC calibration enabled
    /// - 11b: RCOSC calibration is enabled on every 4th time the device is powered up and goes from IDLE to RX. This setting should only be used together with eWOR
    ///
    /// The default value is 0x00
    pub rc_mode, set_rc_mode: 2, 1;

    /// RCOSC power down signal
    ///
    /// # Values
    ///
    /// - 0b: RCOSC is running
    /// - 1b: RCOSC is in power down
    ///
    /// The default value is 0x01
    pub rc_pd, set_rc_pd: 0;
}

impl Default for WorCfg0 {
    fn default() -> Self {
        Self(0x21)
    }
}

bitfield! {
    /// Event 0 Configuration MSB
    ///
    /// # Address
    ///
    /// The address of this register is 0x23
    #[derive(Clone, Copy)]
    pub struct WorEvent0Msb(u8);

    /// Event 0 timeout (MSB)<BR/>
    /// t_EVENT0 = 2^(5*WOR_CFG1.WOR_RES)*EVENT0/f_rcosc [s]
    pub event0_15_8, set_event0_15_8: 7, 0;
}

impl Default for WorEvent0Msb {
    fn default() -> Self {
        Self(0x00)
    }
}

bitfield! {
    /// Event 0 Configuration LSB
    ///
    /// # Address
    ///
    /// The address of this register is 0x24
    #[derive(Clone, Copy)]
    pub struct WorEvent0Lsb(u8);

    /// Event 0 timeout (LSB). See WOR_EVENT0_MSB
    pub event0_7_0, set_event0_7_0: 7, 0;
}

impl Default for WorEvent0Lsb {
    fn default() -> Self {
        Self(0x00)
    }
}

bitfield! {
    /// RX Duty Cycle Mode Configuration
    ///
    /// # Address
    ///
    /// The address of this register is 0x25
    #[derive(Clone, Copy)]
    pub struct RxdcmTime(u8);

    /// Configures the time spent in RXDCM state</br>
    /// RX_DUTY_CYCLE_TIME = 0:</br>
    /// t_RXDCM = 2^WOR_CFG1.WOR_RES[us]</br>
    /// RX_DUTY_CYCLE_TIME != 0:</br>
    /// t_RXDCM = RX_DUTY_CYCLE_TIME*2^WOR_CFG1.WOR_RES[us]
    pub rx_duty_cycle_time, set_rx_duty_cycle_time: 7, 0;
}

impl Default for RxdcmTime {
    fn default() -> Self {
        Self(0x00)
    }
}

bitfield! {
    /// Packet Configuration Reg. 2
    ///
    /// # Address
    ///
    /// The address of this register is 0x26
    #[derive(Clone, Copy)]
    pub struct PktCfg2(u8);

    pub pkt_cfg2_not_used, _: 7;

    /// TX/RX data byte swap enable. In RX, all bits in the received data byte are swapped before written to the RX FIFO. In TX, all bits in the TX FIFO data byte are swapped before being transmitted
    ///
    /// # Values
    ///
    /// - 0b: Data byte swap disabled
    /// - 1b: Data byte swap enabled
    ///
    /// The default value is 0x00
    pub byte_swap_en, set_byte_swap_en: 6;

    /// Select between standard packet mode or 802.15.4g packet mode
    ///
    /// # Values
    ///
    /// - 0b: Standard packet mode enabled
    /// - 1b: 802.15.4g packet mode enabled (will override other packet engine configuration settings)
    ///
    /// The default value is 0x00
    pub fg_mode_en, set_fg_mode_en: 5;

    /// CCA mode. Selects the definition of a clear channel (when to assert the CCA signal)
    ///
    /// # Values
    ///
    /// - 000b: Always give a clear channel indication
    /// - 001b: Indicates clear channel when RSSI is below threshold
    /// - 010b: Indicates clear channel unless currently receiving a packet
    /// - 011b: Indicates clear channel when RSSI is below threshold and currently not receiving a packet
    /// - 100b: Indicates clear channel when RSSI is below threshold and ETSI LBT requirements are met
    /// - 101b: Reserved
    /// - 110b: Reserved
    /// - 111b: Reserved
    ///
    /// The default value is 0x01
    pub cca_mode, set_cca_mode: 4, 2;

    /// Packet format configuration
    ///
    /// # Values
    ///
    /// - 00b: Normal mode / FIFO mode (MDMCFG1.FIFO_EN must be set to 1 and MDMCFG0.TRANSPARENT_MODE_EN must be set to 0)
    /// - 01b: Synchronous serial mode (MDMCFG1.FIFO_EN must be set to 0 and MDMCFG0.TRANSPARENT_MODE_EN must be set to 0). This mode is only supported for 2┬Æary modulations formats in TX. In RX, both 2'ary and 4┬Æary modulation formats are supported
    /// - 10b: Random mode. Send random data using PN9 generator (Set TXLAST != TXFIRST before strobing STX)
    /// - 11b: Transparent serial mode (MDMCFG1.FIFO_EN must be set to 0 and MDMCFG0.TRANSPARENT_MODE_EN must be set to 1). This mode is only supported for 2┬Æary modulations formats
    ///
    /// The default value is 0x00
    pub pkt_format, set_pkt_format: 1, 0;
}

impl Default for PktCfg2 {
    fn default() -> Self {
        Self(0x04)
    }
}

bitfield! {
    /// Packet Configuration Reg. 1
    ///
    /// # Address
    ///
    /// The address of this register is 0x27
    #[derive(Clone, Copy)]
    pub struct PktCfg1(u8);

    /// Forward error correction enable
    ///
    /// # Values
    ///
    /// - 0b: FEC disabled
    /// - 1b: FEC enabled
    ///
    /// The default value is 0x00
    pub fec_en, set_fec_en: 7;

    /// Whitening enable
    ///
    /// # Values
    ///
    /// - 0b: Data whitening disabled
    /// - 1b: Data whitening enabled
    ///
    /// The default value is 0x00
    pub white_data, set_white_data: 6;

    /// PN9 sequence swap enable Determines if the PN9 sequence is swapped prior to whitening/de-whitening. This settings is only used when WHITE_DATA = 1 and PKT_CFG2.FG_MODE_EN = 0
    ///
    /// # Values
    ///
    /// - 0b: PN9 sequence swap disabled
    /// - 1b: PN9 sequence swap enabled
    ///
    /// The default value is 0x00
    pub pn9_swap_en, set_pn9_swap_en: 5;

    /// Address check configuration. Controls how address check is performed in RX mode
    ///
    /// # Values
    ///
    /// - 00b: No address check
    /// - 01b: Address check, no broadcast
    /// - 10b: Address check, 0x00 broadcast
    /// - 11b: Address check, 0x00 and 0xFF broadcast
    ///
    /// The default value is 0x00
    pub addr_check_cfg, set_addr_check_cfg: 4, 3;

    /// CRC configuration
    ///
    /// # Values
    ///
    /// - 00b: CRC disabled for TX and RX
    /// - 01b: CRC calculation in TX mode and CRC check in RX mode enabled. CRC16(X16+X15+X2+1). Initialized to 0xFFFF
    /// - 10b: CRC calculation in TX mode and CRC check in RX mode enabled. CRC16(X16+X12+X5+1). Initialized to 0x0000
    /// - 11b: CRC calculation in TX mode and CRC check in RX mode enabled. 1's complement of CRC16(X16+X12+X5+1). Initialized to 0x1D0F
    ///
    /// The default value is 0x01
    pub crc_cfg, set_crc_cfg: 2, 1;

    /// Append status bytes to RX FIFO. The status bytes contain info about CRC, RSSI, and LQI. When PKT_CFG1.CRC_CFG = 0, the CRC_OK field in the status byte will be 0
    ///
    /// # Values
    ///
    /// - 0b: Status byte not appended
    /// - 1b: Status byte appended
    ///
    /// The default value is 0x01
    pub append_status, set_append_status: 0;
}

impl Default for PktCfg1 {
    fn default() -> Self {
        Self(0x03)
    }
}

bitfield! {
    /// Packet Configuration Reg. 0
    ///
    /// # Address
    ///
    /// The address of this register is 0x28
    #[derive(Clone, Copy)]
    pub struct PktCfg0(u8);

    /// For test purposes only, use values from SmartRF Studio.
    pub pkt_cfg0_reserved7, set_pkt_cfg0_reserved7: 7;

    /// Packet length configuration
    ///
    /// # Values
    ///
    /// - 00b: Fixed packet length mode. Packet Length configured through the PKT_LEN register
    /// - 01b: Variable packet length mode. Packet length configured by the first byte received after sync word
    /// - 10b: Infinite packet length mode
    /// - 11b: Variable packet length mode. Length configured by the 5 LSB of the first byte received after sync word
    ///
    /// The default value is 0x00
    pub length_config, set_length_config: 6, 5;

    /// In fixed packet length mode this field (when not zero) indicates the number of bits to send/receive after PKT_LEN number of bytes are sent/received. CRC is not supported when PKT_LEN_BIT != 0
    pub pkt_bit_len, set_pkt_bit_len: 4, 2;

    /// UART mode enable. When enabled, the packet engine will insert/remove a start and stop bit to/from the transmitted/received bytes
    ///
    /// # Values
    ///
    /// - 0b: UART mode disabled
    /// - 1b: UART mode enabled
    ///
    /// The default value is 0x00
    pub uart_mode_en, set_uart_mode_en: 1;

    /// Swap start and stop bits values
    ///
    /// # Values
    ///
    /// - 0b: Swap disabled. Start/stop bits values are '1'/'0'
    /// - 1b: Swap enabled. Start/stop bits values are '0'/'1'
    ///
    /// The default value is 0x00
    pub uart_swap_en, set_uart_swap_en: 0;
}

impl Default for PktCfg0 {
    fn default() -> Self {
        Self(0x00)
    }
}

bitfield! {
    /// RFEND Configuration Reg. 1
    ///
    /// # Address
    ///
    /// The address of this register is 0x29
    #[derive(Clone, Copy)]
    pub struct RfendCfg1(u8);

    pub rfend_cfg1_not_used, _: 7, 6;

    /// RXOFF mode. Determines the state the radio will enter after receiving a good packet
    ///
    /// # Values
    ///
    /// - 00b: IDLE
    /// - 01b: FSTXON
    /// - 10b: TX
    /// - 11b: RX
    ///
    /// The default value is 0x00
    pub rxoff_mode, set_rxoff_mode: 5, 4;

    /// RX timeout for sync word search in RX<BR/>
    /// RX Timeout = MAX[1,FLOOR[EVENT0/2^(RX_TIME+3)]]*2^(4*WOR_CFG1.WOR_RES)*1250/f_xosc [s]<BR/>
    /// The RX timeout is disabled when RX_TIME = 111b. EVENT0 is found in the WOR_EVENT0_MSB and WOR_EVENT0_LSB registers
    pub rx_time, set_rx_time: 3, 1;

    /// RX timeout qualifier
    ///
    /// # Values
    ///
    /// - 0b: Continue RX mode on RX timeout if sync word is found
    /// - 1b: Continue RX mode on RX timeout if sync word has been found, or if PQT is reached or CS is asserted
    ///
    /// The default value is 0x01
    pub rx_time_qual, set_rx_time_qual: 0;
}

impl Default for RfendCfg1 {
    fn default() -> Self {
        Self(0x0f)
    }
}

bitfield! {
    /// RFEND Configuration Reg. 0
    ///
    /// # Address
    ///
    /// The address of this register is 0x2A
    #[derive(Clone, Copy)]
    pub struct RfendCfg0(u8);

    pub rfend_cfg0_not_used, _: 7;

    /// Enable additional wake-up pulses on the end of calibration. To be used together with the MCU_WAKEUP signal (MARC_STATUS_OUT will be 0)
    ///
    /// # Values
    ///
    /// - 0b: Disable additional wake-up pulse
    /// - 1b: Enable additional wake-up pulse
    ///
    /// The default value is 0x00
    pub cal_end_wake_up_en, set_cal_end_wake_up_en: 6;

    /// TXOFF mode. Determines the state the radio will enter after transmitting a packet
    ///
    /// # Values
    ///
    /// - 00b: IDLE
    /// - 01b: FSTXON
    /// - 10b: TX
    /// - 11b: RX
    ///
    /// The default value is 0x00
    pub txoff_mode, set_txoff_mode: 5, 4;

    /// Terminate on bad packet enable
    ///
    /// # Values
    ///
    /// - 0b: Terminate on bad packet disabled. When a bad packet is received (address, length, or CRC error) the radio stays in RX regardless of the RFEND_CFG1.RXOFF_MODE
    /// - 1b: Terminate on bad packet enabled. RFEND_CFG1.RXOFF_MODE is ignored and the radio enters IDLE mode (or SLEEP mode if eWOR is used) when a bad packet has been received
    ///
    /// The default value is 0x00
    pub term_on_bad_packet_en, set_term_on_bad_packet_en: 3;

    /// Direct RX termination and antenna diversity configuration
    ///
    /// # Values
    ///
    /// - 000b: Antenna diversity and termination based on CS/PQT are disabled
    /// - 001b: RX termination based on CS is enabled (Antenna diversity OFF)
    /// - 010b: Single-switch antenna diversity on CS enabled. One or both antenna is CS evaluated once and RX will terminate if CS failed on both antennas
    /// - 011b: Continuous-switch antenna diversity on CS enabled. Antennas are switched until CS is asserted or RX timeout occurs (if RX timeout is enabled)
    /// - 100b: RX termination based on PQT is enabled (Antenna diversity OFF). <BR> MDMCFG1.CARRIER_SENSE_GATE must be 0 when this feature is used.
    /// - 101b: Single-switch antenna diversity on PQT enabled. One or both antennas are PQT evaluated once and RX will terminate if PQT is not reached on any of the antennas. <BR> MDMCFG1.CARRIER_SENSE_GATE must be 0 when this feature is used.
    /// - 110b: Continuous-switch antenna diversity on PQT enabled. Antennas are switched until PQT is reached or RX timeout occurs (if RX timeout is enabled). <BR> MDMCFG1.CARRIER_SENSE_GATE must be 0 when this feature is used.
    /// - 111b: Reserved
    ///
    /// The default value is 0x00
    pub ant_div_rx_term_cfg, set_ant_div_rx_term_cfg: 2, 0;
}

impl Default for RfendCfg0 {
    fn default() -> Self {
        Self(0x00)
    }
}

bitfield! {
    /// Power Amplifier Configuration Reg. 1
    ///
    /// # Address
    ///
    /// The address of this register is 0x2B
    #[derive(Clone, Copy)]
    pub struct PaCfg1(u8);

    pub pa_cfg2_not_used, _: 7;

    /// PA ramping and ASK/OOK shaping enable
    ///
    /// # Values
    ///
    /// - 0b: PA ramping and ASK/OOK shaping disabled
    /// - 1b: PA ramping and ASK/OOK shaping enabled
    ///
    /// The default value is 0x01
    pub pa_ramp_shape_en, set_pa_ramp_shape_en: 6;

    /// PA power ramp target level<BR/>
    /// Output Power = (PA_POWER_RAMP+1)/2-18 [dBm]<BR/>
    /// PA_POWER_RAMP >= 0x03 for the equation to be valid. {0x00, 0x01, 0x02} are special power levels
    pub pa_power_ramp, set_pa_power_ramp: 5, 0;
}

impl Default for PaCfg1 {
    fn default() -> Self {
        Self(0x7f)
    }
}

bitfield! {
    /// Power Amplifier Configuration Reg. 0
    ///
    /// # Address
    ///
    /// The address of this register is 0x2C
    #[derive(Clone, Copy)]
    pub struct PaCfg0(u8);

    /// First intermediate power level. The first intermediate power level can be programmed within the power level range 0 - 7/16 in steps of 1/16
    pub first_ipl, set_first_ipl: 7, 5;

    /// Second intermediate power level. The second intermediate power level can be programmed within the power level range 8/16 - 15/16 in steps of 1/16
    pub second_ipl, set_second_ipl: 4, 2;

    /// PA ramp time and ASK/OOK shape length. Note that only certain values of PA_CFG0.UPSAMPLER_P complies with the different ASK/OOK shape lengths
    ///
    /// # Values
    ///
    /// - 00b: 3/8 symbol ramp time and 1/32 symbol ASK/OOK shape length (legal UPSAMPLER_P values: 100b, 101b, and 110b)
    /// - 01b: 3/2 symbol ramp time and 1/16 symbol ASK/OOK shape length (legal UPSAMPLER_P values: 011b, 100b, 101b, and 110b)
    /// - 10b: 3 symbol ramp time and 1/8 symbol ASK/OOK shape length (legal UPSAMPLER_P values: 010b, 011b, 100b, 101b, and 110b)
    /// - 11b: 6 symbol ramp time and 1/4 symbol ASK/OOK shape length (legal UPSAMPLER_P values: 010b , 010b, 011b, 100b, 101b, and 110b)
    ///
    /// The default value is 0x02
    pub ramp_shape, set_ramp_shape: 1, 0;
}

impl Default for PaCfg0 {
    fn default() -> Self {
        Self(0x56)
    }
}

bitfield! {
    /// ASK Configuration
    ///
    /// # Address
    ///
    /// The address of this register is 0x2D
    #[derive(Clone, Copy)]
    pub struct AskCfg(u8);

    /// Controls the bandwidth of the data filter in ASK/OOK mode. The -3 dB cut-off frequency (fCut-Off) is given below:</br>
    /// RX_CONFIG_LIMITATION = 0:<BR/>
    /// f-Cut-Off = 4*ASK BW Scale Factor*Rx Filter BW [Hz]<BR/>
    /// RX_CONFIG_LIMITATION = 1:<BR/>
    /// f-Cut-Off = 8*ASK BW Scale Factor*Rx Filter BW [Hz]<BR/>
    /// RX_CONFIG_LIMITATION is found in SYNC_CFG0. A rule of thumb is to set f_Cut-Off >= 5*symbol rate
    ///
    /// # Values
    ///
    /// - 00b: ASK BW scale factor = 0.28
    /// - 01b: ASK BW scale factor = 0.18
    /// - 10b: ASK BW scale factor = 0.15
    /// - 11b: ASK BW scale factor = 0.14
    ///
    /// The default value is 0x00
    pub agc_ask_bw, set_agc_ask_bw: 7, 6;

    /// ASK/OOK depth<BR/>
    /// A_Max = (PA_CFG1.PA_POWER_RAMP+1)/2-18 [dBm]<BR/>
    /// A_Min = (PA_CFG1.PA_POWER_RAMP+1-ASK_DEPTH)/2-18 [dBm]<BR/>
    /// Minimum PA power level is -16 dBm. PA_POWER_RAMP - ASK_DEPTH = 0x00 is OOK off state (< -50 dBm)
    pub ask_depth, set_ask_depth: 5, 0;
}

impl Default for AskCfg {
    fn default() -> Self {
        Self(0x0f)
    }
}

bitfield! {
    /// Packet Length Configuration
    ///
    /// # Address
    ///
    /// The address of this register is 0x2E
    #[derive(Clone, Copy)]
    pub struct PktLen(u8);

    /// In fixed length mode this field indicates the packet length, and a value of 0 indicates the length to be 256 bytes. In variable length packet mode, this value indicates the maximum allowed length packets
    pub packet_length, set_packet_length: 7, 0;
}

impl Default for PktLen {
    fn default() -> Self {
        Self(0x03)
    }
}

bitfield! {
    /// IF Mix Configuration
    ///
    /// # Address
    ///
    /// The address of this register is 0x2F00
    #[derive(Clone, Copy)]
    pub struct IfMixCfg(u8);

    pub if_mix_cfg_not_used, _: 7, 5;

    /// Intermediate frequency configuration. The decimation factor is given by CHAN_BW.ADC_CIC_DECFACT
    ///
    /// # Values
    ///
    /// - 000b: Zero-IF
    /// - 001b: f_IF = -f_xosc/(Decimation Factor*4)[kHz]
    /// - 010b: f_IF = -f_xosc/(Decimation Factor*6)[kHz]
    /// - 011b: f_IF = -f_xosc/(Decimation Factor*8)[kHz]
    /// - 100b: Zero-IF
    /// - 101b: f_IF = f_xosc/(Decimation Factor*4)[kHz]
    /// - 110b: f_IF = f_xosc/(Decimation Factor*6)[kHz]
    /// - 111b: f_IF = f_xosc/(Decimation Factor*8)[kHz]
    ///
    /// The default value is 0x00
    pub cmix_cfg, set_cmix_cfg: 4, 2;

    /// For test purposes only, use values from SmartRF Studio.
    pub if_mix_cfg_reserved1, set_if_mix_cfg_reserved1: 1;

    /// For test purposes only, use values from SmartRF Studio.
    pub if_mix_cfg_reserved0, set_if_mix_cfg_reserved0: 0;
}

impl Default for IfMixCfg {
    fn default() -> Self {
        Self(0x00)
    }
}

bitfield! {
    /// Frequency Offset Correction Configuration
    ///
    /// # Address
    ///
    /// The address of this register is 0x2F01
    #[derive(Clone, Copy)]
    pub struct FreqoffCfg(u8);

    pub freqoff_cfg_not_used, _: 7, 6;

    /// Frequency offset correction enable
    ///
    /// # Values
    ///
    /// - 0b: Frequency offset correction disabled
    /// - 1b: Frequency offset correction enabled
    ///
    /// The default value is 0x01
    pub foc_en, set_foc_en: 5;

    /// Frequency offset correction configuration. FOC_CFG != 00b enables a narrower RX filter BW than FOC_CFG = 00b but needs longer settle time. When FOC in FS is enabled, the device automatically switch to 'FOC after channel filter' when a sync word is detected.
    ///
    /// # Values
    ///
    /// - 00b: FOC after channel filter (typical 0 - 1 preamble bytes for settling)
    /// - 01b: FOC in FS enabled. Loop gain factor is 1/128 (typical 2 - 4 preamble bytes for settling)
    /// - 10b: FOC in FS enabled. Loop gain factor is 1/256 (typical 2 - 4 preamble bytes for settling)
    /// - 11b: FOC in FS enabled. Loop gain factor is 1/512 (typical 2 - 4 preamble bytes for settling)
    ///
    /// The default value is 0x00
    pub foc_cfg, set_foc_cfg: 4, 3;

    /// FOC limit. This is the maximum frequency offset correction in the frequency synthesizer. Only valid when FOC_CFG != 00b
    ///
    /// # Values
    ///
    /// - 0b: RX filter bandwidth/4
    /// - 1b: RX filter bandwidth/8
    ///
    /// The default value is 0x00
    pub foc_limit, set_foc_limit: 2;

    /// Frequency offset correction<BR/>
    /// MDMCFG0.TRANSPARENT_MODE_EN | FOC_KI_FACTOR
    ///
    /// # Values
    ///
    /// - 000b: Frequency offset compensation disabled after sync detected (typical setting for short packets)
    /// - 001b: Frequency offset compensation during packet reception with loop gain factor = 1/32 (fast loop)
    /// - 010b: Frequency offset compensation during packet reception with loop gain factor = 1/64
    /// - 011b: Frequency offset compensation during packet reception with loop gain factor = 1/128 (slow loop)
    /// - 100b: Frequency offset compensation with Loop Gain factor 1/128 (fast loop)
    /// - 101b: Frequency offset compensation with Loop Gain factor 1/256
    /// - 110b: Frequency offset compensation with Loop Gain factor 1/512
    /// - 111b: Frequency offset compensation with Loop Gain factor 1/1024 (slow loop)
    ///
    /// The default value is 0x00
    pub foc_ki_factor, set_foc_ki_factor: 1, 0;
}

impl Default for FreqoffCfg {
    fn default() -> Self {
        Self(0x20)
    }
}

bitfield! {
    /// Timing Offset Correction Configuration
    ///
    /// # Address
    ///
    /// The address of this register is 0x2F02
    #[derive(Clone, Copy)]
    pub struct TocCfg(u8);

    /// Timing offset correction limit. TOC_LIMIT specifies maximum symbol rate offset the receiver is able to handle. TOC_LIMIT != 00b requires 2 - 4 bytes preamble for symbol rate offset compensation
    ///
    /// # Values
    ///
    /// - 00b: < 0.2 %
    /// - 01b: < 2 %
    /// - 10b: Reserved
    /// - 11b: < 12 % (MDMCFG1.CARRIER_SENSE_GATE must be set)
    ///
    /// The default value is 0x00
    pub toc_limit, set_toc_limit: 7, 6;

    /// When TOC_LIMIT = 0 the receiver uses a block based time offset error calculation algorithm where the block length is configurable through register TOC_CFG. Before a sync word is found (SYNC_EVENT is asserted) the TOC_PRE_SYNC_BLOCKLEN sets the actual block length used for the time offset algorithm<BR/>
    ///
    /// # Values
    ///
    /// - 0b: Symbol by Symbol Timing Error Proportional Scale Factor
    ///
    /// The default value is 0x01
    pub toc_pre_sync_blocklen, set_toc_pre_sync_blocklen: 5, 3;

    /// When TOC_LIMIT = 0 the receiver uses a block based time offset error calculation algorithm where the block length is configurable through register TOC_CFG. After a sync word is found (SYNC_EVENT is asserted) the TOC_POST_SYNC_BLOCKLEN sets the actual block length used for the time offset algorithm<BR/>
    ///
    /// # Values
    ///
    /// - 0b: Symbol by Symbol Timing Error Integral Scale Factor
    ///
    /// The default value is 0x03
    pub toc_post_sync_blocklen, set_toc_post_sync_blocklen: 2, 0;
}

impl Default for TocCfg {
    fn default() -> Self {
        Self(0x0b)
    }
}

bitfield! {
    /// MARC Spare
    ///
    /// # Address
    ///
    /// The address of this register is 0x2F03
    #[derive(Clone, Copy)]
    pub struct MarcSpare(u8);

    pub marc_spare_not_used, _: 7, 4;

    /// High level commands used to accelerate AES operations on the FIFO content
    ///
    /// # Values
    ///
    /// - 1000b: Reserved
    /// - 1001b: AES_TXFIFO
    /// - 1010b: AES_RXFIFO
    /// - 1111b: Reserved
    ///
    /// The default value is 0x00
    pub aes_commands, set_aes_commands: 3, 0;
}

impl Default for MarcSpare {
    fn default() -> Self {
        Self(0x00)
    }
}

bitfield! {
    /// External Clock Frequency Configuration
    ///
    /// # Address
    ///
    /// The address of this register is 0x2F04
    #[derive(Clone, Copy)]
    pub struct EcgCfg(u8);

    pub ecg_cfg_not_used, _: 7, 5;

    /// External clock frequency. Controls division factor
    ///
    /// # Values
    ///
    /// - 00000b: 64
    /// - 00001b: 62
    /// - 00010b: 60
    /// - 00011b: 58
    /// - 00100b: 56
    /// - 00101b: 54
    /// - 00110b: 52
    /// - 00111b: 50
    /// - 01000b: 48
    /// - 01001b: 46
    /// - 01010b: 44
    /// - 01011b: 42
    /// - 01100b: 40
    /// - 01101b: 38
    /// - 01110b: 36
    /// - 01111b: 34
    /// - 10000b: 32
    /// - 10001b: 30
    /// - 10010b: 28
    /// - 10011b: 26
    /// - 10100b: 24
    /// - 10101b: 22
    /// - 10110b: 20
    /// - 10111b: 18
    /// - 11000b: 16
    /// - 11001b: 14
    /// - 11010b: 12
    /// - 11011b: 10
    /// - 11100b: 8
    /// - 11101b: 6
    /// - 11110b: 4
    /// - 11111b: 3
    ///
    /// The default value is 0x00
    pub ext_clock_freq, set_ext_clock_freq: 4, 0;
}

impl Default for EcgCfg {
    fn default() -> Self {
        Self(0x00)
    }
}

bitfield! {
    /// General Modem Parameter Configuration Reg. 2
    ///
    /// # Address
    ///
    /// The address of this register is 0x2F05
    #[derive(Clone, Copy)]
    pub struct Mdmcfg2(u8);

    /// Sets the resolution of an ASK bit transition (# of points). The following rule must be satisfied:<BR/>
    ///
    /// # Values
    ///
    /// - 3b: MDMCFG2.UPSAMPLER_P = 4 - PA_CFG0.RAMP_SHAPE + ASK_SHAPE<BR/>
    /// - 3b: MDMCFG2.UPSAMPLER_P = 5 - PA_CFG0.RAMP_SHAPE + ASK_SHAPE
    /// - 00b: 8
    /// - 01b: 16
    /// - 10b: 32
    /// - 11b: 128
    ///
    /// The default value is 0x00
    pub ask_shape, set_ask_shape: 7, 6;

    /// Symbol map configuration. Configures the modulated symbol mapping definition from data bit to modulated symbols. For 2'ary modulation schemes the symbol mapping definition is as follows:
    pub symbol_map_cfg, set_symbol_map_cfg: 5, 4;

    /// UPSAMPLER_P configures the variable upsampling factor P for the TX upsampler. The total upsampling factor = 16*P. The upsampler factor P must satisfy the following:<BR/>
    /// Symbol Rate*16*P < f_xosc/4, , where P should be as large as possible<BR/>
    /// The upsampler reduces repetitive spectrum at 16*symbol rate
    ///
    /// # Values
    ///
    /// - 000b: TX upsampler factor P = 1 (bypassed)
    /// - 001b: TX upsampler factor P = 2
    /// - 010b: TX upsampler factor P = 4
    /// - 011b: TX upsampler factor P = 8
    /// - 100b: TX upsampler factor P = 16
    /// - 101b: TX upsampler factor P = 32
    /// - 110b: TX upsampler factor P = 64
    /// - 111b: Not used
    ///
    /// The default value is 0x04
    pub upsampler_p, set_upsampler_p: 3, 1;

    /// Custom frequency modulation enable
    ///
    /// # Values
    ///
    /// - 0b: CFM mode disabled
    /// - 1b: CFM mode enabled (write frequency word directly)
    ///
    /// The default value is 0x00
    pub cfm_data_en, set_cfm_data_en: 0;
}

impl Default for Mdmcfg2 {
    fn default() -> Self {
        Self(0x08)
    }
}

bitfield! {
    /// External Control Configuration
    ///
    /// # Address
    ///
    /// The address of this register is 0x2F06
    #[derive(Clone, Copy)]
    pub struct ExtCtrl(u8);

    pub ext_ctrl_not_used, _: 7, 3;

    /// Pin control enable. Pin control reuses the SPI interface pins to execute SRX, STX, SPWD, and IDLE strobes
    ///
    /// # Values
    ///
    /// - 0b: Pin control disabled
    /// - 1b: Pin control enabled
    ///
    /// The default value is 0x00
    pub pin_ctrl_en, set_pin_ctrl_en: 2;

    /// External 40k clock enable
    ///
    /// # Values
    ///
    /// - 0b: External 40k clock disabled
    /// - 1b: External 40k clock enabled. IOCFG3.GPIO3_CFG must be set to HIGHZ (EXT_40K_CLOCK)
    ///
    /// The default value is 0x00
    pub ext_40k_clock_en, set_ext_40k_clock_en: 1;

    /// Burst address increment enable
    ///
    /// # Values
    ///
    /// - 0b: Burst address increment disabled (i.e. consecutive writes to the same address location in burst mode)
    /// - 1b: Burst address increment enabled (i.e. the address is incremented during burst access)
    ///
    /// The default value is 0x01
    pub burst_addr_incr_en, set_burst_addr_incr_en: 0;
}

impl Default for ExtCtrl {
    fn default() -> Self {
        Self(0x01)
    }
}

bitfield! {
    /// RC Oscillator Calibration Fine
    ///
    /// # Address
    ///
    /// The address of this register is 0x2F07
    #[derive(Clone, Copy)]
    pub struct RccalFine(u8);

    pub rccal_fine_not_used, _: 7;

    /// 40 kHz RCOSC calibrated fine value
    pub rcc_fine, set_rcc_fine: 6, 0;
}

impl Default for RccalFine {
    fn default() -> Self {
        Self(0x00)
    }
}

bitfield! {
    /// RC Oscillator Calibration Coarse
    ///
    /// # Address
    ///
    /// The address of this register is 0x2F08
    #[derive(Clone, Copy)]
    pub struct RccalCoarse(u8);

    pub rccal_coarse_not_used, _: 7;

    /// 40 kHz RCOSC calibrated coarse value
    pub rcc_coarse, set_rcc_coarse: 6, 0;
}

impl Default for RccalCoarse {
    fn default() -> Self {
        Self(0x00)
    }
}

bitfield! {
    /// RC Oscillator Calibration Clock Offset
    ///
    /// # Address
    ///
    /// The address of this register is 0x2F09
    #[derive(Clone, Copy)]
    pub struct RccalOffset(u8);

    pub rccal_offset_not_used, _: 7, 5;

    /// For test purposes only, use values from SmartRF Studio.
    pub rccal_offset_reserved4_0, set_rccal_offset_reserved4_0: 4, 0;
}

impl Default for RccalOffset {
    fn default() -> Self {
        Self(0x00)
    }
}

bitfield! {
    /// Frequency Offset MSB
    ///
    /// # Address
    ///
    /// The address of this register is 0x2F0A
    #[derive(Clone, Copy)]
    pub struct Freqoff1(u8);

    /// Frequency offset [15:8]. Updated by user or SAFC strobe. The value is in two's complement format
    pub freq_off_15_8, set_freq_off_15_8: 7, 0;
}

impl Default for Freqoff1 {
    fn default() -> Self {
        Self(0x00)
    }
}

bitfield! {
    /// Frequency Offset LSB
    ///
    /// # Address
    ///
    /// The address of this register is 0x2F0B
    #[derive(Clone, Copy)]
    pub struct Freqoff0(u8);

    /// Frequency offset [7:0]. Updated by user or SAFC strobe. The value is in two's complement format
    pub freq_off_7_0, set_freq_off_7_0: 7, 0;
}

impl Default for Freqoff0 {
    fn default() -> Self {
        Self(0x00)
    }
}

bitfield! {
    /// Frequency Configuration [23:16]
    ///
    /// # Address
    ///
    /// The address of this register is 0x2F0C
    #[derive(Clone, Copy)]
    pub struct Freq2(u8);

    /// Frequency [23:16]<BR/>
    /// f_RF = f_vco/LO Divider [Hz] where f_vco = (FREQ/(2^16)*f_xosc) + (FREQOFF/(2^18)*f_xosc) [Hz] and the LO Divider is given by FS_CFG.FSD_BANDSELECT
    pub freq_23_16, set_freq_23_16: 7, 0;
}

impl Default for Freq2 {
    fn default() -> Self {
        Self(0x00)
    }
}

bitfield! {
    /// Frequency Configuration [15:8]
    ///
    /// # Address
    ///
    /// The address of this register is 0x2F0D
    #[derive(Clone, Copy)]
    pub struct Freq1(u8);

    /// Frequency [15:8]. See FREQ2
    pub freq_15_8, set_freq_15_8: 7, 0;
}

impl Default for Freq1 {
    fn default() -> Self {
        Self(0x00)
    }
}

bitfield! {
    /// Frequency Configuration [7:0]
    ///
    /// # Address
    ///
    /// The address of this register is 0x2F0E
    #[derive(Clone, Copy)]
    pub struct Freq0(u8);

    /// Frequency [7:0]. See FREQ2
    pub freq_7_0, set_freq_7_0: 7, 0;
}

impl Default for Freq0 {
    fn default() -> Self {
        Self(0x00)
    }
}

bitfield! {
    /// Analog to Digital Converter Configuration Reg. 2
    ///
    /// # Address
    ///
    /// The address of this register is 0x2F0F
    #[derive(Clone, Copy)]
    pub struct IfAdc2(u8);

    pub if_adc2_not_used, _: 7, 4;

    /// For test purposes only, use values from SmartRF Studio.
    pub if_adc2_reserved3_0, set_if_adc2_reserved3_0: 3, 0;
}

impl Default for IfAdc2 {
    fn default() -> Self {
        Self(0x02)
    }
}

bitfield! {
    /// Analog to Digital Converter Configuration Reg. 1
    ///
    /// # Address
    ///
    /// The address of this register is 0x2F10
    #[derive(Clone, Copy)]
    pub struct IfAdc1(u8);

    /// For test purposes only, use values from SmartRF Studio.
    pub if_adc1_reserved7_6, set_if_adc1_reserved7_6: 7, 6;

    /// For test purposes only, use values from SmartRF Studio.
    pub if_adc1_reserved5_4, set_if_adc1_reserved5_4: 5, 4;

    /// For test purposes only, use values from SmartRF Studio.
    pub if_adc1_reserved3_2, set_if_adc1_reserved3_2: 3, 2;

    /// For test purposes only, use values from SmartRF Studio.
    pub if_adc1_reserved1_0, set_if_adc1_reserved1_0: 1, 0;
}

impl Default for IfAdc1 {
    fn default() -> Self {
        Self(0x5a)
    }
}

bitfield! {
    /// Analog to Digital Converter Configuration Reg. 0
    ///
    /// # Address
    ///
    /// The address of this register is 0x2F11
    #[derive(Clone, Copy)]
    pub struct IfAdc0(u8);

    pub if_adc0_not_used, _: 7, 6;

    /// For test purposes only, use values from SmartRF Studio.
    pub if_adc0_reserved5_3, set_if_adc0_reserved5_3: 5, 3;

    /// For test purposes only, use values from SmartRF Studio.
    pub if_adc0_reserved2_1, set_if_adc0_reserved2_1: 2, 1;

    /// For test purposes only, use values from SmartRF Studio.
    pub if_adc0_reserved0, set_if_adc0_reserved0: 0;
}

impl Default for IfAdc0 {
    fn default() -> Self {
        Self(0x1a)
    }
}

bitfield! {
    /// Frequency Synthesizer Digital Reg. 1
    ///
    /// # Address
    ///
    /// The address of this register is 0x2F12
    #[derive(Clone, Copy)]
    pub struct FsDig1(u8);

    pub fs_dig1_not_used, _: 7, 6;

    /// Loop-filter switch configuration 1
    pub fsd_lpf_switch1_en, set_fsd_lpf_switch1_en: 5;

    /// Loop-filter switch configuration 2
    pub fsd_lpf_switch2_en, set_fsd_lpf_switch2_en: 4;

    /// For test purposes only, use values from SmartRF Studio.
    pub fs_dig1_reserved3_2, set_fs_dig1_reserved3_2: 3, 2;

    /// For test purposes only, use values from SmartRF Studio.
    pub fs_dig1_reserved1, set_fs_dig1_reserved1: 1;

    /// For test purposes only, use values from SmartRF Studio.
    pub fs_dig1_reserved0, set_fs_dig1_reserved0: 0;
}

impl Default for FsDig1 {
    fn default() -> Self {
        Self(0x08)
    }
}

bitfield! {
    /// Frequency Synthesizer Digital Reg. 0
    ///
    /// # Address
    ///
    /// The address of this register is 0x2F13
    #[derive(Clone, Copy)]
    pub struct FsDig0(u8);

    /// For test purposes only, use values from SmartRF Studio.
    pub fs_dig0_reserved7_6, set_fs_dig0_reserved7_6: 7, 6;

    /// For test purposes only, use values from SmartRF Studio.
    pub fs_dig0_reserved5_4, set_fs_dig0_reserved5_4: 5, 4;

    /// FS loop bandwidth in RX
    ///
    /// # Values
    ///
    /// - 00b: 200 kHz
    /// - 01b: 300 kHz
    /// - 10b: 400 kHz
    /// - 11b: 500 kHz
    ///
    /// The default value is 0x02
    pub rx_lpf_bw, set_rx_lpf_bw: 3, 2;

    /// FS loop bandwidth in TX
    ///
    /// # Values
    ///
    /// - 00b: 200 kHz
    /// - 01b: 300 kHz
    /// - 10b: 400 kHz
    /// - 11b: 500 kHz
    ///
    /// The default value is 0x02
    pub tx_lpf_bw, set_tx_lpf_bw: 1, 0;
}

impl Default for FsDig0 {
    fn default() -> Self {
        Self(0x5a)
    }
}

bitfield! {
    /// Frequency Synthesizer Calibration Reg. 3
    ///
    /// # Address
    ///
    /// The address of this register is 0x2F14
    #[derive(Clone, Copy)]
    pub struct FsCal3(u8);

    /// For test purposes only, use values from SmartRF Studio.
    pub fs_cal3_reserved7, set_fs_cal3_reserved7: 7;

    /// KVCO high resolution enable
    ///
    /// # Values
    ///
    /// - 0b: High resolution disabled (normal resolution mode)
    /// - 1b: High resolution enabled (increased charge pump calibration, but will extend the calibration time)
    ///
    /// The default value is 0x00
    pub kvco_high_res_cfg, set_kvco_high_res_cfg: 6;

    /// For test purposes only, use values from SmartRF Studio.
    pub fs_cal3_reserved5_4, set_fs_cal3_reserved5_4: 5, 4;

    /// For test purposes only, use values from SmartRF Studio.
    pub fs_cal3_reserved3_0, set_fs_cal3_reserved3_0: 3, 0;
}

impl Default for FsCal3 {
    fn default() -> Self {
        Self(0x00)
    }
}

bitfield! {
    /// Frequency Synthesizer Calibration Reg. 2
    ///
    /// # Address
    ///
    /// The address of this register is 0x2F15
    #[derive(Clone, Copy)]
    pub struct FsCal2(u8);

    pub fs_cal2_not_used, _: 7, 6;

    /// For test purposes only, use values from SmartRF Studio.
    pub fs_cal2_reserved5_0, set_fs_cal2_reserved5_0: 5, 0;
}

impl Default for FsCal2 {
    fn default() -> Self {
        Self(0x20)
    }
}

bitfield! {
    /// Frequency Synthesizer Calibration Reg. 1
    ///
    /// # Address
    ///
    /// The address of this register is 0x2F16
    #[derive(Clone, Copy)]
    pub struct FsCal1(u8);

    /// For test purposes only, use values from SmartRF Studio.
    pub fs_cal1_reserved7_6, set_fs_cal1_reserved7_6: 7, 6;

    /// For test purposes only, use values from SmartRF Studio.
    pub fs_cal1_reserved5_4, set_fs_cal1_reserved5_4: 5, 4;

    /// For test purposes only, use values from SmartRF Studio.
    pub fs_cal1_reserved3_2, set_fs_cal1_reserved3_2: 3, 2;

    /// For test purposes only, use values from SmartRF Studio.
    pub fs_cal1_reserved1_0, set_fs_cal1_reserved1_0: 1, 0;
}

impl Default for FsCal1 {
    fn default() -> Self {
        Self(0x00)
    }
}

bitfield! {
    /// Frequency Synthesizer Calibration Reg. 0
    ///
    /// # Address
    ///
    /// The address of this register is 0x2F17
    #[derive(Clone, Copy)]
    pub struct FsCal0(u8);

    pub fs_cal0_not_used, _: 7, 4;

    /// Out of lock detector average time
    ///
    /// # Values
    ///
    /// - 00b: Average the measurement over 512 cycles
    /// - 01b: Average the measurement over 1024 cycles
    /// - 10b: Average the measurement over 256 cycles
    /// - 11b: Infinite average
    ///
    /// The default value is 0x00
    pub lock_cfg, set_lock_cfg: 3, 2;

    /// For test purposes only, use values from SmartRF Studio.
    pub fs_cal0_reserved1_0, set_fs_cal0_reserved1_0: 1, 0;
}

impl Default for FsCal0 {
    fn default() -> Self {
        Self(0x00)
    }
}

bitfield! {
    /// Frequency Synthesizer Charge Pump Configuration
    ///
    /// # Address
    ///
    /// The address of this register is 0x2F18
    #[derive(Clone, Copy)]
    pub struct FsChp(u8);

    pub fs_chp_not_used, _: 7, 6;

    /// For test purposes only, use values from SmartRF Studio.
    pub fs_chp_reserved5_0, set_fs_chp_reserved5_0: 5, 0;
}

impl Default for FsChp {
    fn default() -> Self {
        Self(0x28)
    }
}

bitfield! {
    /// Frequency Synthesizer Divide by 2
    ///
    /// # Address
    ///
    /// The address of this register is 0x2F19
    #[derive(Clone, Copy)]
    pub struct FsDivtwo(u8);

    pub fs_divtwo_not_used, _: 7, 2;

    /// For test purposes only, use values from SmartRF Studio.
    pub fs_divtwo_reserved1_0, set_fs_divtwo_reserved1_0: 1, 0;
}

impl Default for FsDivtwo {
    fn default() -> Self {
        Self(0x01)
    }
}

bitfield! {
    /// FS Digital Synthesizer Module Configuration Reg. 1
    ///
    /// # Address
    ///
    /// The address of this register is 0x2F1A
    #[derive(Clone, Copy)]
    pub struct FsDsm1(u8);

    pub fs_dsm1_not_used, _: 7, 3;

    /// For test purposes only, use values from SmartRF Studio.
    pub fs_dsm1_reserved2, set_fs_dsm1_reserved2: 2;

    /// For test purposes only, use values from SmartRF Studio.
    pub fs_dsm1_reserved1_0, set_fs_dsm1_reserved1_0: 1, 0;
}

impl Default for FsDsm1 {
    fn default() -> Self {
        Self(0x00)
    }
}

bitfield! {
    /// FS Digital Synthesizer Module Configuration Reg. 0
    ///
    /// # Address
    ///
    /// The address of this register is 0x2F1B
    #[derive(Clone, Copy)]
    pub struct FsDsm0(u8);

    /// For test purposes only, use values from SmartRF Studio.
    pub fs_dsm0_reserved7_4, set_fs_dsm0_reserved7_4: 7, 4;

    /// For test purposes only, use values from SmartRF Studio.
    pub fs_dsm0_reserved3, set_fs_dsm0_reserved3: 3;

    /// For test purposes only, use values from SmartRF Studio.
    pub fs_dsm0_reserved2_0, set_fs_dsm0_reserved2_0: 2, 0;
}

impl Default for FsDsm0 {
    fn default() -> Self {
        Self(0x03)
    }
}

bitfield! {
    /// Frequency Synthesizer Divider Chain Configuration Reg. 1
    ///
    /// # Address
    ///
    /// The address of this register is 0x2F1C
    #[derive(Clone, Copy)]
    pub struct FsDvc1(u8);

    /// For test purposes only, use values from SmartRF Studio.
    pub fs_dvc1_reserved7_6, set_fs_dvc1_reserved7_6: 7, 6;

    /// For test purposes only, use values from SmartRF Studio.
    pub fs_dvc1_reserved5_4, set_fs_dvc1_reserved5_4: 5, 4;

    /// For test purposes only, use values from SmartRF Studio.
    pub fs_dvc1_reserved3_2, set_fs_dvc1_reserved3_2: 3, 2;

    /// For test purposes only, use values from SmartRF Studio.
    pub fs_dvc1_reserved1_0, set_fs_dvc1_reserved1_0: 1, 0;
}

impl Default for FsDvc1 {
    fn default() -> Self {
        Self(0xff)
    }
}

bitfield! {
    /// Frequency Synthesizer Divider Chain Configuration Reg. 0
    ///
    /// # Address
    ///
    /// The address of this register is 0x2F1D
    #[derive(Clone, Copy)]
    pub struct FsDvc0(u8);

    pub fs_dvc0_not_used, _: 7, 5;

    /// For test purposes only, use values from SmartRF Studio.
    pub fs_dvc0_reserved4_3, set_fs_dvc0_reserved4_3: 4, 3;

    /// For test purposes only, use values from SmartRF Studio.
    pub fs_dvc0_reserved2_0, set_fs_dvc0_reserved2_0: 2, 0;
}

impl Default for FsDvc0 {
    fn default() -> Self {
        Self(0x1f)
    }
}

bitfield! {
    /// Frequency Synthesizer Local Bias Configuration
    ///
    /// # Address
    ///
    /// The address of this register is 0x2F1E
    #[derive(Clone, Copy)]
    pub struct FsLbi(u8);

    pub fs_lbi_not_used, _: 7, 0;
}

impl Default for FsLbi {
    fn default() -> Self {
        Self(0x00)
    }
}

bitfield! {
    /// Frequency Synthesizer Phase Frequency Detector Configuration
    ///
    /// # Address
    ///
    /// The address of this register is 0x2F1F
    #[derive(Clone, Copy)]
    pub struct FsPfd(u8);

    pub fsd_pfd_not_used, _: 7;

    /// For test purposes only, use values from SmartRF Studio.
    pub fs_pfd_reserved6_4, set_fs_pfd_reserved6_4: 6, 4;

    /// For test purposes only, use values from SmartRF Studio.
    pub fs_pfd_reserved3_2, set_fs_pfd_reserved3_2: 3, 2;

    /// For test purposes only, use values from SmartRF Studio.
    pub fs_pfd_reserved1_0, set_fs_pfd_reserved1_0: 1, 0;
}

impl Default for FsPfd {
    fn default() -> Self {
        Self(0x51)
    }
}

bitfield! {
    /// Frequency Synthesizer Prescaler Configuration
    ///
    /// # Address
    ///
    /// The address of this register is 0x2F20
    #[derive(Clone, Copy)]
    pub struct FsPre(u8);

    pub fs_pre_not_used, _: 7;

    /// For test purposes only, use values from SmartRF Studio.
    pub fs_pre_reserved6_5, set_fs_pre_reserved6_5: 6, 5;

    /// For test purposes only, use values from SmartRF Studio.
    pub fs_pre_reserved4_3, set_fs_pre_reserved4_3: 4, 3;

    /// For test purposes only, use values from SmartRF Studio.
    pub fs_pre_reserved2_0, set_fs_pre_reserved2_0: 2, 0;
}

impl Default for FsPre {
    fn default() -> Self {
        Self(0x2c)
    }
}

bitfield! {
    /// Frequency Synthesizer Divider Regulator Configuration
    ///
    /// # Address
    ///
    /// The address of this register is 0x2F21
    #[derive(Clone, Copy)]
    pub struct FsRegDivCml(u8);

    pub fs_reg_div_cml_not_used, _: 7, 5;

    /// For test purposes only, use values from SmartRF Studio.
    pub fs_reg_div_cml_reserved4_2, set_fs_reg_div_cml_reserved4_2: 4, 2;

    /// For test purposes only, use values from SmartRF Studio.
    pub fs_reg_div_cml_reserved1_0, set_fs_reg_div_cml_reserved1_0: 1, 0;
}

impl Default for FsRegDivCml {
    fn default() -> Self {
        Self(0x11)
    }
}

bitfield! {
    /// Frequency Synthesizer Spare
    ///
    /// # Address
    ///
    /// The address of this register is 0x2F22
    #[derive(Clone, Copy)]
    pub struct FsSpare(u8);

    /// For test purposes only, use values from SmartRF Studio.
    pub fs_spare_reserved7_0, set_fs_spare_reserved7_0: 7, 0;
}

impl Default for FsSpare {
    fn default() -> Self {
        Self(0x00)
    }
}

bitfield! {
    /// FS Voltage Controlled Oscillator Configuration Reg. 4
    ///
    /// # Address
    ///
    /// The address of this register is 0x2F23
    #[derive(Clone, Copy)]
    pub struct FsVco4(u8);

    pub fs_vco4_not_used, _: 7, 5;

    /// For test purposes only, use values from SmartRF Studio.
    pub fs_vco4_reserved4_0, set_fs_vco4_reserved4_0: 4, 0;
}

impl Default for FsVco4 {
    fn default() -> Self {
        Self(0x14)
    }
}

bitfield! {
    /// FS Voltage Controlled Oscillator Configuration Reg. 3
    ///
    /// # Address
    ///
    /// The address of this register is 0x2F24
    #[derive(Clone, Copy)]
    pub struct FsVco3(u8);

    pub fs_vco3_not_used, _: 7, 1;

    /// For test purposes only, use values from SmartRF Studio.
    pub fs_vco3_reserved0, set_fs_vco3_reserved0: 0;
}

impl Default for FsVco3 {
    fn default() -> Self {
        Self(0x00)
    }
}

bitfield! {
    /// FS Voltage Controlled Oscillator Configuration Reg. 2
    ///
    /// # Address
    ///
    /// The address of this register is 0x2F25
    #[derive(Clone, Copy)]
    pub struct FsVco2(u8);

    pub fs_vco2_not_used, _: 7;

    /// For test purposes only, use values from SmartRF Studio.
    pub fs_vco2_reserved6_0, set_fs_vco2_reserved6_0: 6, 0;
}

impl Default for FsVco2 {
    fn default() -> Self {
        Self(0x00)
    }
}

bitfield! {
    /// FS Voltage Controlled Oscillator Configuration Reg. 1
    ///
    /// # Address
    ///
    /// The address of this register is 0x2F26
    #[derive(Clone, Copy)]
    pub struct FsVco1(u8);

    /// VCO VCDAC configuration. Used in open-loop CAL mode.  Note that avdd is the internal VCO regulated voltage
    ///
    /// # Values
    ///
    /// - 000000b: VCDAC out = min 160 mV
    /// - 111111b: VCDAC out = max avdd - 160 mV
    ///
    /// The default value is 0x00
    pub fsd_vcdac, set_fsd_vcdac: 7, 2;

    /// For test purposes only, use values from SmartRF Studio.
    pub fs_vco1_reserved1_0, set_fs_vco1_reserved1_0: 1, 0;
}

impl Default for FsVco1 {
    fn default() -> Self {
        Self(0x00)
    }
}

bitfield! {
    /// FS Voltage Controlled Oscillator Configuration Reg. 0
    ///
    /// # Address
    ///
    /// The address of this register is 0x2F27
    #[derive(Clone, Copy)]
    pub struct FsVco0(u8);

    /// For test purposes only, use values from SmartRF Studio.
    pub fs_vco0_reserved7, set_fs_vco0_reserved7: 7;

    /// For test purposes only, use values from SmartRF Studio.
    pub fs_vco0_reserved6_2, set_fs_vco0_reserved6_2: 6, 2;

    /// For test purposes only, use values from SmartRF Studio.
    pub fs_vco0_reserved1_0, set_fs_vco0_reserved1_0: 1, 0;
}

impl Default for FsVco0 {
    fn default() -> Self {
        Self(0x81)
    }
}

bitfield! {
    /// Global Bias Configuration Reg. 6
    ///
    /// # Address
    ///
    /// The address of this register is 0x2F28
    #[derive(Clone, Copy)]
    pub struct Gbias6(u8);

    pub gbias6_not_used, _: 7, 6;

    /// For test purposes only, use values from SmartRF Studio.
    pub gbias6_reserved5_0, set_gbias6_reserved5_0: 5, 0;
}

impl Default for Gbias6 {
    fn default() -> Self {
        Self(0x00)
    }
}

bitfield! {
    /// Global Bias Configuration Reg. 5
    ///
    /// # Address
    ///
    /// The address of this register is 0x2F29
    #[derive(Clone, Copy)]
    pub struct Gbias5(u8);

    pub gbias5_not_used, _: 7, 4;

    /// For test purposes only, use values from SmartRF Studio.
    pub gbias5_reserved3_0, set_gbias5_reserved3_0: 3, 0;
}

impl Default for Gbias5 {
    fn default() -> Self {
        Self(0x02)
    }
}

bitfield! {
    /// Global Bias Configuration Reg. 4
    ///
    /// # Address
    ///
    /// The address of this register is 0x2F2A
    #[derive(Clone, Copy)]
    pub struct Gbias4(u8);

    pub gbias4_not_used, _: 7, 6;

    /// For test purposes only, use values from SmartRF Studio.
    pub gbias4_reserved5_0, set_gbias4_reserved5_0: 5, 0;
}

impl Default for Gbias4 {
    fn default() -> Self {
        Self(0x00)
    }
}

bitfield! {
    /// Global Bias Configuration Reg. 3
    ///
    /// # Address
    ///
    /// The address of this register is 0x2F2B
    #[derive(Clone, Copy)]
    pub struct Gbias3(u8);

    pub gbias3_not_used, _: 7, 6;

    /// For test purposes only, use values from SmartRF Studio.
    pub gbias3_reserved5_0, set_gbias3_reserved5_0: 5, 0;
}

impl Default for Gbias3 {
    fn default() -> Self {
        Self(0x00)
    }
}

bitfield! {
    /// Global Bias Configuration Reg. 2
    ///
    /// # Address
    ///
    /// The address of this register is 0x2F2C
    #[derive(Clone, Copy)]
    pub struct Gbias2(u8);

    pub gbias2_not_used, _: 7;

    /// For test purposes only, use values from SmartRF Studio.
    pub gbias2_reserved6_3, set_gbias2_reserved6_3: 6, 3;

    /// For test purposes only, use values from SmartRF Studio.
    pub gbias2_reserved2_0, set_gbias2_reserved2_0: 2, 0;
}

impl Default for Gbias2 {
    fn default() -> Self {
        Self(0x10)
    }
}

bitfield! {
    /// Global Bias Configuration Reg. 1
    ///
    /// # Address
    ///
    /// The address of this register is 0x2F2D
    #[derive(Clone, Copy)]
    pub struct Gbias1(u8);

    pub gbias1_not_used, _: 7, 5;

    /// For test purposes only, use values from SmartRF Studio.
    pub gbias1_reserved4_0, set_gbias1_reserved4_0: 4, 0;
}

impl Default for Gbias1 {
    fn default() -> Self {
        Self(0x00)
    }
}

bitfield! {
    /// Global Bias Configuration Reg. 0
    ///
    /// # Address
    ///
    /// The address of this register is 0x2F2E
    #[derive(Clone, Copy)]
    pub struct Gbias0(u8);

    pub gbias0_not_used, _: 7, 2;

    /// For test purposes only, use values from SmartRF Studio.
    pub gbias0_reserved1, set_gbias0_reserved1: 1;

    /// For test purposes only, use values from SmartRF Studio.
    pub gbias0_reserved0, set_gbias0_reserved0: 0;
}

impl Default for Gbias0 {
    fn default() -> Self {
        Self(0x00)
    }
}

bitfield! {
    /// Intermediate Frequency Amplifier Configuration
    ///
    /// # Address
    ///
    /// The address of this register is 0x2F2F
    #[derive(Clone, Copy)]
    pub struct Ifamp(u8);

    pub ifamp_not_used, _: 7, 4;

    /// Single side bandwidth control bits covering frequency range from 300 kHz to 1500 kHz. Single Side BW > f_IF+(RX Filter BW/2)
    ///
    /// # Values
    ///
    /// - 00b:  300 kHz
    /// - 01b:  600 kHz
    /// - 10b: 1000 kHz
    /// - 11b: 1500 kHz
    ///
    /// The default value is 0x00
    pub ifamp_bw, set_ifamp_bw: 3, 2;

    /// For test purposes only, use values from SmartRF Studio.
    pub ifamp_reserved1_0, set_ifamp_reserved1_0: 1, 0;
}

impl Default for Ifamp {
    fn default() -> Self {
        Self(0x01)
    }
}

bitfield! {
    /// Low Noise Amplifier Configuration
    ///
    /// # Address
    ///
    /// The address of this register is 0x2F30
    #[derive(Clone, Copy)]
    pub struct Lna(u8);

    pub lna_not_used, _: 7, 2;

    /// For test purposes only, use values from SmartRF Studio.
    pub lna_reserved1_0, set_lna_reserved1_0: 1, 0;
}

impl Default for Lna {
    fn default() -> Self {
        Self(0x01)
    }
}

bitfield! {
    /// RX Mixer Configuration
    ///
    /// # Address
    ///
    /// The address of this register is 0x2F31
    #[derive(Clone, Copy)]
    pub struct Rxmix(u8);

    pub rxmix_not_used, _: 7, 2;

    /// For test purposes only, use values from SmartRF Studio.
    pub rxmix_reserved1_0, set_rxmix_reserved1_0: 1, 0;
}

impl Default for Rxmix {
    fn default() -> Self {
        Self(0x01)
    }
}

bitfield! {
    /// Crystal Oscillator Configuration Reg. 5
    ///
    /// # Address
    ///
    /// The address of this register is 0x2F32
    #[derive(Clone, Copy)]
    pub struct Xosc5(u8);

    pub xosc5_not_used, _: 7, 4;

    /// For test purposes only, use values from SmartRF Studio.
    pub xosc5_reserved3_0, set_xosc5_reserved3_0: 3, 0;
}

impl Default for Xosc5 {
    fn default() -> Self {
        Self(0x0c)
    }
}

bitfield! {
    /// Crystal Oscillator Configuration Reg. 4
    ///
    /// # Address
    ///
    /// The address of this register is 0x2F33
    #[derive(Clone, Copy)]
    pub struct Xosc4(u8);

    /// For test purposes only, use values from SmartRF Studio.
    pub xosc4_reserved7_0, set_xosc4_reserved7_0: 7, 0;
}

impl Default for Xosc4 {
    fn default() -> Self {
        Self(0xa0)
    }
}

bitfield! {
    /// Crystal Oscillator Configuration Reg. 3
    ///
    /// # Address
    ///
    /// The address of this register is 0x2F34
    #[derive(Clone, Copy)]
    pub struct Xosc3(u8);

    /// For test purposes only, use values from SmartRF Studio.
    pub xosc3_reserved7_2, set_xosc3_reserved7_2: 7, 2;

    /// For test purposes only, use values from SmartRF Studio.
    pub xosc3_reserved1_0, set_xosc3_reserved1_0: 1, 0;
}

impl Default for Xosc3 {
    fn default() -> Self {
        Self(0x03)
    }
}

bitfield! {
    /// Crystal Oscillator Configuration Reg. 2
    ///
    /// # Address
    ///
    /// The address of this register is 0x2F35
    #[derive(Clone, Copy)]
    pub struct Xosc2(u8);

    pub xosc2_not_used, _: 7, 4;

    /// For test purposes only, use values from SmartRF Studio.
    pub xosc2_reserved3_2, set_xosc2_reserved3_2: 3, 2;

    /// For test purposes only, use values from SmartRF Studio.
    pub xosc2_reserved1, set_xosc2_reserved1: 1;

    ///
    /// # Values
    ///
    /// - 0b: The XOSC will be turned off if the SXOFF, SPWD, or SWOR command strobes are issued
    /// - 1b: The XOSC is forced on even if an SXOFF, SPWD, or SWOR command strobe has been issued. This can be used to enable fast start-up from SLEEP/XOFF on the expense of a higher current consumption
    ///
    /// The default value is 0x00
    pub xosc_core_pd_override, set_xosc_core_pd_override: 0;
}

impl Default for Xosc2 {
    fn default() -> Self {
        Self(0x04)
    }
}

bitfield! {
    /// Crystal Oscillator Configuration Reg. 1
    ///
    /// # Address
    ///
    /// The address of this register is 0x2F36
    #[derive(Clone, Copy)]
    pub struct Xosc1(u8);

    pub xosc1_not_used, _: 7, 3;

    /// For test purposes only, use values from SmartRF Studio.
    pub xosc1_reserved2, set_xosc1_reserved2: 2;

    /// XOSC buffer select. Selects internal XOSC buffer for RF PLL
    ///
    /// # Values
    ///
    /// - 0b: Low power, single ended buffer (differential buffer is shut down)
    /// - 1b: Low phase noise, differential buffer (low power buffer still used for digital clock)
    ///
    /// The default value is 0x00
    pub xosc_buf_sel, set_xosc_buf_sel: 1;

    /// XOSC is stable (has finished settling)
    pub xosc_stable, _: 0;
}

impl Default for Xosc1 {
    fn default() -> Self {
        Self(0x01)
    }
}

bitfield! {
    /// Crystal Oscillator Configuration Reg. 0
    ///
    /// # Address
    ///
    /// The address of this register is 0x2F37
    #[derive(Clone, Copy)]
    pub struct Xosc0(u8);

    pub xosc0_not_used, _: 7, 2;

    /// For test purposes only, use values from SmartRF Studio.
    pub xosc0_reserved1, _: 1;

    /// For test purposes only, use values from SmartRF Studio.
    pub xosc0_reserved0, _: 0;
}

impl Default for Xosc0 {
    fn default() -> Self {
        Self(0x00)
    }
}

bitfield! {
    /// Analog Spare
    ///
    /// # Address
    ///
    /// The address of this register is 0x2F38
    #[derive(Clone, Copy)]
    pub struct AnalogSpare(u8);

    /// For test purposes only, use values from SmartRF Studio.
    pub analog_spare_reserved7_0, set_analog_spare_reserved7_0: 7, 0;
}

impl Default for AnalogSpare {
    fn default() -> Self {
        Self(0x00)
    }
}

bitfield! {
    /// Power Amplifier Configuration Reg. 3
    ///
    /// # Address
    ///
    /// The address of this register is 0x2F39
    #[derive(Clone, Copy)]
    pub struct PaCfg3(u8);

    pub pa_cfg3_not_used, _: 7, 3;

    /// For test purposes only, use values from SmartRF Studio.
    pub pa_cfg3_reserved2_0, set_pa_cfg3_reserved2_0: 2, 0;
}

impl Default for PaCfg3 {
    fn default() -> Self {
        Self(0x00)
    }
}

bitfield! {
    /// eWOR Timer Counter Value MSB
    ///
    /// # Address
    ///
    /// The address of this register is 0x2F64
    #[derive(Clone, Copy)]
    pub struct WorTime1(u8);

    /// eWOR timer counter value [15:8]
    pub wor_status_15_8, _: 7, 0;
}

impl Default for WorTime1 {
    fn default() -> Self {
        Self(0x00)
    }
}

bitfield! {
    /// eWOR Timer Counter Value LSB
    ///
    /// # Address
    ///
    /// The address of this register is 0x2F65
    #[derive(Clone, Copy)]
    pub struct WorTime0(u8);

    /// eWOR timer counter value [7:0]
    pub wor_status_7_0, _: 7, 0;
}

impl Default for WorTime0 {
    fn default() -> Self {
        Self(0x00)
    }
}

bitfield! {
    /// eWOR Timer Capture Value MSB
    ///
    /// # Address
    ///
    /// The address of this register is 0x2F66
    #[derive(Clone, Copy)]
    pub struct WorCapture1(u8);

    /// eWOR timer capture value [15:8]. Capture timer value on sync detect to simplify timer re-synchronization
    pub wor_capture_15_8, _: 7, 0;
}

impl Default for WorCapture1 {
    fn default() -> Self {
        Self(0x00)
    }
}

bitfield! {
    /// eWOR Timer Capture Value LSB
    ///
    /// # Address
    ///
    /// The address of this register is 0x2F67
    #[derive(Clone, Copy)]
    pub struct WorCapture0(u8);

    /// eWOR timer capture Value [7:0]. Capture timer value on sync detect to simplify timer re-synchronization
    pub wor_capture_7_0, _: 7, 0;
}

impl Default for WorCapture0 {
    fn default() -> Self {
        Self(0x00)
    }
}

bitfield! {
    /// MARC Built-In Self-Test
    ///
    /// # Address
    ///
    /// The address of this register is 0x2F68
    #[derive(Clone, Copy)]
    pub struct Bist(u8);

    pub bist_not_used, _: 7, 4;

    /// For test purposes only, use values from SmartRF Studio.
    pub bist_reserved3, set_bist_reserved3: 3;

    /// For test purposes only, use values from SmartRF Studio.
    pub bist_reserved2, _: 2;

    /// For test purposes only, use values from SmartRF Studio.
    pub bist_reserved1, _: 1;

    /// For test purposes only, use values from SmartRF Studio.
    pub bist_reserved0, set_bist_reserved0: 0;
}

impl Default for Bist {
    fn default() -> Self {
        Self(0x00)
    }
}

bitfield! {
    /// DC Filter Offset I MSB
    ///
    /// # Address
    ///
    /// The address of this register is 0x2F69
    #[derive(Clone, Copy)]
    pub struct DcfiltoffsetI1(u8);

    /// DC compensation, real value [15:8]
    pub dcfilt_offset_i_15_8, set_dcfilt_offset_i_15_8: 7, 0;
}

impl Default for DcfiltoffsetI1 {
    fn default() -> Self {
        Self(0x00)
    }
}

bitfield! {
    /// DC Filter Offset I LSB
    ///
    /// # Address
    ///
    /// The address of this register is 0x2F6A
    #[derive(Clone, Copy)]
    pub struct DcfiltoffsetI0(u8);

    /// DC compensation, real value [7:0]
    pub dcfilt_offset_i_7_0, set_dcfilt_offset_i_7_0: 7, 0;
}

impl Default for DcfiltoffsetI0 {
    fn default() -> Self {
        Self(0x00)
    }
}

bitfield! {
    /// DC Filter Offset Q MSB
    ///
    /// # Address
    ///
    /// The address of this register is 0x2F6B
    #[derive(Clone, Copy)]
    pub struct DcfiltoffsetQ1(u8);

    /// DC compensation, imaginary value [15:8]
    pub dcfilt_offset_q_15_8, set_dcfilt_offset_q_15_8: 7, 0;
}

impl Default for DcfiltoffsetQ1 {
    fn default() -> Self {
        Self(0x00)
    }
}

bitfield! {
    /// DC Filter Offset Q LSB
    ///
    /// # Address
    ///
    /// The address of this register is 0x2F6C
    #[derive(Clone, Copy)]
    pub struct DcfiltoffsetQ0(u8);

    /// DC compensation, imaginary value [7:0]
    pub dcfilt_offset_q_7_0, set_dcfilt_offset_q_7_0: 7, 0;
}

impl Default for DcfiltoffsetQ0 {
    fn default() -> Self {
        Self(0x00)
    }
}

bitfield! {
    /// IQ Imbalance Value I MSB
    ///
    /// # Address
    ///
    /// The address of this register is 0x2F6D
    #[derive(Clone, Copy)]
    pub struct IqieI1(u8);

    /// IQ imbalance value, real part [15:8]
    pub iqie_i_15_8, set_iqie_i_15_8: 7, 0;
}

impl Default for IqieI1 {
    fn default() -> Self {
        Self(0x00)
    }
}

bitfield! {
    /// IQ Imbalance Value I LSB
    ///
    /// # Address
    ///
    /// The address of this register is 0x2F6E
    #[derive(Clone, Copy)]
    pub struct IqieI0(u8);

    /// IQ imbalance value, real part [7:0]
    pub iqie_i_7_0, set_iqie_i_7_0: 7, 0;
}

impl Default for IqieI0 {
    fn default() -> Self {
        Self(0x00)
    }
}

bitfield! {
    /// IQ Imbalance Value Q MSB
    ///
    /// # Address
    ///
    /// The address of this register is 0x2F6F
    #[derive(Clone, Copy)]
    pub struct IqieQ1(u8);

    /// IQ imbalance value, imaginary part [15:8]
    pub iqie_q_15_8, set_iqie_q_15_8: 7, 0;
}

impl Default for IqieQ1 {
    fn default() -> Self {
        Self(0x00)
    }
}

bitfield! {
    /// IQ Imbalance Value Q LSB
    ///
    /// # Address
    ///
    /// The address of this register is 0x2F70
    #[derive(Clone, Copy)]
    pub struct IqieQ0(u8);

    /// IQ imbalance value, imaginary part [7:0]
    pub iqie_q_7_0, set_iqie_q_7_0: 7, 0;
}

impl Default for IqieQ0 {
    fn default() -> Self {
        Self(0x00)
    }
}

bitfield! {
    /// Received Signal Strength Indicator Reg. 1
    ///
    /// # Address
    ///
    /// The address of this register is 0x2F71
    #[derive(Clone, Copy)]
    pub struct Rssi1(u8);

    /// Received signal strength indicator. 8 MSB of RSSI[11:0]. RSSI[11:0] is a two's complement number with 0.0625 dB resolution hence ranging from -128 to 127 dBm. A value of -128 dBm indicates that the RSSI is invalid. To get a correct RSSI value a calibrated RSSI offset value should be subtracted from the value given by RSSI[11:0]. This RSSI offset value can either be subtracted from RSSI[11:0] manually or the offset can be written to AGC_GAIN_ADJUST.GAIN_ADJUSTMENT meaning that RSSI[11:0] will give a correct value directly
    pub rssi_11_4, _: 7, 0;
}

impl Default for Rssi1 {
    fn default() -> Self {
        Self(0x80)
    }
}

bitfield! {
    /// Received Signal Strength Indicator Reg.0
    ///
    /// # Address
    ///
    /// The address of this register is 0x2F72
    #[derive(Clone, Copy)]
    pub struct Rssi0(u8);

    pub rssi0_not_used, _: 7;

    /// Received signal strength indicator. 4 LSB of RSSI[11:0]. See RSSI1.RSSI_11_4
    pub rssi_3_0, _: 6, 3;

    /// Carrier sense
    ///
    /// # Values
    ///
    /// - 0b: No carrier detected
    /// - 1b: carrier detected
    ///
    /// The default value is 0x00
    pub carrier_sense, _: 2;

    /// Carrier sense valid
    ///
    /// # Values
    ///
    /// - 0b: Carrier sense not valid
    /// - 1b: Carrier sense valid
    ///
    /// The default value is 0x00
    pub carrier_sense_valid, _: 1;

    /// RSSI valid
    ///
    /// # Values
    ///
    /// - 0b: RSSI not valid
    /// - 1b: RSSI valid
    ///
    /// The default value is 0x00
    pub rssi_valid, _: 0;
}

impl Default for Rssi0 {
    fn default() -> Self {
        Self(0x00)
    }
}

bitfield! {
    /// MARC State
    ///
    /// # Address
    ///
    /// The address of this register is 0x2F73
    #[derive(Clone, Copy)]
    pub struct Marcstate(u8);

    pub marcstate_not_used, _: 7;

    /// MARC 2 pin state value
    ///
    /// # Values
    ///
    /// - 00b: SETTLING
    /// - 01b: TX
    /// - 10b: IDLE
    /// - 11b: RX
    ///
    /// The default value is 0x02
    pub marc_2pin_state, _: 6, 5;

    pub marc_state, _: 4, 0;
}

impl Default for Marcstate {
    fn default() -> Self {
        Self(0x41)
    }
}

bitfield! {
    /// Link Quality Indicator Value
    ///
    /// # Address
    ///
    /// The address of this register is 0x2F74
    #[derive(Clone, Copy)]
    pub struct LqiVal(u8);

    /// CRC OK. Asserted in RX when PKT_CFG1.CRC_CFG = 1 or 10b and a good packet is received. This signal is always on if the radio is in TX or if the radio is in RX and PKT_CFG1.CRC_CFG = 0. The signal is de-asserted when RX mode is entered and PKT_CFG1.CRC_CFG != 0
    ///
    /// # Values
    ///
    /// - 0b: CRC check not ok (bit error)
    /// - 1b: CRC check ok (no bit error)
    ///
    /// The default value is 0x00
    pub pkt_crc_ok, _: 7;

    /// Link quality indicator. 0 when not valid. A low value indicates a better link than what a high value does
    pub lqi, _: 6, 0;
}

impl Default for LqiVal {
    fn default() -> Self {
        Self(0x00)
    }
}

bitfield! {
    /// Preamble and Sync Word Error
    ///
    /// # Address
    ///
    /// The address of this register is 0x2F75
    #[derive(Clone, Copy)]
    pub struct PqtSyncErr(u8);

    /// Preamble qualifier value. The actual preamble qualifier value can be greater than 15 but since PQT_ERROR is only 4 bits wide PQT_ERROR = MIN[actual PQT qualifier value] modulo 16. This means that if PQT _ERROR = 0001b the actual preamble qualifier value is either 1 or 17. When a sync word is detected (SYNC_EVENT is asserted) the PQT_ERROR register field is not updated again before RX mode is re-entered. As long as the radio is in RX searching for a sync word the register field will be updated continuously
    pub pqt_error, _: 7, 4;

    /// Sync word qualifier value. The actual sync word qualifier value can be greater than 15 but since SYNC_ERROR is only 4 bits wide SYNC_ERROR = FLOOR[actual sync word qualifier value/2] modulo 16. This means that if SYNC_ERROR = 0001b the actual sync word qualifier value is either 2, 3, 34, or 35. When a sync word is received (SYNC_EVENT is asserted) the SYNC_ERROR register field is not updated again before RX mode is re-entered. As long as the radio is in RX searching for a sync word the register field will be updated continuously
    pub sync_error, _: 3, 0;
}

impl Default for PqtSyncErr {
    fn default() -> Self {
        Self(0xff)
    }
}

bitfield! {
    /// Demodulator Status
    ///
    /// # Address
    ///
    /// The address of this register is 0x2F76
    #[derive(Clone, Copy)]
    pub struct DemStatus(u8);

    /// RSSI step found during packet reception (after the assertion of SYNC_EVENT). The RSSI step is 10 or 16 dB and is configured through AGC_CFG1.RSSI_STEP_THR
    ///
    /// # Values
    ///
    /// - 0b: No RSSI step found during packet reception
    /// - 1b: RSSI step found during packet reception
    ///
    /// The default value is 0x00
    pub rssi_step_found, _: 7;

    /// Collision found. Asserted if a new preamble is found and the RSSI has increased 10 or 16 dB during packet reception (depending on AGC_CFG1.RSSI_STEP_THR). MDMCFG1.COLLISION_DETECT_EN must be 1
    ///
    /// # Values
    ///
    /// - 0b: No collision found
    /// - 1b: Collision found
    ///
    /// The default value is 0x00
    pub collision_found, _: 6;

    /// DualSync Detect. Only valid when SYNC_CFG0.SYNC_MODE = 111b. When SYNC_EVENT is asserted this bit can be checked to see which sync word is found
    ///
    /// # Values
    ///
    /// - 0b: Sync word found = [SYNC15_8:SYNC7_0]
    /// - 1b: Sync word found = [SYNC31_24:SYNC23_16)]
    ///
    /// The default value is 0x00
    pub sync_low0_high1, _: 5;

    /// For test purposes only, use values from SmartRF Studio.
    pub dem_status_reserved4_1, _: 4, 1;

    /// Image found detector
    ///
    /// # Values
    ///
    /// - 0b: No image found
    /// - 1b: Image found
    ///
    /// The default value is 0x00
    pub image_found, _: 0;
}

impl Default for DemStatus {
    fn default() -> Self {
        Self(0x00)
    }
}

bitfield! {
    /// Frequency Offset Estimate MSB
    ///
    /// # Address
    ///
    /// The address of this register is 0x2F77
    #[derive(Clone, Copy)]
    pub struct FreqoffEst1(u8);

    /// Frequency offset estimate [15:8] MSB<BR/>
    /// Frequency Offset Estimate = FREOFF_EST*f_xosc/LO Divider/2^18.0 [Hz]. The value is in two's complement format. The LO divider value can be found in FS_CFG.FSD_BANDSELECT register field
    pub freqoff_est_15_8, _: 7, 0;
}

impl Default for FreqoffEst1 {
    fn default() -> Self {
        Self(0x00)
    }
}

bitfield! {
    /// Frequency Offset Estimate LSB
    ///
    /// # Address
    ///
    /// The address of this register is 0x2F78
    #[derive(Clone, Copy)]
    pub struct FreqoffEst0(u8);

    /// See FREQOFF_EST1
    pub freqoff_est_7_0, _: 7, 0;
}

impl Default for FreqoffEst0 {
    fn default() -> Self {
        Self(0x00)
    }
}

bitfield! {
    /// Automatic Gain Control Reg. 3
    ///
    /// # Address
    ///
    /// The address of this register is 0x2F79
    #[derive(Clone, Copy)]
    pub struct AgcGain3(u8);

    pub agc_gain3_not_used, _: 7;

    /// AGC front end gain. Actual applied gain with 1 dB resolution
    pub agc_front_end_gain, _: 6, 0;
}

impl Default for AgcGain3 {
    fn default() -> Self {
        Self(0x00)
    }
}

bitfield! {
    /// Automatic Gain Control Reg. 2
    ///
    /// # Address
    ///
    /// The address of this register is 0x2F7A
    #[derive(Clone, Copy)]
    pub struct AgcGain2(u8);

    /// Override AGC gain control
    ///
    /// # Values
    ///
    /// - 1b: AGC controls front end gain
    /// - 0b: Front end gain controlled by registers AGC_GAIN2, AGC_GAIN1, and AGC_GAIN0
    ///
    /// The default value is 0x01
    pub agc_drives_fe_gain, set_agc_drives_fe_gain: 7;

    /// For test purposes only, use values from SmartRF Studio.
    pub agc_gain2_reserved6_3, set_agc_gain2_reserved6_3: 6, 3;

    /// For test purposes only, use values from SmartRF Studio.
    pub agc_gain2_reserved2_0, set_agc_gain2_reserved2_0: 2, 0;
}

impl Default for AgcGain2 {
    fn default() -> Self {
        Self(0xd1)
    }
}

bitfield! {
    /// Automatic Gain Control Reg. 1
    ///
    /// # Address
    ///
    /// The address of this register is 0x2F7B
    #[derive(Clone, Copy)]
    pub struct AgcGain1(u8);

    pub agc_gain1_not_used, _: 7, 5;

    /// For test purposes only, use values from SmartRF Studio.
    pub agc_gain1_reserved4_3, set_agc_gain1_reserved4_3: 4, 3;

    /// For test purposes only, use values from SmartRF Studio.
    pub agc_gain1_reserved2_0, set_agc_gain1_reserved2_0: 2, 0;
}

impl Default for AgcGain1 {
    fn default() -> Self {
        Self(0x00)
    }
}

bitfield! {
    /// Automatic Gain Control Reg. 0
    ///
    /// # Address
    ///
    /// The address of this register is 0x2F7C
    #[derive(Clone, Copy)]
    pub struct AgcGain0(u8);

    pub agc_gain0_not_used, _: 7;

    /// For test purposes only, use values from SmartRF Studio.
    pub agc_gain0_reserved6_5, set_agc_gain0_reserved6_5: 6, 5;

    /// For test purposes only, use values from SmartRF Studio.
    pub agc_gain0_reserved4_0, set_agc_gain0_reserved4_0: 4, 0;
}

impl Default for AgcGain0 {
    fn default() -> Self {
        Self(0x3f)
    }
}

bitfield! {
    /// Custom Frequency Modulation RX Data
    ///
    /// # Address
    ///
    /// The address of this register is 0x2F7D
    #[derive(Clone, Copy)]
    pub struct CfmRxDataOut(u8);

    /// 8-bit signed soft-decision symbol data, either from normal receiver or transparent receiver. Can be read using burst mode to do custom demodulation<BR/>
    /// f_offset = f_dev*CFM_RX_DATA/64 [Hz] (two's complement format)<BR/>
    /// f_dev is the programmed frequency deviation
    pub cfm_rx_data, _: 7, 0;
}

impl Default for CfmRxDataOut {
    fn default() -> Self {
        Self(0x00)
    }
}

bitfield! {
    /// Custom Frequency Modulation TX Data
    ///
    /// # Address
    ///
    /// The address of this register is 0x2F7E
    #[derive(Clone, Copy)]
    pub struct CfmTxDataIn(u8);

    /// 8-bit signed soft TX data input register for custom SW controlled modulation. Can be accessed using burst mode to get arbitrary modulation<BR/>
    /// f_offset = f_dev*CFM_TX_DATA/64 [Hz] (two's complement format). f_dev is the programmed frequency deviation
    pub cfm_tx_data, set_cfm_tx_data: 7, 0;
}

impl Default for CfmTxDataIn {
    fn default() -> Self {
        Self(0x00)
    }
}

bitfield! {
    /// ASK Soft Decision Output
    ///
    /// # Address
    ///
    /// The address of this register is 0x2F7F
    #[derive(Clone, Copy)]
    pub struct AskSoftRxData(u8);

    pub ask_soft_not_used, _: 7, 6;

    /// The OOK/ASK receiver use a max peak magnitude tracker and low peak magnitude tracker to estimate ASK_THRESHOLD. The ASK_THRESHOLD is used to do hard decision of OOK/ASK symbols<BR/>
    /// ASK_SOFT = +16 when magnitude is = ASK_THRESHOLD<BR/>
    /// ASK_SOFT = -16 when magnitude is = ASK_THRESHOLD
    pub ask_soft, _: 5, 0;
}

impl Default for AskSoftRxData {
    fn default() -> Self {
        Self(0x30)
    }
}

bitfield! {
    /// Random Number Generator Value
    ///
    /// # Address
    ///
    /// The address of this register is 0x2F80
    #[derive(Clone, Copy)]
    pub struct Rndgen(u8);

    /// Random number generator enable
    ///
    /// # Values
    ///
    /// - 0b: Random number generator disabled
    /// - 1b: Random number generator enabled
    ///
    /// The default value is 0x00
    pub rndgen_en, set_rndgen_en: 7;

    /// Random number value. Number generated by 7 bit LFSR register (X7+X6+1). Number will be further randomized when in RX by XORing the feedback with receiver noise
    pub rndgen_value, _: 6, 0;
}

impl Default for Rndgen {
    fn default() -> Self {
        Self(0x7f)
    }
}

bitfield! {
    /// Signal Magnitude after CORDIC [16]
    ///
    /// # Address
    ///
    /// The address of this register is 0x2F81
    #[derive(Clone, Copy)]
    pub struct Magn2(u8);

    pub magn_not_used, _: 7, 1;

    /// Instantaneous signal magnitude after CORDIC, 17-bit [16]
    pub magn_16, _: 0;
}

impl Default for Magn2 {
    fn default() -> Self {
        Self(0x00)
    }
}

bitfield! {
    /// Signal Magnitude after CORDIC [15:8]
    ///
    /// # Address
    ///
    /// The address of this register is 0x2F82
    #[derive(Clone, Copy)]
    pub struct Magn1(u8);

    /// Instantaneous signal magnitude after CORDIC, 17-bit [15:8]
    pub magn_15_8, _: 7, 0;
}

impl Default for Magn1 {
    fn default() -> Self {
        Self(0x00)
    }
}

bitfield! {
    /// Signal Magnitude after CORDIC [7:0]
    ///
    /// # Address
    ///
    /// The address of this register is 0x2F83
    #[derive(Clone, Copy)]
    pub struct Magn0(u8);

    /// Instantaneous signal magnitude after CORDIC, 17-bit [7:0]
    pub magn_7_0, _: 7, 0;
}

impl Default for Magn0 {
    fn default() -> Self {
        Self(0x00)
    }
}

bitfield! {
    /// Signal Angular after CORDIC [9:8]
    ///
    /// # Address
    ///
    /// The address of this register is 0x2F84
    #[derive(Clone, Copy)]
    pub struct Ang1(u8);

    pub ang1_not_used, _: 7, 2;

    /// Instantaneous signal angular after CORDIC, 10-bit [9:8]
    pub angular_9_8, _: 1, 0;
}

impl Default for Ang1 {
    fn default() -> Self {
        Self(0x00)
    }
}

bitfield! {
    /// Signal Angular after CORDIC [7:0]
    ///
    /// # Address
    ///
    /// The address of this register is 0x2F85
    #[derive(Clone, Copy)]
    pub struct Ang0(u8);

    /// Instantaneous signal angular after CORDIC, 10-bit [7:0]
    pub angular_7_0, _: 7, 0;
}

impl Default for Ang0 {
    fn default() -> Self {
        Self(0x00)
    }
}

bitfield! {
    /// Channel Filter Data Real Part [16]
    ///
    /// # Address
    ///
    /// The address of this register is 0x2F86
    #[derive(Clone, Copy)]
    pub struct ChfiltI2(u8);

    pub chfilt_i2_not_used, _: 7, 2;

    ///
    /// # Values
    ///
    /// - 0b: Channel filter data not valid
    /// - 1b: Channel filter data valid (asserted after 16 channel filter samples)
    ///
    /// The default value is 0x01
    pub chfilt_startup_valid, _: 1;

    /// Channel filter data, real part, 17-bit [16]
    pub chfilt_i_16, _: 0;
}

impl Default for ChfiltI2 {
    fn default() -> Self {
        Self(0x02)
    }
}

bitfield! {
    /// Channel Filter Data Real Part [15:8]
    ///
    /// # Address
    ///
    /// The address of this register is 0x2F87
    #[derive(Clone, Copy)]
    pub struct ChfiltI1(u8);

    /// Channel filter data, real part, 17-bit [15:8]
    pub chfilt_i_15_8, _: 7, 0;
}

impl Default for ChfiltI1 {
    fn default() -> Self {
        Self(0x00)
    }
}

bitfield! {
    /// Channel Filter Data Real Part [7:0]
    ///
    /// # Address
    ///
    /// The address of this register is 0x2F88
    #[derive(Clone, Copy)]
    pub struct ChfiltI0(u8);

    /// Channel filter data, real part, 17-bit [7:0]
    pub chfilt_i_7_0, _: 7, 0;
}

impl Default for ChfiltI0 {
    fn default() -> Self {
        Self(0x00)
    }
}

bitfield! {
    /// Channel Filter Data Imaginary Part [16]
    ///
    /// # Address
    ///
    /// The address of this register is 0x2F89
    #[derive(Clone, Copy)]
    pub struct ChfiltQ2(u8);

    pub chfilt_q2_not_used, _: 7, 1;

    /// Channel filter data, imaginary part, 17-bit [16]
    pub chfilt_q_16, _: 0;
}

impl Default for ChfiltQ2 {
    fn default() -> Self {
        Self(0x00)
    }
}

bitfield! {
    /// Channel Filter Data Imaginary Part [15:8]
    ///
    /// # Address
    ///
    /// The address of this register is 0x2F8A
    #[derive(Clone, Copy)]
    pub struct ChfiltQ1(u8);

    /// Channel filter data, imaginary part, 17-bit [15:8]
    pub chfilt_q_15_8, _: 7, 0;
}

impl Default for ChfiltQ1 {
    fn default() -> Self {
        Self(0x00)
    }
}

bitfield! {
    /// Channel Filter Data Imaginary Part [7:0]
    ///
    /// # Address
    ///
    /// The address of this register is 0x2F8B
    #[derive(Clone, Copy)]
    pub struct ChfiltQ0(u8);

    /// Channel filter data, imaginary part, 17-bit [7:0]
    pub chfilt_q_7_0, _: 7, 0;
}

impl Default for ChfiltQ0 {
    fn default() -> Self {
        Self(0x00)
    }
}

bitfield! {
    /// General Purpose Input/Output Status
    ///
    /// # Address
    ///
    /// The address of this register is 0x2F8C
    #[derive(Clone, Copy)]
    pub struct GpioStatus(u8);

    /// For test purposes only, use values from SmartRF Studio.
    pub gpio_status_reserved7_4, _: 7, 4;

    /// State of GPIO pins. SERIAL_STATUS.IOC_SYNC_PINS_EN must be 1
    pub gpio_state, _: 3, 0;
}

impl Default for GpioStatus {
    fn default() -> Self {
        Self(0x00)
    }
}

bitfield! {
    /// Frequency Synthesizer Calibration Control
    ///
    /// # Address
    ///
    /// The address of this register is 0x2F8D
    #[derive(Clone, Copy)]
    pub struct FscalCtrl(u8);

    pub fscal_ctrl_not_used, _: 7;

    /// For test purposes only, use values from SmartRF Studio.
    pub fscal_ctrl_reserved6, set_fscal_ctrl_reserved6: 6;

    /// For test purposes only, use values from SmartRF Studio.
    pub fscal_ctrl_reserved5, set_fscal_ctrl_reserved5: 5;

    /// For test purposes only, use values from SmartRF Studio.
    pub fscal_ctrl_reserved4, set_fscal_ctrl_reserved4: 4;

    /// For test purposes only, use values from SmartRF Studio.
    pub fscal_ctrl_reserved3, set_fscal_ctrl_reserved3: 3;

    /// For test purposes only, use values from SmartRF Studio.
    pub fscal_ctrl_reserved2_1, set_fscal_ctrl_reserved2_1: 2, 1;

    /// Out of lock indicator (FS_CFG.FS_LOCK_EN must be 1). The state of this signal is only valid in RX, TX, and FSTXON state
    ///
    /// # Values
    ///
    /// - 0b: FS is out of lock
    /// - 1b: FS out of lock not detected
    ///
    /// The default value is 0x01
    pub lock, _: 0;
}

impl Default for FscalCtrl {
    fn default() -> Self {
        Self(0x01)
    }
}

bitfield! {
    /// Frequency Synthesizer Phase Adjust
    ///
    /// # Address
    ///
    /// The address of this register is 0x2F8E
    #[derive(Clone, Copy)]
    pub struct PhaseAdjust(u8);

    /// For test purposes only, use values from SmartRF Studio.
    pub phase_adjust_reserved7_0, _: 7, 0;
}

impl Default for PhaseAdjust {
    fn default() -> Self {
        Self(0x00)
    }
}

bitfield! {
    /// Part Number
    ///
    /// # Address
    ///
    /// The address of this register is 0x2F8F
    #[derive(Clone, Copy)]
    pub struct Partnumber(u8);

    /// Chip ID
    ///
    /// # Values
    ///
    /// - 0x20b: CC1200
    /// - 0x21b: CC1201
    ///
    /// The default value is 0x00
    pub partnum, _: 7, 0;
}

impl Default for Partnumber {
    fn default() -> Self {
        Self(0x00)
    }
}

bitfield! {
    /// Part Revision
    ///
    /// # Address
    ///
    /// The address of this register is 0x2F90
    #[derive(Clone, Copy)]
    pub struct Partversion(u8);

    /// Chip revision
    pub partver, _: 7, 0;
}

impl Default for Partversion {
    fn default() -> Self {
        Self(0x00)
    }
}

bitfield! {
    /// Serial Status
    ///
    /// # Address
    ///
    /// The address of this register is 0x2F91
    #[derive(Clone, Copy)]
    pub struct SerialStatus(u8);

    pub serial_status_not_used, _: 7, 6;

    /// Configures which memory to access when using direct memory access
    ///
    /// # Values
    ///
    /// - 0b: FIFO buffers
    /// - 1b: FEC workspace or 128 bytes free area
    ///
    /// The default value is 0x00
    pub spi_direct_access_cfg, set_spi_direct_access_cfg: 5;

    /// Internal 40 kHz RC oscillator clock
    pub clk40, _: 4;

    /// Enable synchronizer for IO pins. Required for transparent TX and for reading GPIO_STATUS.GPIO_STATE
    pub ioc_sync_pins_en, set_ioc_sync_pins_en: 3;

    /// Modulator soft data clock (16 times higher than the programmed symbol rate)
    pub cfm_tx_data_clk, _: 2;

    /// Serial RX data
    pub serial_rx, _: 1;

    /// Serial RX data clock
    pub serial_rx_clk, _: 0;
}

impl Default for SerialStatus {
    fn default() -> Self {
        Self(0x00)
    }
}

bitfield! {
    /// Modem Status Reg. 1
    ///
    /// # Address
    ///
    /// The address of this register is 0x2F92
    #[derive(Clone, Copy)]
    pub struct ModemStatus1(u8);

    /// Asserted simultaneously as SYNC_EVENT. De-asserted when an SRX strobe has been issued
    pub sync_found, _: 7;

    /// Asserted when number of bytes is greater than the RX FIFO threshold. De-asserted when the RX FIFO is empty
    pub rxfifo_full, _: 6;

    /// Asserted when number of bytes is greater than the RX FIFO threshold. De-asserted when the RX FIFO is drained below (or is equal) to the same threshold
    pub rxfifo_thr, _: 5;

    /// High when no bytes reside in the RX FIFO
    pub rxfifo_empty, _: 4;

    /// Asserted when the RX FIFO has overflowed (the radio has received more bytes after the RXFIFO is full). De-asserted when the RX FIFO is flushed
    pub rxfifo_overflow, _: 3;

    /// Asserted if the user try to read from an empty RX FIFO. De-asserted when the RX FIFO is flushed
    pub rxfifo_underflow, _: 2;

    /// Asserted when a preamble is detected (the preamble qualifier value is less than the programmed PQT threshold). The signal will stay asserted as long as a preamble is present but will de-assert on sync found (SYNC_EVENT asserted). If the preamble disappears, the signal will de-assert after a timeout defined by the sync word length + 10 symbols after preamble was lost
    pub pqt_reached, _: 1;

    /// Asserted after 11, 12, 13, 14,1 5, 17, 24, or 32 bits are received (depending on the PREAMBLE_CFG0.PQT_VALID_TIMEOUT setting) or after a preamble is detected
    pub pqt_valid, _: 0;
}

impl Default for ModemStatus1 {
    fn default() -> Self {
        Self(0x01)
    }
}

bitfield! {
    /// Modem Status Reg. 0
    ///
    /// # Address
    ///
    /// The address of this register is 0x2F93
    #[derive(Clone, Copy)]
    pub struct ModemStatus0(u8);

    pub modem_status0_not_used, _: 7;

    /// Internal FEC overflow has occurred
    pub feec_rx_overflow, _: 6;

    /// For test purposes only, use values from SmartRF Studio.
    pub modem_status0_reserved5, _: 5;

    /// Last bit of sync word has been sent
    pub sync_sent, _: 4;

    /// Asserted when the TX FIFO is full. De-asserted when the number of bytes is below threshold
    pub txfifo_full, _: 3;

    /// Asserted when number of bytes is greater than or equal to the TX FIFO threshold
    pub txfifo_thr, _: 2;

    /// Asserted when the TX FIFO has overflowed (The user have tried to write to a full TX FIFO). De-asserted when the TX FIFO is flushed
    pub txfifo_overflow, _: 1;

    /// Asserted when the TX FIFO has underflowed (TX FIFO is empty before the complete packet is sent). De-asserted when the TX FIFO is flushed
    pub txfifo_underflow, _: 0;
}

impl Default for ModemStatus0 {
    fn default() -> Self {
        Self(0x00)
    }
}

bitfield! {
    /// MARC Status Reg. 1
    ///
    /// # Address
    ///
    /// The address of this register is 0x2F94
    #[derive(Clone, Copy)]
    pub struct MarcStatus1(u8);

    /// This register should be read to find what caused the MCU_WAKEUP signal to be asserted
    ///
    /// # Values
    ///
    /// - 00000000b: No failure
    /// - 00000001b: RX timeout occurred
    /// - 00000010b: RX termination based on CS or PQT
    /// - 00000011b: eWOR sync lost (16 slots with no successful reception)
    /// - 00000100b: Packet discarded due to maximum length filtering
    /// - 00000101b: Packet discarded due to address filtering
    /// - 00000110b: Packet discarded due to CRC filtering
    /// - 00000111b: TX FIFO overflow error occurred
    /// - 00001000b: TX FIFO underflow error occurred
    /// - 00001001b: RX FIFO overflow error occurred
    /// - 00001010b: RX FIFO underflow error occurred
    /// - 00001011b: TX ON CCA failed
    /// - 01000000b: TX finished successfully
    /// - 10000000b: RX finished successfully (a packet is in the RX FIFO ready to be read)
    ///
    /// The default value is 0x00
    pub marc_status_out, _: 7, 0;
}

impl Default for MarcStatus1 {
    fn default() -> Self {
        Self(0x00)
    }
}

bitfield! {
    /// MARC Status Reg. 0
    ///
    /// # Address
    ///
    /// The address of this register is 0x2F95
    #[derive(Clone, Copy)]
    pub struct MarcStatus0(u8);

    pub marc_status0_not_used, _: 7, 4;

    /// For test purposes only, use values from SmartRF Studio.
    pub marc_status0_reserved3, _: 3;

    /// This bit can be read after the TXONCCA_DONE signal has been asserted
    ///
    /// # Values
    ///
    /// - 0b: The channel was clear. The radio will enter TX state
    /// - 1b: The channel was busy. The radio will remain in RX state
    ///
    /// The default value is 0x00
    pub txoncca_failed, _: 2;

    /// For test purposes only, use values from SmartRF Studio.
    pub marc_status0_reserved1, _: 1;

    /// RCOSC has been calibrated at least once
    pub rcc_cal_valid, _: 0;
}

impl Default for MarcStatus0 {
    fn default() -> Self {
        Self(0x00)
    }
}

bitfield! {
    /// Power Amplifier Intermediate Frequency Amplifier Test
    ///
    /// # Address
    ///
    /// The address of this register is 0x2F96
    #[derive(Clone, Copy)]
    pub struct PaIfampTest(u8);

    pub pa_ifamp_test_not_used, _: 7, 5;

    /// For test purposes only, use values from SmartRF Studio.
    pub pa_ifamp_test_reserved4, set_pa_ifamp_test_reserved4: 4;

    /// For test purposes only, use values from SmartRF Studio.
    pub pa_ifamp_test_reserved3, set_pa_ifamp_test_reserved3: 3;

    /// For test purposes only, use values from SmartRF Studio.
    pub pa_ifamp_test_reserved2, set_pa_ifamp_test_reserved2: 2;

    /// For test purposes only, use values from SmartRF Studio.
    pub pa_ifamp_test_reserved1, set_pa_ifamp_test_reserved1: 1;

    /// For test purposes only, use values from SmartRF Studio.
    pub pa_ifamp_test_reserved0, set_pa_ifamp_test_reserved0: 0;
}

impl Default for PaIfampTest {
    fn default() -> Self {
        Self(0x00)
    }
}

bitfield! {
    /// Frequency Synthesizer Test
    ///
    /// # Address
    ///
    /// The address of this register is 0x2F97
    #[derive(Clone, Copy)]
    pub struct FsrfTest(u8);

    pub fsrf_test_not_used, _: 7;

    /// For test purposes only, use values from SmartRF Studio.
    pub fsrf_test_reserved6, set_fsrf_test_reserved6: 6;

    /// For test purposes only, use values from SmartRF Studio.
    pub fsrf_test_reserved5_4, set_fsrf_test_reserved5_4: 5, 4;

    /// For test purposes only, use values from SmartRF Studio.
    pub fsrf_test_reserved3, set_fsrf_test_reserved3: 3;

    /// For test purposes only, use values from SmartRF Studio.
    pub fsrf_test_reserved2_0, set_fsrf_test_reserved2_0: 2, 0;
}

impl Default for FsrfTest {
    fn default() -> Self {
        Self(0x00)
    }
}

bitfield! {
    /// Frequency Synthesizer Prescaler Test
    ///
    /// # Address
    ///
    /// The address of this register is 0x2F98
    #[derive(Clone, Copy)]
    pub struct PreTest(u8);

    pub pre_test_not_used, _: 7, 5;

    /// For test purposes only, use values from SmartRF Studio.
    pub pre_test_reserved4, set_pre_test_reserved4: 4;

    /// For test purposes only, use values from SmartRF Studio.
    pub pre_test_reserved3_0, set_pre_test_reserved3_0: 3, 0;
}

impl Default for PreTest {
    fn default() -> Self {
        Self(0x00)
    }
}

bitfield! {
    /// Frequency Synthesizer Prescaler Override
    ///
    /// # Address
    ///
    /// The address of this register is 0x2F99
    #[derive(Clone, Copy)]
    pub struct PreOvr(u8);

    /// For test purposes only, use values from SmartRF Studio.
    pub pre_ovr_reserved7_4, set_pre_ovr_reserved7_4: 7, 4;

    /// For test purposes only, use values from SmartRF Studio.
    pub pre_ovr_reserved3_0, set_pre_ovr_reserved3_0: 3, 0;
}

impl Default for PreOvr {
    fn default() -> Self {
        Self(0x00)
    }
}

bitfield! {
    /// Analog to Digital Converter Test
    ///
    /// # Address
    ///
    /// The address of this register is 0x2F9A
    #[derive(Clone, Copy)]
    pub struct AdcTest(u8);

    pub adc_test_not_used, _: 7, 6;

    /// For test purposes only, use values from SmartRF Studio.
    pub adc_test_reserved5, set_adc_test_reserved5: 5;

    /// For test purposes only, use values from SmartRF Studio.
    pub adc_test_reserved4_0, set_adc_test_reserved4_0: 4, 0;
}

impl Default for AdcTest {
    fn default() -> Self {
        Self(0x00)
    }
}

bitfield! {
    /// Digital Divider Chain Test
    ///
    /// # Address
    ///
    /// The address of this register is 0x2F9B
    #[derive(Clone, Copy)]
    pub struct DvcTest(u8);

    pub dvc_test_not_used, _: 7, 5;

    /// For test purposes only, use values from SmartRF Studio.
    pub dvc_test_reserved4_0, set_dvc_test_reserved4_0: 4, 0;
}

impl Default for DvcTest {
    fn default() -> Self {
        Self(0x0b)
    }
}

bitfield! {
    /// Analog Test
    ///
    /// # Address
    ///
    /// The address of this register is 0x2F9C
    #[derive(Clone, Copy)]
    pub struct Atest(u8);

    pub atest_not_used, _: 7;

    /// For test purposes only, use values from SmartRF Studio.
    pub atest_reserved6, set_atest_reserved6: 6;

    /// For test purposes only, use values from SmartRF Studio.
    pub atest_reserved5_0, set_atest_reserved5_0: 5, 0;
}

impl Default for Atest {
    fn default() -> Self {
        Self(0x40)
    }
}

bitfield! {
    /// Analog Test LVDS
    ///
    /// # Address
    ///
    /// The address of this register is 0x2F9D
    #[derive(Clone, Copy)]
    pub struct AtestLvds(u8);

    pub atest_lvds_not_used, _: 7, 6;

    /// For test purposes only, use values from SmartRF Studio.
    pub atest_lvds_reserved5_4, set_atest_lvds_reserved5_4: 5, 4;

    /// For test purposes only, use values from SmartRF Studio.
    pub atest_lvds_reserved3_0, set_atest_lvds_reserved3_0: 3, 0;
}

impl Default for AtestLvds {
    fn default() -> Self {
        Self(0x00)
    }
}

bitfield! {
    /// Analog Test Mode
    ///
    /// # Address
    ///
    /// The address of this register is 0x2F9E
    #[derive(Clone, Copy)]
    pub struct AtestMode(u8);

    /// For test purposes only, use values from SmartRF Studio.
    pub atest_mode_reserved7_4, set_atest_mode_reserved7_4: 7, 4;

    /// For test purposes only, use values from SmartRF Studio.
    pub atest_mode_reserved3_0, set_atest_mode_reserved3_0: 3, 0;
}

impl Default for AtestMode {
    fn default() -> Self {
        Self(0x00)
    }
}

bitfield! {
    /// Crystal Oscillator Test Reg. 1
    ///
    /// # Address
    ///
    /// The address of this register is 0x2F9F
    #[derive(Clone, Copy)]
    pub struct XoscTest1(u8);

    /// For test purposes only, use values from SmartRF Studio.
    pub xosc_test1_reserved7, set_xosc_test1_reserved7: 7;

    /// For test purposes only, use values from SmartRF Studio.
    pub xosc_test1_reserved6, set_xosc_test1_reserved6: 6;

    /// For test purposes only, use values from SmartRF Studio.
    pub xosc_test1_reserved5_2, _: 5, 2;

    /// For test purposes only, use values from SmartRF Studio.
    pub xosc_test1_reserved1_0, set_xosc_test1_reserved1_0: 1, 0;
}

impl Default for XoscTest1 {
    fn default() -> Self {
        Self(0x3c)
    }
}

bitfield! {
    /// Crystal Oscillator Test Reg. 0
    ///
    /// # Address
    ///
    /// The address of this register is 0x2FA0
    #[derive(Clone, Copy)]
    pub struct XoscTest0(u8);

    /// For test purposes only, use values from SmartRF Studio.
    pub xosc_test0_reserved7_0, set_xosc_test0_reserved7_0: 7, 0;
}

impl Default for XoscTest0 {
    fn default() -> Self {
        Self(0x00)
    }
}

bitfield! {
    /// AES
    ///
    /// # Address
    ///
    /// The address of this register is 0x2FA1
    #[derive(Clone, Copy)]
    pub struct Aes(u8);

    pub aes_not_used, _: 7, 2;

    /// Setting this bit to 1 will abort the AES encryption cycle. The bit will be cleared by HW when the abortion sequence is completed
    pub aes_abort, set_aes_abort: 1;

    /// AES enable. The bit will be cleared by HW when an encryption cycle has finished
    ///
    /// # Values
    ///
    /// - 0b: Halt the current AES encryption
    /// - 1b: AES module is enabled and the AES encryption cycle will start/continue given that AES_ABORT is low
    ///
    /// The default value is 0x00
    pub aes_run, set_aes_run: 0;
}

impl Default for Aes {
    fn default() -> Self {
        Self(0x00)
    }
}

bitfield! {
    /// MODEM Test
    ///
    /// # Address
    ///
    /// The address of this register is 0x2FA2
    #[derive(Clone, Copy)]
    pub struct MdmTest(u8);

    pub mdm_test_not_used, _: 7, 4;

    /// For test purposes only, use values from SmartRF Studio.
    pub mdm_test_reserved3_0, set_mdm_test_reserved3_0: 3, 0;
}

impl Default for MdmTest {
    fn default() -> Self {
        Self(0x00)
    }
}

bitfield! {
    /// RX FIFO Pointer First Entry
    ///
    /// # Address
    ///
    /// The address of this register is 0x2FD2
    #[derive(Clone, Copy)]
    pub struct Rxfirst(u8);

    /// Pointer to the first entry in the RX FIFO
    pub rx_first, set_rx_first: 7, 0;
}

impl Default for Rxfirst {
    fn default() -> Self {
        Self(0x00)
    }
}

bitfield! {
    /// TX FIFO Pointer First Entry
    ///
    /// # Address
    ///
    /// The address of this register is 0x2FD3
    #[derive(Clone, Copy)]
    pub struct Txfirst(u8);

    /// Pointer to the first entry in the TX FIFO
    pub tx_first, set_tx_first: 7, 0;
}

impl Default for Txfirst {
    fn default() -> Self {
        Self(0x00)
    }
}

bitfield! {
    /// RX FIFO Pointer Last Entry
    ///
    /// # Address
    ///
    /// The address of this register is 0x2FD4
    #[derive(Clone, Copy)]
    pub struct Rxlast(u8);

    /// Pointer to the last entry in the RX FIFO
    pub rx_last, set_rx_last: 7, 0;
}

impl Default for Rxlast {
    fn default() -> Self {
        Self(0x00)
    }
}

bitfield! {
    /// TX FIFO Pointer Last Entry
    ///
    /// # Address
    ///
    /// The address of this register is 0x2FD5
    #[derive(Clone, Copy)]
    pub struct Txlast(u8);

    /// Pointer to the last entry in the TX FIFO
    pub tx_last, set_tx_last: 7, 0;
}

impl Default for Txlast {
    fn default() -> Self {
        Self(0x00)
    }
}

bitfield! {
    /// TX FIFO Status
    ///
    /// # Address
    ///
    /// The address of this register is 0x2FD6
    #[derive(Clone, Copy)]
    pub struct NumTxbytes(u8);

    /// Number of bytes in the TX FIFO
    pub txbytes, _: 7, 0;
}

impl Default for NumTxbytes {
    fn default() -> Self {
        Self(0x00)
    }
}

bitfield! {
    /// RX FIFO Status
    ///
    /// # Address
    ///
    /// The address of this register is 0x2FD7
    #[derive(Clone, Copy)]
    pub struct NumRxbytes(u8);

    /// Number of bytes in the RX FIFO
    pub rxbytes, _: 7, 0;
}

impl Default for NumRxbytes {
    fn default() -> Self {
        Self(0x00)
    }
}

bitfield! {
    /// TX FIFO Status
    ///
    /// # Address
    ///
    /// The address of this register is 0x2FD8
    #[derive(Clone, Copy)]
    pub struct FifoNumTxbytes(u8);

    pub fifo_num_txbytes_not_used, _: 7, 4;

    /// Number of free entries in the TX FIFO. 1111b means that there are 15 or more free entries
    pub fifo_txbytes, _: 3, 0;
}

impl Default for FifoNumTxbytes {
    fn default() -> Self {
        Self(0x0f)
    }
}

bitfield! {
    /// RX FIFO Status
    ///
    /// # Address
    ///
    /// The address of this register is 0x2FD9
    #[derive(Clone, Copy)]
    pub struct FifoNumRxbytes(u8);

    pub fifo_num_rxbytes_not_used, _: 7, 4;

    /// Number of available bytes in the RX FIFO. 1111b means that there are 15 or more bytes available to read
    pub fifo_rxbytes, _: 3, 0;
}

impl Default for FifoNumRxbytes {
    fn default() -> Self {
        Self(0x00)
    }
}

bitfield! {
    /// RX FIFO Status
    ///
    /// # Address
    ///
    /// The address of this register is 0x2FDA
    #[derive(Clone, Copy)]
    pub struct RxfifoPreBuf(u8);

    /// Contains the first byte received in the RX FIFO when the RX FIFO is empty (i.e. RXFIRST = RXLAST)
    pub pre_buf, _: 7, 0;
}

impl Default for RxfifoPreBuf {
    fn default() -> Self {
        Self(0x00)
    }
}

bitfield! {
    /// Advanced Encryption Standard Key [127:120]
    ///
    /// # Address
    ///
    /// The address of this register is 0x2FE0
    #[derive(Clone, Copy)]
    pub struct AesKey15(u8);

    /// 16 bytes AES key, [127:120]
    pub aes_key_127_120, set_aes_key_127_120: 7, 0;
}

impl Default for AesKey15 {
    fn default() -> Self {
        Self(0x00)
    }
}

bitfield! {
    /// Advanced Encryption Standard Key [119:112]
    ///
    /// # Address
    ///
    /// The address of this register is 0x2FE1
    #[derive(Clone, Copy)]
    pub struct AesKey14(u8);

    /// 16 bytes AES key, [119:112]
    pub aes_key_119_112, set_aes_key_119_112: 7, 0;
}

impl Default for AesKey14 {
    fn default() -> Self {
        Self(0x00)
    }
}

bitfield! {
    /// Advanced Encryption Standard Key [111:104]
    ///
    /// # Address
    ///
    /// The address of this register is 0x2FE2
    #[derive(Clone, Copy)]
    pub struct AesKey13(u8);

    /// 16 bytes AES key, [111:104]
    pub aes_key_111_104, set_aes_key_111_104: 7, 0;
}

impl Default for AesKey13 {
    fn default() -> Self {
        Self(0x00)
    }
}

bitfield! {
    /// Advanced Encryption Standard Key [103:96]
    ///
    /// # Address
    ///
    /// The address of this register is 0x2FE3
    #[derive(Clone, Copy)]
    pub struct AesKey12(u8);

    /// 16 bytes AES key, [103:96]
    pub aes_key_103_96, set_aes_key_103_96: 7, 0;
}

impl Default for AesKey12 {
    fn default() -> Self {
        Self(0x00)
    }
}

bitfield! {
    /// Advanced Encryption Standard Key [95:88]
    ///
    /// # Address
    ///
    /// The address of this register is 0x2FE4
    #[derive(Clone, Copy)]
    pub struct AesKey11(u8);

    /// 16 bytes AES key, [95:88]
    pub aes_key_95_88, set_aes_key_95_88: 7, 0;
}

impl Default for AesKey11 {
    fn default() -> Self {
        Self(0x00)
    }
}

bitfield! {
    /// Advanced Encryption Standard Key [87:80]
    ///
    /// # Address
    ///
    /// The address of this register is 0x2FE5
    #[derive(Clone, Copy)]
    pub struct AesKey10(u8);

    /// 16 bytes AES key, [87:80]
    pub aes_key_87_80, set_aes_key_87_80: 7, 0;
}

impl Default for AesKey10 {
    fn default() -> Self {
        Self(0x00)
    }
}

bitfield! {
    /// Advanced Encryption Standard Key [79:72]
    ///
    /// # Address
    ///
    /// The address of this register is 0x2FE6
    #[derive(Clone, Copy)]
    pub struct AesKey9(u8);

    /// 16 bytes AES key, [79:72]
    pub aes_key_79_72, set_aes_key_79_72: 7, 0;
}

impl Default for AesKey9 {
    fn default() -> Self {
        Self(0x00)
    }
}

bitfield! {
    /// Advanced Encryption Standard Key [71:64]
    ///
    /// # Address
    ///
    /// The address of this register is 0x2FE7
    #[derive(Clone, Copy)]
    pub struct AesKey8(u8);

    /// 16 bytes AES key, [71:64]
    pub aes_key_71_64, set_aes_key_71_64: 7, 0;
}

impl Default for AesKey8 {
    fn default() -> Self {
        Self(0x00)
    }
}

bitfield! {
    /// Advanced Encryption Standard Key [63:56]
    ///
    /// # Address
    ///
    /// The address of this register is 0x2FE8
    #[derive(Clone, Copy)]
    pub struct AesKey7(u8);

    /// 16 bytes AES key, [63:56]
    pub aes_key_63_56, set_aes_key_63_56: 7, 0;
}

impl Default for AesKey7 {
    fn default() -> Self {
        Self(0x00)
    }
}

bitfield! {
    /// Advanced Encryption Standard Key [55:48]
    ///
    /// # Address
    ///
    /// The address of this register is 0x2FE9
    #[derive(Clone, Copy)]
    pub struct AesKey6(u8);

    /// 16 bytes AES key, [55:48]
    pub aes_key_55_48, set_aes_key_55_48: 7, 0;
}

impl Default for AesKey6 {
    fn default() -> Self {
        Self(0x00)
    }
}

bitfield! {
    /// Advanced Encryption Standard Key [47:40]
    ///
    /// # Address
    ///
    /// The address of this register is 0x2FEA
    #[derive(Clone, Copy)]
    pub struct AesKey5(u8);

    /// 16 bytes AES key, [47:40]
    pub aes_key_47_40, set_aes_key_47_40: 7, 0;
}

impl Default for AesKey5 {
    fn default() -> Self {
        Self(0x00)
    }
}

bitfield! {
    /// Advanced Encryption Standard Key [39:32]
    ///
    /// # Address
    ///
    /// The address of this register is 0x2FEB
    #[derive(Clone, Copy)]
    pub struct AesKey4(u8);

    /// 16 bytes AES key, [39:32]
    pub aes_key_39_32, set_aes_key_39_32: 7, 0;
}

impl Default for AesKey4 {
    fn default() -> Self {
        Self(0x00)
    }
}

bitfield! {
    /// Advanced Encryption Standard Key [31:24]
    ///
    /// # Address
    ///
    /// The address of this register is 0x2FEC
    #[derive(Clone, Copy)]
    pub struct AesKey3(u8);

    /// 16 bytes AES key, [31:24]
    pub aes_key_31_24, set_aes_key_31_24: 7, 0;
}

impl Default for AesKey3 {
    fn default() -> Self {
        Self(0x00)
    }
}

bitfield! {
    /// Advanced Encryption Standard Key [23:16]
    ///
    /// # Address
    ///
    /// The address of this register is 0x2FED
    #[derive(Clone, Copy)]
    pub struct AesKey2(u8);

    /// 16 bytes AES key, [23:16]
    pub aes_key_23_16, set_aes_key_23_16: 7, 0;
}

impl Default for AesKey2 {
    fn default() -> Self {
        Self(0x00)
    }
}

bitfield! {
    /// Advanced Encryption Standard Key [15:8]
    ///
    /// # Address
    ///
    /// The address of this register is 0x2FEE
    #[derive(Clone, Copy)]
    pub struct AesKey1(u8);

    /// 16 bytes AES key, [15:8]
    pub aes_key_15_8, set_aes_key_15_8: 7, 0;
}

impl Default for AesKey1 {
    fn default() -> Self {
        Self(0x00)
    }
}

bitfield! {
    /// Advanced Encryption Standard Key [7:0]
    ///
    /// # Address
    ///
    /// The address of this register is 0x2FEF
    #[derive(Clone, Copy)]
    pub struct AesKey0(u8);

    /// 16 bytes AES key, [7:0]
    pub aes_key_7_0, set_aes_key_7_0: 7, 0;
}

impl Default for AesKey0 {
    fn default() -> Self {
        Self(0x00)
    }
}

bitfield! {
    /// Advanced Encryption Standard Buffer [127:120]
    ///
    /// # Address
    ///
    /// The address of this register is 0x2FF0
    #[derive(Clone, Copy)]
    pub struct AesBuffer15(u8);

    /// AES data buffer [127:120]. The content serves as input to the AES encryption module, and the content will be overwritten with the encrypted data when the AES encryption is completed
    pub aes_buffer_127_120, set_aes_buffer_127_120: 7, 0;
}

impl Default for AesBuffer15 {
    fn default() -> Self {
        Self(0x00)
    }
}

bitfield! {
    /// Advanced Encryption Standard Buffer [119:112]
    ///
    /// # Address
    ///
    /// The address of this register is 0x2FF1
    #[derive(Clone, Copy)]
    pub struct AesBuffer14(u8);

    /// AES data buffer [119:112]. See AES_BUFFER15 for details
    pub aes_buffer_119_112, set_aes_buffer_119_112: 7, 0;
}

impl Default for AesBuffer14 {
    fn default() -> Self {
        Self(0x00)
    }
}

bitfield! {
    /// Advanced Encryption Standard Buffer [111:104]
    ///
    /// # Address
    ///
    /// The address of this register is 0x2FF2
    #[derive(Clone, Copy)]
    pub struct AesBuffer13(u8);

    /// AES data buffer [111:104]. See AES_BUFFER15 for details
    pub aes_buffer_111_104, set_aes_buffer_111_104: 7, 0;
}

impl Default for AesBuffer13 {
    fn default() -> Self {
        Self(0x00)
    }
}

bitfield! {
    /// Advanced Encryption Standard Buffer [103:93]
    ///
    /// # Address
    ///
    /// The address of this register is 0x2FF3
    #[derive(Clone, Copy)]
    pub struct AesBuffer12(u8);

    /// AES data buffer [103:93]. See AES_BUFFER15 for details
    pub aes_buffer_103_93, set_aes_buffer_103_93: 7, 0;
}

impl Default for AesBuffer12 {
    fn default() -> Self {
        Self(0x00)
    }
}

bitfield! {
    /// Advanced Encryption Standard Buffer [95:88]
    ///
    /// # Address
    ///
    /// The address of this register is 0x2FF4
    #[derive(Clone, Copy)]
    pub struct AesBuffer11(u8);

    /// AES data buffer [95:88]. See AES_BUFFER15 for details
    pub aes_buffer_95_88, set_aes_buffer_95_88: 7, 0;
}

impl Default for AesBuffer11 {
    fn default() -> Self {
        Self(0x00)
    }
}

bitfield! {
    /// Advanced Encryption Standard Buffer [87:80]
    ///
    /// # Address
    ///
    /// The address of this register is 0x2FF5
    #[derive(Clone, Copy)]
    pub struct AesBuffer10(u8);

    /// AES data buffer [87:80]. See AES_BUFFER15 for details
    pub aes_buffer_87_80, set_aes_buffer_87_80: 7, 0;
}

impl Default for AesBuffer10 {
    fn default() -> Self {
        Self(0x00)
    }
}

bitfield! {
    /// Advanced Encryption Standard Buffer [79:72]
    ///
    /// # Address
    ///
    /// The address of this register is 0x2FF6
    #[derive(Clone, Copy)]
    pub struct AesBuffer9(u8);

    /// AES data buffer [79:72]. See AES_BUFFER15 for details
    pub aes_buffer_79_72, set_aes_buffer_79_72: 7, 0;
}

impl Default for AesBuffer9 {
    fn default() -> Self {
        Self(0x00)
    }
}

bitfield! {
    /// Advanced Encryption Standard Buffer [71:64]
    ///
    /// # Address
    ///
    /// The address of this register is 0x2FF7
    #[derive(Clone, Copy)]
    pub struct AesBuffer8(u8);

    /// AES data buffer [71:64]. See AES_BUFFER15 for details
    pub aes_buffer_71_64, set_aes_buffer_71_64: 7, 0;
}

impl Default for AesBuffer8 {
    fn default() -> Self {
        Self(0x00)
    }
}

bitfield! {
    /// Advanced Encryption Standard Buffer [63:56]
    ///
    /// # Address
    ///
    /// The address of this register is 0x2FF8
    #[derive(Clone, Copy)]
    pub struct AesBuffer7(u8);

    /// AES data buffer [63:56]. See AES_BUFFER15 for details
    pub aes_buffer_63_56, set_aes_buffer_63_56: 7, 0;
}

impl Default for AesBuffer7 {
    fn default() -> Self {
        Self(0x00)
    }
}

bitfield! {
    /// Advanced Encryption Standard Buffer [55:48]
    ///
    /// # Address
    ///
    /// The address of this register is 0x2FF9
    #[derive(Clone, Copy)]
    pub struct AesBuffer6(u8);

    /// AES data buffer [55:48]. See AES_BUFFER15 for details
    pub aes_buffer_55_48, set_aes_buffer_55_48: 7, 0;
}

impl Default for AesBuffer6 {
    fn default() -> Self {
        Self(0x00)
    }
}

bitfield! {
    /// Advanced Encryption Standard Buffer [47:40]
    ///
    /// # Address
    ///
    /// The address of this register is 0x2FFA
    #[derive(Clone, Copy)]
    pub struct AesBuffer5(u8);

    /// AES data buffer [47:40]. See AES_BUFFER15 for details
    pub aes_buffer_47_40, set_aes_buffer_47_40: 7, 0;
}

impl Default for AesBuffer5 {
    fn default() -> Self {
        Self(0x00)
    }
}

bitfield! {
    /// Advanced Encryption Standard Buffer [39:32]
    ///
    /// # Address
    ///
    /// The address of this register is 0x2FFB
    #[derive(Clone, Copy)]
    pub struct AesBuffer4(u8);

    /// AES data buffer [39:32]. See AES_BUFFER15 for details
    pub aes_buffer_39_32, set_aes_buffer_39_32: 7, 0;
}

impl Default for AesBuffer4 {
    fn default() -> Self {
        Self(0x00)
    }
}

bitfield! {
    /// Advanced Encryption Standard Buffer [31:24]
    ///
    /// # Address
    ///
    /// The address of this register is 0x2FFC
    #[derive(Clone, Copy)]
    pub struct AesBuffer3(u8);

    /// AES data buffer [131:24]. See AES_BUFFER15 for details
    pub aes_buffer_31_24, set_aes_buffer_31_24: 7, 0;
}

impl Default for AesBuffer3 {
    fn default() -> Self {
        Self(0x00)
    }
}

bitfield! {
    /// Advanced Encryption Standard Buffer [23:16]
    ///
    /// # Address
    ///
    /// The address of this register is 0x2FFD
    #[derive(Clone, Copy)]
    pub struct AesBuffer2(u8);

    /// AES data buffer [23:16]. See AES_BUFFER15 for details
    pub aes_buffer_23_16, set_aes_buffer_23_16: 7, 0;
}

impl Default for AesBuffer2 {
    fn default() -> Self {
        Self(0x00)
    }
}

bitfield! {
    /// Advanced Encryption Standard Buffer [15:8]
    ///
    /// # Address
    ///
    /// The address of this register is 0x2FFE
    #[derive(Clone, Copy)]
    pub struct AesBuffer1(u8);

    /// AES data buffer [15:8]. See AES_BUFFER15 for details
    pub aes_buffer_15_8, set_aes_buffer_15_8: 7, 0;
}

impl Default for AesBuffer1 {
    fn default() -> Self {
        Self(0x00)
    }
}

bitfield! {
    /// Advanced Encryption Standard Buffer [7:0]
    ///
    /// # Address
    ///
    /// The address of this register is 0x2FFF
    #[derive(Clone, Copy)]
    pub struct AesBuffer0(u8);

    /// AES data buffer [7:0]. See AES_BUFFER15 for details
    pub aes_buffer_7_0, set_aes_buffer_7_0: 7, 0;
}

impl Default for AesBuffer0 {
    fn default() -> Self {
        Self(0x00)
    }
}
