use bitflags::bitflags;

bitflags! {
    #[derive(Debug)]
    pub struct StatusRegister: u16 {
        const INTERRUPT_STATUS = 1 << 2;
        const CAPABILITIES_LIST = 1 << 3;
        const MHZ66_CAPABLE = 1 << 4;
        const FAST_BACK_TO_BACK_CAPABLE = 1 << 7;
        const MASTER_DATA_PARITY_ERROR = 1 << 8;
        const SIGNALED_TARGET_ABORT = 1 << 11;
        const RECEIVED_TARGET_ABORT = 1 << 12;
        const RECEIVED_MASTER_ABORT = 1 << 13;
        const SIGNALED_SYSTEM_ERROR = 1 << 14;
        const DETECTED_PARITY_ERROR = 1 << 15;
    }
}
