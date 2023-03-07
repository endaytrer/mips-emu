#[derive(Debug)]
pub enum Exception {
    Interrupt = 0,
    PageFault = 1,
    LoadIllegalAddress = 4,
    StoreIllegalAddress = 5,
    InstructionBusError = 6,
    DataBusError = 7,
    Syscall = 8,
    Break = 9,
    Reserved = 10,
    Overflow = 12
}