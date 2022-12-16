use crate::opcode::PriReg;

#[derive(Copy, Clone)]
pub enum Gpio {
    Gpio0,
    Gpio1,
    Gpio2,
    Gpio3,
}

impl Gpio {
    pub fn iocfg_reg(self) -> PriReg {
        match self {
            Gpio::Gpio0 => PriReg::IOCFG0,
            Gpio::Gpio1 => PriReg::IOCFG1,
            Gpio::Gpio2 => PriReg::IOCFG2,
            Gpio::Gpio3 => PriReg::IOCFG3,
        }
    }
}

macro_rules! gpio_output {
    ($name:ident, $($variant:ident = $value:expr),*) => {
        #[allow(non_camel_case_types)]
        pub enum $name {
            /// Asserted when the RX FIFO is filled above FIFO_CFG.FIFO_THR. De-asserted
            /// when the RX FIFO is drained below (or is equal) to the same threshold.
            RXFIFO_THR = 0,
            /// Asserted when the RX FIFO is filled above FIFO_CFG.FIFO_THR or the end of
            /// packet is reached. De-asserted when the RX FIFO is empty.
            RXFIFO_THR_PKT = 1,
            /// Asserted when the TX FIFO is filled above (or is equal to)
            /// (127−FIFO_CFG.FIFO_THR). De-asserted when the TX FIFO is drained below the
            /// same threshold.
            TXFIFO_THR = 2,
            /// Asserted when the TX FIFO is full.
            /// De-asserted when the TX FIFO is drained below (127−FIFO_CFG.FIFO_THR).
            TXFIFO_THR_PKT = 3,
            RXFIFO_OVERFLOW = 4,
            TXFIFO_UNDERFLOW = 5,
            /// Asserted when sync word has been received and de-asserted at the end of the
            /// packet. Will de-assert when the optional address and/or length check fails
            /// or the RX FIFO overflows/underflows
            PKT_SYNC_RXTX = 6,
            CRC_OK = 7,
            SERIAL_CLK = 8,
            SERIAL_RX = 9,
            RESERVED_10,
            PQT_REACHED = 11,
            PQT_VALID = 12,
            /// RSSI calculation is valid
            RSSI_VALID = 13,
            // 14 depends on pin
            // 15 depends on pin
            CARRIER_SENSE_VALID = 16,
            CARRIER_SENSE = 17,
            // 18 depends on pin
            PKT_CRC_OK = 19,
            MCU_WAKEUP = 20,
            SYNC_LOW0_HIGH1 = 21,
            // 22 depends on pin
            LNA_PA_REG_PD = 23,
            LNA_PD = 24,
            PA_PD = 25,
            RX0TX1_CFG = 26,
            RESERVED_27 = 27,
            IMAGE_FOUND = 28,
            CLKEN_CFM = 29,
            CFM_TX_DATA_CLK = 30,
            RESERVED_31 = 31,
            RESERVED_32 = 32,
            RSSI_STEP_FOUND = 33,
            // 34 depends on pin
            // 35 depends on pin
            ANTENNA_SELECT = 36,
            MARC_2PIN_STATUS1 = 37,
            MARC_2PIN_STATUS0 = 38,
            // 39 depends on pin
            // 40 depends on pin
            // 41 depends on pin
            PA_RAMP_UP = 42,
            AGC_STABLE_GAIN = 44,
            AGC_UPDATE = 45,
            // 46 depends on pin
            RESERVED_47 = 47,
            HIGHZ = 48,
            EXT_CLOCK = 49,
            CHIP_RDYn = 50,
            HW0 = 51,
            RESERVED_52 = 52,
            RESERVED_53 = 53,
            CLOCK_40K = 54,
            WOR_EVENT0 = 55,
            WOR_EVENT1 = 56,
            WOR_EVENT2 = 57,
            RESERVED_58 = 58,
            XOSC_STABLE = 59,
            EXT_OSC_EN = 60,
            RESERVED_61 = 61,
            RESERVED_62 = 62,
            RESERVED_63 = 63,
            $($variant = $value),+
        }
    }
}

gpio_output!(
    Gpio0Output,
    AGC_UPDATE_14 = 14,
    TXONCCA_FAILED = 15,
    DSSS_DATA1 = 18,
    AES_COMMAND_ACTIVE = 22,
    RSSI_STEP_EVENT = 34,
    LOCK = 35,
    RXFIFO_UNDERFLOW = 39,
    CHFILT_STARTUP_VALID = 40,
    COLLISION_EVENT = 41,
    UART_FRAMING_ERROR = 43,
    ADC_I_DATA_SAMPLE = 46
);

gpio_output!(
    Gpio1Output,
    AGC_HOLD = 14,
    CCA_STATUS = 15,
    DSSS_CLK = 18,
    RESERVED_22 = 22,
    RSSI_STEP_EVENT = 34,
    LOCK = 35,
    RESERVED_39 = 39,
    RCC_CAL_VALID = 40,
    COLLISION_FOUND = 41,
    ADDR_FAILED = 43,
    ADC_CLOCK = 46
);

gpio_output!(
    Gpio2Output,
    RSSI_UPDATE = 14,
    TXONCCA_DONE = 15,
    DSSS_DATA0 = 18,
    RESERVED_22 = 22,
    AES_RUN = 34,
    RESERVED_35 = 35,
    TXFIFO_OVERFLOW = 39,
    CHFILT_VALID = 40,
    SYNC_EVENT = 41,
    LENGTH_FAILED = 43,
    ADC_Q_DATA_SAMPLE = 46
);

gpio_output!(
    Gpio3Output,
    RSSI_UPDATE = 14,
    CCA_STATUS = 15,
    DSSS_CLK = 18,
    RESERVED_22 = 22,
    AES_RUN = 34,
    RESERVED_35 = 35,
    RESERVED_39 = 39,
    MAGN_VALID = 40,
    COLLISION_FOUND = 41,
    CRC_FAILED = 43,
    ADC_CLOCK = 46
);
