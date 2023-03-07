use std::{thread, sync::{Arc, Mutex}};
use crate::bus::{UART_BASE, UART_END, VIRTIO_BASE, VIRTIO_END};

use crate::cpu::{Cpu, Instruction};

mod cpu;
mod coprocessor;
mod bus;
mod dram;
mod rom;
mod devices;
mod exception;
mod utils;
mod memory;


#[cfg(test)]
mod test;

fn main() {
    let mut cpu = Cpu::new("asm/gauss");
    // let stop_signal = Arc::new(Mutex::new(false));
    // let ss_main = stop_signal.clone();
    // let cause = cpu.bus.load_raw_cause();
    // thread::spawn(move || {
    //     cpu.run(ss_main);
    // });
    cpu.debug(3087);
    cpu.print_status();
}
