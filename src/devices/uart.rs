// The code is from https://github.com/d0iasm/rvemu.

//! The uart module contains the implementation of a universal asynchronous receiver-transmitter
//! (UART) for the CLI tool. The device is 16550A UART, which is used in the QEMU virt machine.
//! See more information in http://byterunner.com/16550.html.

use std::io;
use std::io::prelude::*;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Condvar, Mutex,
};
use std::thread;

use crate::bus::{UART_SIZE};
use crate::cpu::Size;
use crate::exception::Exception;

use super::device::Device;

/// The interrupt request of UART.
pub const UART_IRQ: u32 = 10;

/// Receive holding register (for input bytes).
const UART_RHR: u32 = 0;
/// Transmit holding register (for output bytes).
const UART_THR: u32 = 0;
/// Interrupt enable register.
const _UART_IER: u32 = 1;
/// FIFO control register.
const _UART_FCR: u32 = 2;
/// Interrupt status register.
/// ISR BIT-0:
///     0 = an interrupt is pending and the ISR contents may be used as a pointer to the appropriate
/// interrupt service routine.
///     1 = no interrupt is pending.
const _UART_ISR: u32 = 2;
/// Line control register.
const _UART_LCR: u32 = 3;
/// Line status register.
/// LSR BIT 0:
///     0 = no data in receive holding register or FIFO.
///     1 = data has been receive and saved in the receive holding register or FIFO.
/// LSR BIT 5:
///     0 = transmit holding register is full. 16550 will not accept any data for transmission.
///     1 = transmitter hold register (or FIFO) is empty. CPU can load the next character.
const UART_LSR: u32 = 5;

/// The receiver (RX).
const UART_LSR_RX: u8 = 1;
/// The transmitter (TX).
const UART_LSR_TX: u8 = 1 << 5;

/// The UART, the size of which is 0x100 (2**8).
pub struct Uart {
    uart: Arc<(Mutex<[u8; UART_SIZE as usize]>, Condvar)>,
    interrupting: Arc<AtomicBool>,
}

impl Uart {
    /// Create a new UART object.
    pub fn new() -> Self {
        let uart = Arc::new((Mutex::new([0; UART_SIZE as usize]), Condvar::new()));
        let interrupting = Arc::new(AtomicBool::new(false));
        {
            let (uart, _cvar) = &*uart;
            let mut uart = uart.lock().expect("failed to get an UART object");
            // Transmitter hold register is empty. It allows input anytime.
            uart[UART_LSR as usize] |= UART_LSR_TX;
        }

        // Create a new thread for waiting for input.
        let mut byte = [0; 1];
        let cloned_uart = uart.clone();
        let cloned_interrupting = interrupting.clone();
        let _uart_thread_for_read = thread::spawn(move || loop {
            match io::stdin().read(&mut byte) {
                Ok(_) => {
                    let (uart, cvar) = &*cloned_uart;
                    let mut uart = uart.lock().expect("failed to get an UART object");
                    // Wait for the thread to start up.
                    while (uart[UART_LSR as usize] & UART_LSR_RX) == 1 {
                        uart = cvar.wait(uart).expect("the mutex is poisoned");
                    }
                    uart[0] = byte[0];
                    cloned_interrupting.store(true, Ordering::Release);
                    // Data has been receive.
                    uart[UART_LSR as usize] |= UART_LSR_RX;
                }
                Err(e) => {
                    println!("input via UART is error: {}", e);
                }
            }
        });

        Self { uart, interrupting }
    }

    /// Return true if an interrupt is pending. Clear the interrupting flag by swapping a value.
    pub fn is_interrupting(&self) -> bool {
        self.interrupting.swap(false, Ordering::Acquire)
    }

}


impl Device for Uart {
    /// Read a byte from the receive holding register.
    fn read(&mut self, index: u32, size: Size) -> Result<u32, Exception> {
        if size != Size::Byte {
            return Err(Exception::LoadIllegalAddress);
        }

        let (uart, cvar) = &*self.uart;
        let mut uart = uart.lock().expect("failed to get an UART object");
        match index {
            UART_RHR => {
                cvar.notify_one();
                uart[UART_LSR as usize] &= !UART_LSR_RX;
                Ok(uart[UART_RHR as usize] as u32)
            }
            _ => Ok(uart[index as usize] as u32),
        }
    }

    /// Write a byte to the transmit holding register.
    fn write(&mut self, index: u32, value: u32, size: Size) -> Result<(), Exception> {
        if size != Size::Byte {
            return Err(Exception::StoreIllegalAddress);
        }

        // An OS allows to write a byte to a UART when UART_LSR_TX is 1.
        // e.g. (xv6):
        //   // wait for Transmit Holding Empty to be set in LSR.
        //   while((ReadReg(LSR) & (1 << 5)) == 0)
        //   ;
        //   WriteReg(THR, c);
        //
        // e.g. (riscv-pk):
        //   while ((uart16550[UART_REG_LSR << uart16550_reg_shift] & UART_REG_STATUS_TX) == 0);
        //   uart16550[UART_REG_QUEUE << uart16550_reg_shift] = ch;
        let (uart, _cvar) = &*self.uart;
        let mut uart = uart.lock().expect("failed to get an UART object");
        match index {
            UART_THR => {
                print!("{}", value as u8 as char);
                io::stdout().flush().expect("failed to flush stdout");
            }
            _ => {
                uart[index as usize] = value as u8;
            }
        }
        Ok(())
    }
}