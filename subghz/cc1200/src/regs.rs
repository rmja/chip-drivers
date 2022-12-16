use core::mem::transmute;

use bitfield::bitfield;

use crate::gpio::{Gpio3Output, Gpio2Output, Gpio1Output, Gpio0Output};

bitfield! {
    #[derive(Clone, Copy)]
    pub struct Iocfg3(u8);
    pub gpio3_atran, set_gpio3_atran: 7;
    pub gpio3_inv, set_gpio3_inv: 6;
    gpio3_cfg_bits, set_gpio3_cfg: 5, 0;
}

impl Iocfg3 {
    pub fn gpio3_cfg(&self) -> Gpio3Output {
        unsafe { transmute(self.gpio3_cfg_bits()) }
    }
}

bitfield! {
    #[derive(Clone, Copy)]
    pub struct Iocfg2(u8);
    pub gpio2_atran, set_gpio2_atran: 7;
    pub gpio2_inv, set_gpio2_inv: 6;
    gpio2_cfg_bits, set_gpio2_cfg: 5, 0;
}

impl Iocfg2 {
    pub fn gpio2_cfg(&self) -> Gpio2Output {
        unsafe { transmute(self.gpio2_cfg_bits()) }
    }
}

bitfield! {
    #[derive(Clone, Copy)]
    pub struct Iocfg1(u8);
    pub gpio1_atran, set_gpio1_atran: 7;
    pub gpio1_inv, set_gpio1_inv: 6;
    gpio1_cfg_bits, set_gpio1_cfg: 5, 0;
}

impl Iocfg1 {
    pub fn gpio1_cfg(&self) -> Gpio1Output {
        unsafe { transmute(self.gpio1_cfg_bits()) }
    }
}

bitfield! {
    #[derive(Clone, Copy)]
    pub struct Iocfg0(u8);
    pub gpio0_atran, set_gpio0_atran: 7;
    pub gpio0_inv, set_gpio0_inv: 6;
    gpio0_cfg_bits, set_gpio0_cfg: 5, 0;
}

impl Iocfg0 {
    pub fn gpio0_cfg(&self) -> Gpio0Output {
        unsafe { transmute(self.gpio0_cfg_bits()) }
    }
}

bitfield! {
    #[derive(Clone, Copy)]
    pub struct Sync3(u8);
    pub sync31_24, set_sync31_24: 7, 0;
}

bitfield! {
    #[derive(Clone, Copy)]
    pub struct Sync2(u8);
    pub sync23_16, set_sync23_16: 7, 0;
}

bitfield! {
    #[derive(Clone, Copy)]
    pub struct Sync1(u8);
    pub sync15_8, set_sync15_8: 7, 0;
}

bitfield! {
    #[derive(Clone, Copy)]
    pub struct Sync0(u8);
    pub sync7_0, set_sync7_0: 7, 0;
}

bitfield! {
    #[derive(Clone, Copy)]
    pub struct SyncCfg1(u8);
    pub sync_mode, set_sync_mode: 7, 5;
    pub sync_thr, set_sync_thr: 4, 0;
}

bitfield! {
    #[derive(Clone, Copy)]
    pub struct SyncCfg0(u8);
    not_used, _: 7, 6;
    pub auto_clear, set_auto_clear: 5;
    pub rx_config_limitation, set_rx_config_limitation: 4;
    pub pqt_gating_en, set_pqt_gating_en: 3;
    pub ext_sync_detect, set_ext_sync_detect: 2;
    pub strict_sync_check, set_strict_sync_check: 1, 0;
}

bitfield! {
    #[derive(Clone, Copy)]
    pub struct DeviationM(u8);
    pub dev_m, set_dev_m: 7, 0;
}

bitfield! {
    #[derive(Clone, Copy)]
    pub struct ModcfgDevE(u8);
    pub modem_mode, set_modem_mode: 7, 6;
    pub mod_format, set_mod_format: 5, 3;
    pub dev_e, set_dev_e: 2, 0;
}

bitfield! {
    #[derive(Clone, Copy)]
    pub struct DcfiltCfg(u8);
    not_used, _: 7;
    pub dcfilt_freeze_coeff, set_dcfilt_freeze_coeff: 6;
    pub dcfilt_bw_settle, set_dcfilt_bw_settle: 5, 3;
    pub dcfilt_bw, set_dcfilt_bw: 2, 0;
}

bitfield! {
    #[derive(Clone, Copy)]
    pub struct PreambleCfg1(u8);
    not_used, _: 7, 6;
    pub num_preamble, set_num_preamble: 5, 2;
    pub preamble_word, set_preamble_word: 1, 0;
}

bitfield! {
    #[derive(Clone, Copy)]
    pub struct PreambleCfg0(u8);
    pub pqt_en, set_pqt_en: 7;
    pub pqt_valid_timeout, set_pqt_valid_timeout: 6, 4;
    pub pqt, set_pqt: 3, 0;
}

bitfield! {
    #[derive(Clone, Copy)]
    pub struct Iqic(u8);
    pub iqic_en, set_iqic_en: 7;
    pub iqic_update_coeff_en, set_iqic_update_coeff_en: 6;
    pub iqic_blen_settle, set_iqic_blen_settle: 5, 4;
    pub iqic_blen, set_iqic_blen: 3, 2;
    pub iqic_imgch_level_thr, set_iqic_imgch_level_thr: 1, 0;
}

bitfield! {
    #[derive(Clone, Copy)]
    pub struct ChanBw(u8);
    pub adc_cic_decfact, set_adc_cic_decfact: 7, 6;
    pub bb_cic_decfact, set_bb_cic_decfact: 5, 0;
}

bitfield! {
    #[derive(Clone, Copy)]
    pub struct Mdmcfg2(u8);
    pub ask_shape, set_ask_shape: 7, 6;
    pub symbol_map_cfg, set_symbol_map_cfg: 5, 4;
    pub upsampler_p, set_upsampler_p: 3, 1;
    pub cfm_data_en, set_cfm_data_en: 0;
}

bitfield! {
    #[derive(Clone, Copy)]
    pub struct Mdmcfg1(u8);
    pub carrier_sense_gate, set_carrier_sense_gate: 7;
    pub fifo_en, set_fifo_en: 6;
    pub manchester_en, set_manchester_en: 5;
    pub invert_data_en, set_invert_data_en: 4;
    pub collision_detect_en, set_collision_detect_en: 3;
    pub dvga_gain, set_dvga_gain: 2, 1;
    pub single_adc_en, set_single_adc_en: 0;
}

bitfield! {
    #[derive(Clone, Copy)]
    pub struct Mdmcfg0(u8);
    pub reserved7, set_reserved7: 7;
    pub transparent_mode_en, set_transparent_mode_en: 6;
    pub transparent_intfact, set_transparent_intfact: 5, 4;
    pub data_filter_en, set_data_filter_en: 3;
    pub viterbi_en, set_viterbi_en: 2;
    pub reserved1_0, set_reserved1_0: 1, 0;
}

bitfield! {
    #[derive(Clone, Copy)]
    pub struct SymbolRate2(u8);
    pub srate_e, set_srate_e: 7, 4;
    pub srate_m_19_16, set_srate_m_19_16: 3, 0;
}

bitfield! {
    #[derive(Clone, Copy)]
    pub struct SymbolRate1(u8);
    pub srate_m_15_8, set_srate_m_15_8: 7, 0;
}

bitfield! {
    #[derive(Clone, Copy)]
    pub struct SymbolRate0(u8);
    pub srate_m_7_0, set_srate_m_7_0: 7, 0;
}

bitfield! {
    #[derive(Clone, Copy)]
    pub struct AgcRef(u8);
    pub agc_reference, set_agc_reference: 7, 0;
}

bitfield! {
    #[derive(Clone, Copy)]
    pub struct AgcCsThr(u8);
    pub agc_cs_th, set_agc_cs_th: 7, 0;
}

bitfield! {
    #[derive(Clone, Copy)]
    pub struct AgcGainAdjust(u8);
    pub gain_adjustment, set_gain_adjustment: 7, 0;
}

bitfield! {
    #[derive(Clone, Copy)]
    pub struct AgcCfg3(u8);
    pub agc_sync_behaviour, set_agc_sync_behaviour: 7, 5;
    pub agc_min_gain, set_agc_min_gain: 4, 0;
}

bitfield! {
    #[derive(Clone, Copy)]
    pub struct AgcCfg2(u8);
    pub start_previous_gain_en, set_start_previous_gain_en: 7;
    pub fe_performance_mode, set_fe_performance_mode: 6, 5;
    pub agc_max_gain, set_agc_max_gain: 4, 0;
}

bitfield! {
    #[derive(Clone, Copy)]
    pub struct AgcCfg1(u8);
    not_used, _: 7;
    pub rssi_step_thr, set_rssi_step_thr: 6;
    pub agc_win_size, set_agc_win_size: 5, 3;
    pub agc_settle_wait, set_agc_settle_wait: 2, 0;
}

bitfield! {
    #[derive(Clone, Copy)]
    pub struct AgcCfg0(u8);
    pub agc_hyst_level, set_agc_hyst_level: 7, 6;
    pub agc_slewrate_limit, set_agc_slewrate_limit: 5, 4;
    pub rssi_valid_cnt, set_rssi_valid_cnt: 3, 2;
    pub agc_ask_decay, set_agc_ask_decay: 1, 0;
}

bitfield! {
    #[derive(Clone, Copy)]
    pub struct FifoCfg(u8);
    pub crc_autoflush, set_crc_autoflush: 7;
    pub fifo_thr, set_fifo_thr: 6, 0;
}

bitfield! {
    #[derive(Clone, Copy)]
    pub struct DevAddr(u8);
    pub device_addr, set_device_addr: 7, 0;
}

bitfield! {
    #[derive(Clone, Copy)]
    pub struct SettlingCfg(u8);
    not_used, _: 7, 5;
    pub fs_autocal, set_fs_autocal: 4, 3;
    pub lock_time, set_lock_time: 2, 1;
    pub fsreg_time, set_fsreg_time: 0;
}

bitfield! {
    #[derive(Clone, Copy)]
    pub struct FsCfg(u8);
    not_used, _: 7, 5;
    pub fs_lock_en, set_fs_lock_en: 4;
    pub fsd_bandselect, set_fsd_bandselect: 3, 0;
}

bitfield! {
    #[derive(Clone, Copy)]
    pub struct WorCfg1(u8);
    pub wor_res, set_wor_res: 7, 6;
    pub wor_mode, set_wor_mode: 5, 3;
    pub event1, set_event1: 2, 0;
}

bitfield! {
    #[derive(Clone, Copy)]
    pub struct WorCfg0(u8);
    pub rx_duty_cycle_mode, _: 7, 6;
    pub div_256hz_en, set_div_256hz_en: 5;
    pub event2_cfg, set_event2_cfg: 4, 3;
    pub rc_mode, set_rc_mode: 2, 1;
    pub rc_pd, set_rc_pd: 0;
}

bitfield! {
    #[derive(Clone, Copy)]
    pub struct WorEvent0Msb(u8);
    pub event0_15_8, set_event0_15_8: 7, 0;
}

bitfield! {
    #[derive(Clone, Copy)]
    pub struct WorEvent0Lsb(u8);
    pub event0_7_0, set_event0_7_0: 7, 0;
}

bitfield! {
    #[derive(Clone, Copy)]
    pub struct RxdcmTime(u8);
    pub rx_duty_cycle_time, set_rx_duty_cycle_time: 7, 0;
}

bitfield! {
    #[derive(Clone, Copy)]
    pub struct PktCfg2(u8);
    not_used, _: 7;
    pub byte_swap_en, set_byte_swap_en: 6;
    pub fg_mode_en, set_fg_mode_en: 5;
    pub cca_mode, set_cca_mode: 4, 2;
    pub pkt_format, set_pkt_format: 1, 0;
}

bitfield! {
    #[derive(Clone, Copy)]
    pub struct PktCfg1(u8);
    pub fec_en, set_fec_en: 7;
    pub white_data, set_white_data: 6;
    pub pn9_swap_en, set_pn9_swap_en: 5;
    pub addr_check_cfg, set_addr_check_cfg: 4, 3;
    pub crc_cfg, set_crc_cfg: 2, 1;
    pub append_status, set_append_status: 0;
}

bitfield! {
    #[derive(Clone, Copy)]
    pub struct PktCfg0(u8);
    pub reserved7, set_reserved7: 7;
    pub length_config, set_length_config: 6, 5;
    pub pkt_bit_len, set_pkt_bit_len: 4, 2;
    pub uart_mode_en, set_uart_mode_en: 1;
    pub uart_swap_en, set_uart_swap_en: 0;
}

bitfield! {
    #[derive(Clone, Copy)]
    pub struct RfendCfg1(u8);
    not_used, _: 7, 6;
    pub rxoff_mode, set_rxoff_mode: 5, 4;
    pub rx_time, set_rx_time: 3, 1;
    pub rx_time_qual, set_rx_time_qual: 0;
}

bitfield! {
    #[derive(Clone, Copy)]
    pub struct RfendCfg0(u8);
    not_used, _: 7;
    pub cal_end_wake_up_en, set_cal_end_wake_up_en: 6;
    pub txoff_mode, set_txoff_mode: 5, 4;
    pub term_on_bad_packet_en, set_term_on_bad_packet_en: 3;
    pub ant_div_rx_term_cfg, set_ant_div_rx_term_cfg: 2, 0;
}

bitfield! {
    #[derive(Clone, Copy)]
    pub struct PaCfg1(u8);
    not_used, _: 7;
    pub pa_ramp_shape_en, set_pa_ramp_shape_en: 6;
    pub pa_power_ramp, set_pa_power_ramp: 5, 0;
}

bitfield! {
    #[derive(Clone, Copy)]
    pub struct PaCfg0(u8);
    pub first_ipl, set_first_ipl: 7, 5;
    pub second_ipl, set_second_ipl: 4, 2;
    pub ramp_shape, set_ramp_shape: 1, 0;
}

bitfield! {
    #[derive(Clone, Copy)]
    pub struct AskCfg(u8);
    pub agc_ask_bw, set_agc_ask_bw: 7, 6;
    pub ask_depth, set_ask_depth: 5, 0;
}

bitfield! {
    #[derive(Clone, Copy)]
    pub struct PktLen(u8);
    pub packet_length, set_packet_length: 7, 0;
}

bitfield! {
    #[derive(Clone, Copy)]
    pub struct IfMixCfg(u8);
    not_used, _: 7, 5;
    pub cmix_cfg, set_cmix_cfg: 4, 2;
    pub reserved1_0, set_reserved1_0: 1, 0;
}

bitfield! {
    #[derive(Clone, Copy)]
    pub struct FreqoffCfg(u8);
    not_used, _: 7, 6;
    pub foc_en, set_foc_en: 5;
    pub foc_cfg, set_foc_cfg: 4, 3;
    pub foc_limit, set_foc_limit: 2;
    pub foc_ki_factor, set_foc_ki_factor: 1, 0;
}

bitfield! {
    #[derive(Clone, Copy)]
    pub struct TocCfg(u8);
    pub toc_limit, set_toc_limit: 7, 6;
    pub toc_pre_sync_blocklen, set_toc_pre_sync_blocklen: 5, 3;
    pub toc_post_sync_blocklen, set_toc_post_sync_blocklen: 2, 0;
}

bitfield! {
    #[derive(Clone, Copy)]
    pub struct MarcSpare(u8);
    not_used, _: 7, 4;
    pub aes_commands, set_aes_commands: 3, 0;
}

bitfield! {
    #[derive(Clone, Copy)]
    pub struct EcgCfg(u8);
    not_used, _: 7, 5;
    pub ext_clock_freq, set_ext_clock_freq: 4, 0;
}

bitfield! {
    #[derive(Clone, Copy)]
    pub struct ExtCtrl(u8);
    not_used, _: 7, 3;
    pub pin_ctrl_en, set_pin_ctrl_en: 2;
    pub ext_40k_clock_en, set_ext_40k_clock_en: 1;
    pub burst_addr_incr_en, set_burst_addr_incr_en: 0;
}

bitfield! {
    #[derive(Clone, Copy)]
    pub struct RccalFine(u8);
    not_used, _: 7;
    pub rcc_fine, set_rcc_fine: 6, 0;
}

bitfield! {
    #[derive(Clone, Copy)]
    pub struct RccalCourse(u8);
    not_used, _: 7;
    pub rcc_course, set_rcc_course: 6, 0;
}

bitfield! {
    #[derive(Clone, Copy)]
    pub struct RccalOffset(u8);
    not_used, _: 7, 5;
    pub reserved4_0, set_reserved4_0: 4, 0;
}

bitfield! {
    #[derive(Clone, Copy)]
    pub struct Freqoff1(u8);
    pub freq_off_15_8, set_freq_off_15_8: 7, 0;
}

bitfield! {
    #[derive(Clone, Copy)]
    pub struct Freqoff0(u8);
    pub freq_off_7_0, set_freq_off_7_0: 7, 0;
}

bitfield! {
    #[derive(Clone, Copy)]
    pub struct Freq2(u8);
    pub freq_23_16, set_freq_23_16: 7, 0;
}

bitfield! {
    #[derive(Clone, Copy)]
    pub struct Freq1(u8);
    pub freq_15_8, set_freq_15_8: 7, 0;
}

bitfield! {
    #[derive(Clone, Copy)]
    pub struct Freq0(u8);
    pub freq_7_0, set_freq_7_0: 7, 0;
}

bitfield! {
    #[derive(Clone, Copy)]
    pub struct IfAdc2(u8);
    not_used, _: 7, 4;
    pub reserved3_0, set_reserved3_0: 3, 0;
}

bitfield! {
    #[derive(Clone, Copy)]
    pub struct IfAdc1(u8);
    pub reserved7_0, set_reserved7_0: 7, 0;
}

bitfield! {
    #[derive(Clone, Copy)]
    pub struct IfAdc0(u8);
    not_used, _: 7, 6;
    pub reserved5_0, set_reserved5_0: 5, 0;
}

bitfield! {
    #[derive(Clone, Copy)]
    pub struct FsDig1(u8);
    not_used, _: 7, 6;
    pub reserved5_0, set_reserved5_0: 5, 0;
}

bitfield! {
    #[derive(Clone, Copy)]
    pub struct FsDig0(u8);
    pub reserved7_4, set_reserved7_4: 7, 4;
    pub rx_lpf_bw, set_rx_lpf_bw: 3, 2;
    pub tx_lpf_bw, set_tx_lpf_bw: 1, 0;
}

bitfield! {
    #[derive(Clone, Copy)]
    pub struct FsCal3(u8);
    pub fs_cal3_reserved7, set_fs_cal3_reserved7: 7;
    pub kvco_high_res_cfg, set_kvco_high_res_cfg: 6;
    pub reserved5_0, set_reserved5_0: 5, 0;
}

bitfield! {
    #[derive(Clone, Copy)]
    pub struct FsCal2(u8);
    not_used, _: 7, 6;
    pub reserved5_0, set_reserved5_0: 5, 0;
}

bitfield! {
    #[derive(Clone, Copy)]
    pub struct FsCal1(u8);
    pub reserved7_0, set_reserved7_0: 7, 0;
}

bitfield! {
    #[derive(Clone, Copy)]
    pub struct FsCal0(u8);
    not_used, _: 7, 4;
    pub lock_cfg, set_lock_cfg: 3, 2;
    pub reserved1_0, set_reserved1_0: 1, 0;
}

bitfield! {
    #[derive(Clone, Copy)]
    pub struct FsChp(u8);
    not_used, _: 7, 6;
    pub reserved5_0, set_reserved5_0: 5, 0;
}

bitfield! {
    #[derive(Clone, Copy)]
    pub struct FsDivtwo(u8);
    not_used, _: 7, 2;
    pub reserved1_0, set_reserved1_0: 1, 0;
}

bitfield! {
    #[derive(Clone, Copy)]
    pub struct FsDsm1(u8);
    not_used, _: 7, 3;
    pub reserved2_0, set_reserved2_0: 2, 0;
}

bitfield! {
    #[derive(Clone, Copy)]
    pub struct FsDsm0(u8);
    pub reserved7_0, set_reserved7_0: 7, 0;
}

bitfield! {
    #[derive(Clone, Copy)]
    pub struct FsDvc1(u8);
    pub reserved7_0, set_reserved7_0: 7, 0;
}

bitfield! {
    #[derive(Clone, Copy)]
    pub struct FsDvc0(u8);
    not_used, _: 7, 5;
    pub reserved4_0, set_reserved4_0: 4, 0;
}

bitfield! {
    #[derive(Clone, Copy)]
    pub struct FsLbi(u8);
    not_used, _: 7, 0;
}

bitfield! {
    #[derive(Clone, Copy)]
    pub struct FsPfd(u8);
    not_used, _: 7;
    pub reserved6_0, set_reserved6_0: 6, 0;
}

bitfield! {
    #[derive(Clone, Copy)]
    pub struct FsPre(u8);
    not_used, _: 7;
    pub reserved6_0, set_reserved6_0: 6, 0;
}

bitfield! {
    #[derive(Clone, Copy)]
    pub struct FsRegDivCml(u8);
    not_used, _: 7, 5;
    pub reserved4_0, set_reserved4_0: 4, 0;
}

bitfield! {
    #[derive(Clone, Copy)]
    pub struct FsSpare(u8);
    pub reserved7_0, set_reserved7_0: 7, 0;
}

bitfield! {
    #[derive(Clone, Copy)]
    pub struct FsVco4(u8);
    not_used, _: 7, 5;
    pub reserved4_0, set_reserved4_0: 4, 0;
}

bitfield! {
    #[derive(Clone, Copy)]
    pub struct FsVco3(u8);
    not_used, _: 7, 1;
    pub reserved0, set_reserved0: 0;
}

bitfield! {
    #[derive(Clone, Copy)]
    pub struct FsVco2(u8);
    not_used, _: 7;
    pub reserved6_0, set_reserved6_0: 6, 0;
}

bitfield! {
    #[derive(Clone, Copy)]
    pub struct FsVco1(u8);
    pub fsd_vcdac, set_fsd_vcdac: 7, 2;
    pub reserved1_0, set_reserved1_0: 1, 0;
}

bitfield! {
    #[derive(Clone, Copy)]
    pub struct FsVco0(u8);
    pub reserved7_0, set_reserved7_0: 7, 0;
}

bitfield! {
    #[derive(Clone, Copy)]
    pub struct Gbias6(u8);
    not_used, _: 7, 6;
    pub reserved5_0, set_reserved5_0: 5, 0;
}

bitfield! {
    #[derive(Clone, Copy)]
    pub struct Gbias5(u8);
    not_used, _: 7, 4;
    pub reserved3_0, set_reserved3_0: 3, 0;
}

bitfield! {
    #[derive(Clone, Copy)]
    pub struct Gbias4(u8);
    not_used, _: 7, 6;
    pub reserved5_0, set_reserved5_0: 5, 0;
}

bitfield! {
    #[derive(Clone, Copy)]
    pub struct Gbias3(u8);
    not_used, _: 7, 6;
    pub reserved5_0, set_reserved5_0: 5, 0;
}

bitfield! {
    #[derive(Clone, Copy)]
    pub struct Gbias2(u8);
    not_used, _: 7;
    pub reserved6_0, set_reserved6_0: 6, 0;
}

bitfield! {
    #[derive(Clone, Copy)]
    pub struct Gbias1(u8);
    not_used, _: 7, 5;
    pub reserved4_0, set_reserved4_0: 4, 0;
}

bitfield! {
    #[derive(Clone, Copy)]
    pub struct Gbias0(u8);
    not_used, _: 7, 2;
    pub reserved1_0, set_reserved1_0: 1, 0;
}

bitfield! {
    #[derive(Clone, Copy)]
    pub struct Ifamp(u8);
    not_used, _: 7, 4;
    pub ifamp_bw, set_ifamp_bw: 3, 2;
    pub reserved1_0, set_reserved1_0: 1, 0;
}

bitfield! {
    #[derive(Clone, Copy)]
    pub struct Lna(u8);
    not_used, _: 7, 2;
    pub reserved1_0, set_reserved1_0: 1, 0;
}

bitfield! {
    #[derive(Clone, Copy)]
    pub struct Rxmix(u8);
    not_used, _: 7, 2;
    pub reserved1_0, set_reserved1_0: 1, 0;
}

bitfield! {
    #[derive(Clone, Copy)]
    pub struct Xosc5(u8);
    not_used, _: 7, 4;
    pub reserved3_0, set_reserved3_0: 3, 0;
}

bitfield! {
    #[derive(Clone, Copy)]
    pub struct Xosc4(u8);
    pub reserved7_0, set_reserved7_0: 7, 0;
}

bitfield! {
    #[derive(Clone, Copy)]
    pub struct Xosc3(u8);
    pub reserved7_0, set_reserved7_0: 7, 0;
}

bitfield! {
    #[derive(Clone, Copy)]
    pub struct Xosc2(u8);
    not_used, _: 7, 4;
    pub reserved3_1, set_reserved3_1: 3, 1;
    pub xosc_core_pd_override, set_xosc_core_pd_override: 0;
}

bitfield! {
    #[derive(Clone, Copy)]
    pub struct Xosc1(u8);
    not_used, _: 6, 3;
    pub reserved2, set_reserved2: 2;
    pub xosc_buf_sel, set_xosc_buf_sel: 1;
    pub xosc_stable, _: 0;
}

bitfield! {
    #[derive(Clone, Copy)]
    pub struct Xosc0(u8);
    not_used, _: 7, 2;
    pub reserved1_0, set_reserved1_0: 1, 0;
}

bitfield! {
    #[derive(Clone, Copy)]
    pub struct AnalogSpare(u8);
    pub reserved7_0, set_reserved7_0: 7, 0;
}

bitfield! {
    #[derive(Clone, Copy)]
    pub struct PaCfg3(u8);
    not_used, _: 7, 3;
    pub reserved2_0, set_reserved2_0: 2, 0;
}

bitfield! {
    #[derive(Clone, Copy)]
    pub struct WorTime1(u8);
    pub wor_status_15_8, set_wor_status_15_8: 7, 0;
}

bitfield! {
    #[derive(Clone, Copy)]
    pub struct WorTime0(u8);
    pub wor_status_7_0, set_wor_status_7_0: 7, 0;
}

bitfield! {
    #[derive(Clone, Copy)]
    pub struct WorCapture1(u8);
    pub wor_capture_15_8, set_wor_capture_15_8: 7, 0;
}

bitfield! {
    #[derive(Clone, Copy)]
    pub struct WorCapture0(u8);
    pub wor_capture_7_0, set_wor_capture_7_0: 7, 0;
}

bitfield! {
    #[derive(Clone, Copy)]
    pub struct Bist(u8);
    not_used, _: 7, 4;
    pub reserved3_0, set_reserved3_0: 3, 0;
}

bitfield! {
    #[derive(Clone, Copy)]
    pub struct DcfiltoffsetI1(u8);
    pub dcfilt_offset_i_15_8, set_dcfilt_offset_i_15_8: 7, 0;
}

bitfield! {
    #[derive(Clone, Copy)]
    pub struct DcfiltoffsetI0(u8);
    pub dcfilt_offset_i_7_0, set_dcfilt_offset_i_7_0: 7, 0;
}

bitfield! {
    #[derive(Clone, Copy)]
    pub struct DcfiltoffsetQ1(u8);
    pub dcfilt_offset_q_15_8, set_dcfilt_offset_q_15_8: 7, 0;
}

bitfield! {
    #[derive(Clone, Copy)]
    pub struct DcfiltoffsetQ0(u8);
    pub dcfilt_offset_q_7_0, set_dcfilt_offset_q_7_0: 7, 0;
}

bitfield! {
    #[derive(Clone, Copy)]
    pub struct IqieI1(u8);
    pub iqie_i_15_8, set_iqie_i_15_8: 7, 0;
}

bitfield! {
    #[derive(Clone, Copy)]
    pub struct IqieI0(u8);
    pub iqie_i_7_0, set_iqie_i_7_0: 7, 0;
}

bitfield! {
    #[derive(Clone, Copy)]
    pub struct IqieQ1(u8);
    pub iqie_q_15_8, set_iqie_q_15_8: 7, 0;
}

bitfield! {
    #[derive(Clone, Copy)]
    pub struct IqieQ0(u8);
    pub iqie_q_7_0, set_iqie_q_7_0: 7, 0;
}

bitfield! {
    #[derive(Clone, Copy)]
    pub struct Rssi1(u8);
    pub rssi_11_4, _: 7, 0;
}

bitfield! {
    #[derive(Clone, Copy)]
    pub struct Rssi0(u8);
    not_used, _: 7;
    pub rssi_3_0, _: 6, 3;
    pub carrier_sense, _: 2;
    pub carrier_sense_valid, _: 1;
    pub rssi_valid, _: 0;
}

bitfield! {
    #[derive(Clone, Copy)]
    pub struct Marcstate(u8);
    not_used, _: 7;
    pub marc_2pin_state, _: 6, 5;
    pub marc_state, _: 4, 0;
}

bitfield! {
    #[derive(Clone, Copy)]
    pub struct LqiVal(u8);
    pub pkt_crc_ok, _: 7;
    pub lqi, _: 6, 0;
}

bitfield! {
    #[derive(Clone, Copy)]
    pub struct PqtSyncErr(u8);
    pub pqt_error, _: 7, 4;
    pub sync_error, _: 3, 0;
}

bitfield! {
    #[derive(Clone, Copy)]
    pub struct DemStatus(u8);
    pub rssi_step_found, _: 7;
    pub collision_found, _: 6;
    pub sync_low0_high1, _: 5;
    pub sro_indicator, _: 4, 1;
    pub image_found, _: 0;
}

bitfield! {
    #[derive(Clone, Copy)]
    pub struct FreqoffEst1(u8);
    pub freqoff_est_15_8, _: 7, 0;
}

bitfield! {
    #[derive(Clone, Copy)]
    pub struct FreqoffEst0(u8);
    pub freqoff_est_7_0, _: 7, 0;
}

bitfield! {
    #[derive(Clone, Copy)]
    pub struct AgcGain3(u8);
    not_used, _: 7;
    pub agc_front_end_gain, _: 6, 0;
}

bitfield! {
    #[derive(Clone, Copy)]
    pub struct AgcGain2(u8);
    pub agc_drives_fe_gain, set_agc_drives_fe_gain: 7;
    pub reserved6_0, set_reserved6_0: 6, 0;
}

bitfield! {
    #[derive(Clone, Copy)]
    pub struct AgcGain1(u8);
    not_used, _: 7, 5;
    pub reserved4_0, set_reserved4_0: 4, 0;
}

bitfield! {
    #[derive(Clone, Copy)]
    pub struct AgcGain0(u8);
    not_used, _: 7;
    pub reserved6_0, set_reserved6_0: 6, 0;
}

bitfield! {
    #[derive(Clone, Copy)]
    pub struct CfmRxDataOut(u8);
    pub cfm_rx_data, _: 7, 0;
}

bitfield! {
    #[derive(Clone, Copy)]
    pub struct CfmTxDataIn(u8);
    pub cfm_tx_data, set_cfm_tx_data: 7, 0;
}

bitfield! {
    #[derive(Clone, Copy)]
    pub struct AskSoftRxData(u8);
    not_used, _: 7, 6;
    pub ask_soft, _: 5, 0;
}

bitfield! {
    #[derive(Clone, Copy)]
    pub struct Rndgen(u8);
    pub rndgen_en, set_rndgen_en: 7;
    pub rndgen_value, _: 6, 0;
}

bitfield! {
    #[derive(Clone, Copy)]
    pub struct Magn2(u8);
    not_used, _: 7, 1;
    pub magn_16, _: 0;
}

bitfield! {
    #[derive(Clone, Copy)]
    pub struct Magn1(u8);
    pub magn_15_8, _: 7, 0;
}

bitfield! {
    #[derive(Clone, Copy)]
    pub struct Magn0(u8);
    pub magn_7_0, _: 7, 0;
}

bitfield! {
    #[derive(Clone, Copy)]
    pub struct Ang1(u8);
    not_used, _: 7, 2;
    pub angular_9_8, _: 1, 0;
}

bitfield! {
    #[derive(Clone, Copy)]
    pub struct Ang0(u8);
    pub angular_7_0, _: 7, 0;
}

bitfield! {
    #[derive(Clone, Copy)]
    pub struct ChfiltI2(u8);
    not_used, _: 7, 2;
    pub chfilt_startup_valid, _: 1;
    pub chfilt_i_16, _: 0;
}

bitfield! {
    #[derive(Clone, Copy)]
    pub struct ChfiltI1(u8);
    pub chfilt_i_15_8, _: 7, 0;
}

bitfield! {
    #[derive(Clone, Copy)]
    pub struct ChfiltI0(u8);
    pub chfilt_i_7_0, _: 7, 0;
}

bitfield! {
    #[derive(Clone, Copy)]
    pub struct ChfiltQ2(u8);
    not_used, _: 7, 1;
    pub chfilt_q_16, _: 0;
}

bitfield! {
    #[derive(Clone, Copy)]
    pub struct ChfiltQ1(u8);
    pub chfilt_q_15_8, _: 7, 0;
}

bitfield! {
    #[derive(Clone, Copy)]
    pub struct ChfiltQ0(u8);
    pub chfilt_q_7_0, _: 7, 0;
}

bitfield! {
    #[derive(Clone, Copy)]
    pub struct GpioStatus(u8);
    pub marc_gdo_state, _: 7, 4;
    pub gpio_state, _: 3, 0;
}

bitfield! {
    #[derive(Clone, Copy)]
    pub struct FscalCtrl(u8);
    not_used, _: 7;
    pub reserved6_1, set_reserved6_1: 6, 1;
    pub lock, _: 0;
}

bitfield! {
    #[derive(Clone, Copy)]
    pub struct PhaseAdjust(u8);
    pub reserved7_0, _: 7, 0;
}

bitfield! {
    #[derive(Clone, Copy)]
    pub struct Partnumber(u8);
    pub partnum, _: 7, 0;
}

bitfield! {
    #[derive(Clone, Copy)]
    pub struct Partversion(u8);
    pub partver, _: 7, 0;
}

bitfield! {
    #[derive(Clone, Copy)]
    pub struct SerialStatus(u8);
    not_used, _: 7, 6;
    pub spi_direct_access_cfg, set_spi_direct_access_cfg: 5;
    pub clk40k, _: 4;
    pub ioc_sync_pins_en, set_ioc_sync_pins_en: 3;
    pub cfm_tx_data_clk, _: 2;
    pub serial_rx, _: 1;
    pub serial_rx_clk, _: 0;
}

bitfield! {
    #[derive(Clone, Copy)]
    pub struct ModemStatus1(u8);
    pub sync_found, _: 7;
    pub rxfifo_full, _: 6;
    pub rxfifo_thr, _: 5;
    pub rxfifo_empty, _: 4;
    pub rxfifo_overflow, _: 3;
    pub rxfifo_underflow, _: 2;
    pub pqt_reached, _: 1;
    pub pqt_valid, _: 0;
}

bitfield! {
    #[derive(Clone, Copy)]
    pub struct ModemStatus0(u8);
    not_used, _: 7;
    pub fec_rx_overflow, _: 6;
    pub reserved5, _: 5;
    pub sync_sent, _: 4;
    pub txfifo_full, _: 3;
    pub txfifo_thr, _: 2;
    pub txfifo_overflow, _: 1;
    pub txfifo_underflow, _: 0;
}

bitfield! {
    #[derive(Clone, Copy)]
    pub struct MarcStatus1(u8);
    pub marc_status_out, _: 7, 0;
}

bitfield! {
    #[derive(Clone, Copy)]
    pub struct MarcStatus0(u8);
    not_used, _: 7, 4;
    pub reserved3, _: 3;
    pub txoncca_failed, _: 2;
    pub reserved1, _: 1;
    pub rcc_cal_valid, _: 0;
}
