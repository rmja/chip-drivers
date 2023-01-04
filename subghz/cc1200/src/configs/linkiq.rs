use crate::{
    regs::{pri::Iocfg3, Register},
    ConfigPatch,
};

/// This CC1200 configuration is obtained using TI Smart RF Studio 7 v. 2.15.0
///
/// The following sequence of adjustmensts are performed after the CC1200 chip is selected from the main screen:
/// 1. Select "Expert Mode".
/// 2. Select "Packet RX".
/// 3. In the Device Control Panel, under Typical Settings, select "Symbol rate: 38.4kbps, 2-GFSK, RX BW 100kHz ETSI Standard (868MHz)".
/// 4. Under RF parameters, set:
///     Carrier Frequency: 868.450012MHz (868.450MHz)
///     Symbol Rate: 10 ksps
///     RX Filter BW: 20.325203kHz (20kHz)
///     Modulation Format: 2-GFSK
///     Deviation: 2.498627kHz (2.5kHz)
/// 5. In the Register View, make the following final adjustments:
///     IOCFG3         0x33  HW0
///     IOCFG2         0x33  HW0
///     IOCFG1         0x30
///     IOCFG0         0x33  HW0
///     SYNC3          0xD1
///     SYNC2          0xE7
///     SYNC1          0xE5
///     SYNC0          0x06
///     SYNC_CFG1      0xAB -> 0xAF  32 bit syncword and max threshold
///     PREAMBLE_CFG1  0x14 -> 0x19  4 byte 55 style preamble
///     FIFO_CFG       0x00 -> 0x0F  16 bytes FIFO threshold
///     SETTLING_CFG   0x0B  Enable auto-calibration when going from IDLE to RX/TX
///     PKT_CFG2       0x00  Use FIFO packet mode
///     RFEND_CFG1     0x0F -> 0x3F  Re-enter RX when RX ends
/// 6. Export registers using "RF settings" template - select all registers
pub const LINKIQ_CH0: ConfigPatch<'static> = ConfigPatch {
    first_address: Iocfg3::ADDRESS,
    values: &[
        0x06, // IOCFG3                GPIO3 IO Pin Configuration
        0x06, // IOCFG2                GPIO2 IO Pin Configuration
        0x30, // IOCFG1                GPIO1 IO Pin Configuration
        0x3C, // IOCFG0                GPIO0 IO Pin Configuration
        0x93, // SYNC3                 Sync Word Configuration [31:24]
        0x0B, // SYNC2                 Sync Word Configuration [23:16]
        0x51, // SYNC1                 Sync Word Configuration [15:8]
        0xDE, // SYNC0                 Sync Word Configuration [7:0]
        0xAF, // SYNC_CFG1             Sync Word Detection Configuration Reg. 1
        0x03, // SYNC_CFG0             Sync Word Detection Configuration Reg. 0
        0x83, // DEVIATION_M           Frequency Deviation Configuration
        0x08, // MODCFG_DEV_E          Modulation Format and Frequency Deviation Configur..
        0x4C, // DCFILT_CFG            Digital DC Removal Configuration
        0x14, // PREAMBLE_CFG1         Preamble Length Configuration Reg. 1
        0x8A, // PREAMBLE_CFG0         Preamble Detection Configuration Reg. 0
        0xC8, // IQIC                  Digital Image Channel Compensation Configuration
        0x69, // CHAN_BW               Channel Filter Configuration
        0x42, // MDMCFG1               General Modem Parameter Configuration Reg. 1
        0x05, // MDMCFG0               General Modem Parameter Configuration Reg. 0
        0x70, // SYMBOL_RATE2          Symbol Rate Configuration Exponent and Mantissa [1..
        0x62, // SYMBOL_RATE1          Symbol Rate Configuration Mantissa [15:8]
        0x4E, // SYMBOL_RATE0          Symbol Rate Configuration Mantissa [7:0]
        0x20, // AGC_REF               AGC Reference Level Configuration
        0xEE, // AGC_CS_THR            Carrier Sense Threshold Configuration
        0x00, // AGC_GAIN_ADJUST       RSSI Offset Configuration
        0xB1, // AGC_CFG3              Automatic Gain Control Configuration Reg. 3
        0x20, // AGC_CFG2              Automatic Gain Control Configuration Reg. 2
        0x11, // AGC_CFG1              Automatic Gain Control Configuration Reg. 1
        0x94, // AGC_CFG0              Automatic Gain Control Configuration Reg. 0
        0x00, // FIFO_CFG              FIFO Configuration
        0x00, // DEV_ADDR              Device Address Configuration
        0x0B, // SETTLING_CFG          Frequency Synthesizer Calibration and Settling Con..
        0x12, // FS_CFG                Frequency Synthesizer Configuration
        0x08, // WOR_CFG1              eWOR Configuration Reg. 1
        0x21, // WOR_CFG0              eWOR Configuration Reg. 0
        0x00, // WOR_EVENT0_MSB        Event 0 Configuration MSB
        0x00, // WOR_EVENT0_LSB        Event 0 Configuration LSB
        0x00, // RXDCM_TIME            RX Duty Cycle Mode Configuration
        0x00, // PKT_CFG2              Packet Configuration Reg. 2
        0x03, // PKT_CFG1              Packet Configuration Reg. 1
        0x20, // PKT_CFG0              Packet Configuration Reg. 0
        0x0F, // RFEND_CFG1            RFEND Configuration Reg. 1
        0x00, // RFEND_CFG0            RFEND Configuration Reg. 0
        0x7F, // PA_CFG1               Power Amplifier Configuration Reg. 1
        0x55, // PA_CFG0               Power Amplifier Configuration Reg. 0
        0x0F, // ASK_CFG               ASK Configuration
        0xFF, // PKT_LEN               Packet Length Configuration
        0x1C, // IF_MIX_CFG            IF Mix Configuration
        0x20, // FREQOFF_CFG           Frequency Offset Correction Configuration
        0x03, // TOC_CFG               Timing Offset Correction Configuration
        0x00, // MARC_SPARE            MARC Spare
        0x00, // ECG_CFG               External Clock Frequency Configuration
        0x02, // MDMCFG2               General Modem Parameter Configuration Reg. 2
        0x01, // EXT_CTRL              External Control Configuration
        0x00, // RCCAL_FINE            RC Oscillator Calibration Fine
        0x00, // RCCAL_COARSE          RC Oscillator Calibration Coarse
        0x00, // RCCAL_OFFSET          RC Oscillator Calibration Clock Offset
        0x00, // FREQOFF1              Frequency Offset MSB
        0x00, // FREQOFF0              Frequency Offset LSB
        0x56, // FREQ2                 Frequency Configuration [23:16]
        0xD8, // FREQ1                 Frequency Configuration [15:8]
        0x52, // FREQ0                 Frequency Configuration [7:0]
        0x02, // IF_ADC2               Analog to Digital Converter Configuration Reg. 2
        0xEE, // IF_ADC1               Analog to Digital Converter Configuration Reg. 1
        0x10, // IF_ADC0               Analog to Digital Converter Configuration Reg. 0
        0x07, // FS_DIG1               Frequency Synthesizer Digital Reg. 1
        0xAF, // FS_DIG0               Frequency Synthesizer Digital Reg. 0
        0x00, // FS_CAL3               Frequency Synthesizer Calibration Reg. 3
        0x20, // FS_CAL2               Frequency Synthesizer Calibration Reg. 2
        0x40, // FS_CAL1               Frequency Synthesizer Calibration Reg. 1
        0x0E, // FS_CAL0               Frequency Synthesizer Calibration Reg. 0
        0x28, // FS_CHP                Frequency Synthesizer Charge Pump Configuration
        0x03, // FS_DIVTWO             Frequency Synthesizer Divide by 2
        0x00, // FS_DSM1               FS Digital Synthesizer Module Configuration Reg. 1
        0x33, // FS_DSM0               FS Digital Synthesizer Module Configuration Reg. 0
        0xFF, // FS_DVC1               Frequency Synthesizer Divider Chain Configuration ..
        0x17, // FS_DVC0               Frequency Synthesizer Divider Chain Configuration ..
        0x00, // FS_LBI                Frequency Synthesizer Local Bias Configuration
        0x00, // FS_PFD                Frequency Synthesizer Phase Frequency Detector Con..
        0x6E, // FS_PRE                Frequency Synthesizer Prescaler Configuration
        0x1C, // FS_REG_DIV_CML        Frequency Synthesizer Divider Regulator Configurat..
        0xAC, // FS_SPARE              Frequency Synthesizer Spare
        0x14, // FS_VCO4               FS Voltage Controlled Oscillator Configuration Reg..
        0x00, // FS_VCO3               FS Voltage Controlled Oscillator Configuration Reg..
        0x00, // FS_VCO2               FS Voltage Controlled Oscillator Configuration Reg..
        0x00, // FS_VCO1               FS Voltage Controlled Oscillator Configuration Reg..
        0xB5, // FS_VCO0               FS Voltage Controlled Oscillator Configuration Reg..
        0x00, // GBIAS6                Global Bias Configuration Reg. 6
        0x02, // GBIAS5                Global Bias Configuration Reg. 5
        0x00, // GBIAS4                Global Bias Configuration Reg. 4
        0x00, // GBIAS3                Global Bias Configuration Reg. 3
        0x10, // GBIAS2                Global Bias Configuration Reg. 2
        0x00, // GBIAS1                Global Bias Configuration Reg. 1
        0x00, // GBIAS0                Global Bias Configuration Reg. 0
        0x09, // IFAMP                 Intermediate Frequency Amplifier Configuration
        0x01, // LNA                   Low Noise Amplifier Configuration
        0x01, // RXMIX                 RX Mixer Configuration
        0x0E, // XOSC5                 Crystal Oscillator Configuration Reg. 5
        0xA0, // XOSC4                 Crystal Oscillator Configuration Reg. 4
        0x03, // XOSC3                 Crystal Oscillator Configuration Reg. 3
        0x04, // XOSC2                 Crystal Oscillator Configuration Reg. 2
        0x03, // XOSC1                 Crystal Oscillator Configuration Reg. 1
        0x00, // XOSC0                 Crystal Oscillator Configuration Reg. 0
        0x00, // ANALOG_SPARE          Analog Spare
        0x00, // PA_CFG3               Power Amplifier Configuration Reg. 3
    ],
};
