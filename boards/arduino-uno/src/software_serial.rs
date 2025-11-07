//! Software Serial (UART emulation on any pins)
//!
//! This module provides software-emulated UART communication on any digital pins.
//! It uses pin change interrupts for reception and precise timing for both TX and RX.
//!
//! Note: Only one SoftwareSerial instance can actively receive at a time.

use core::ptr::{read_volatile, write_volatile};
use core::cell::Cell;
use critical_section::Mutex;
use crate::ports::{digital_pin_to_port, digital_pin_to_bit_mask, port_input_register, port_output_register, port_mode_register};

// Receive buffer size
const RX_BUFFER_SIZE: usize = 64;

// Pin Change Interrupt registers
const PCICR: *mut u8 = 0x68 as *mut u8;
const PCMSK0: *mut u8 = 0x6B as *mut u8;
const PCMSK1: *mut u8 = 0x6C as *mut u8;
const PCMSK2: *mut u8 = 0x6D as *mut u8;

/// Software Serial state
struct SoftwareSerialState {
    rx_buffer: [u8; RX_BUFFER_SIZE],
    rx_buffer_head: usize,
    rx_buffer_tail: usize,
    buffer_overflow: bool,

    // Timing delays (in 4-cycle units for tunedDelay)
    rx_delay_centering: u16,
    rx_delay_intrabit: u16,
    rx_delay_stopbit: u16,
    tx_delay: u16,

    // Pin configuration
    rx_pin: u8,
    tx_pin: u8,
    rx_bit_mask: u8,
    tx_bit_mask: u8,
    rx_port: *const u8,
    tx_port: *mut u8,

    inverse_logic: bool,
    is_listening: bool,
}

static ACTIVE_INSTANCE: Mutex<Cell<Option<usize>>> = Mutex::new(Cell::new(None));
static mut INSTANCES: [Option<SoftwareSerialState>; 4] = [None, None, None, None];
static mut INSTANCE_COUNT: usize = 0;

/// Software Serial instance
pub struct SoftwareSerial {
    instance_id: usize,
}

impl SoftwareSerial {
    /// Create a new SoftwareSerial instance
    ///
    /// # Arguments
    /// * `rx_pin` - Pin number for receiving data
    /// * `tx_pin` - Pin number for transmitting data
    /// * `inverse_logic` - Use inverse signal levels (default false)
    ///
    /// # Example
    /// ```no_run
    /// use arduino_uno::SoftwareSerial;
    ///
    /// let mut sw_serial = SoftwareSerial::new(2, 3, false);
    /// sw_serial.begin(9600);
    /// ```
    pub fn new(rx_pin: u8, tx_pin: u8, inverse_logic: bool) -> Self {
        let instance_id = unsafe {
            let id = INSTANCE_COUNT;
            INSTANCE_COUNT += 1;

            let rx_port = digital_pin_to_port(rx_pin);
            let tx_port = digital_pin_to_port(tx_pin);
            let rx_bit_mask = digital_pin_to_bit_mask(rx_pin);
            let tx_bit_mask = digital_pin_to_bit_mask(tx_pin);

            INSTANCES[id] = Some(SoftwareSerialState {
                rx_buffer: [0; RX_BUFFER_SIZE],
                rx_buffer_head: 0,
                rx_buffer_tail: 0,
                buffer_overflow: false,
                rx_delay_centering: 0,
                rx_delay_intrabit: 0,
                rx_delay_stopbit: 0,
                tx_delay: 0,
                rx_pin,
                tx_pin,
                rx_bit_mask,
                tx_bit_mask,
                rx_port: port_input_register(rx_port),
                tx_port: port_output_register(tx_port),
                inverse_logic,
                is_listening: false,
            });

            id
        };

        Self { instance_id }
    }

    /// Initialize the software serial port at the specified baud rate
    ///
    /// # Arguments
    /// * `baud` - Baud rate (e.g., 9600, 19200, 38400)
    pub fn begin(&mut self, baud: u32) {
        unsafe {
            if let Some(state) = &mut INSTANCES[self.instance_id] {
                // Calculate timing delays based on CPU frequency (16MHz) and baud rate
                // Each cycle is 62.5ns at 16MHz
                let bit_delay = (16_000_000 / baud) as u16;

                // Convert to 4-cycle delays for tunedDelay
                state.tx_delay = (bit_delay / 4).saturating_sub(15);
                state.rx_delay_centering = (bit_delay / 2 / 4).saturating_sub(5);
                state.rx_delay_intrabit = (bit_delay / 4).saturating_sub(15);
                state.rx_delay_stopbit = (bit_delay / 4).saturating_sub(15);

                // Set TX pin as output, idle high (or low if inverse)
                let tx_port = digital_pin_to_port(state.tx_pin);
                let tx_ddr = port_mode_register(tx_port);
                let tx_port_reg = state.tx_port;

                // Set pin mode to output
                let ddr_val = read_volatile(tx_ddr);
                write_volatile(tx_ddr, ddr_val | state.tx_bit_mask);

                // Set initial state (idle)
                let port_val = read_volatile(tx_port_reg);
                if state.inverse_logic {
                    write_volatile(tx_port_reg, port_val & !state.tx_bit_mask);
                } else {
                    write_volatile(tx_port_reg, port_val | state.tx_bit_mask);
                }

                // Set RX pin as input
                let rx_port = digital_pin_to_port(state.rx_pin);
                let rx_ddr = port_mode_register(rx_port);
                let ddr_val = read_volatile(rx_ddr);
                write_volatile(rx_ddr, ddr_val & !state.rx_bit_mask);
            }
        }

        self.listen();
    }

    /// Enable this instance to receive data
    pub fn listen(&mut self) {
        critical_section::with(|cs| {
            ACTIVE_INSTANCE.borrow(cs).set(Some(self.instance_id));

            unsafe {
                if let Some(state) = &mut INSTANCES[self.instance_id] {
                    state.is_listening = true;
                    state.buffer_overflow = false;
                    state.rx_buffer_head = 0;
                    state.rx_buffer_tail = 0;

                    // Enable PCINT for RX pin
                    self.enable_pcint(state.rx_pin);
                }
            }
        });
    }

    /// Stop listening for data
    pub fn end(&mut self) {
        unsafe {
            if let Some(state) = &mut INSTANCES[self.instance_id] {
                state.is_listening = false;
                self.disable_pcint(state.rx_pin);
            }
        }
    }

    /// Check if this instance is currently listening
    pub fn is_listening(&self) -> bool {
        unsafe {
            INSTANCES[self.instance_id]
                .as_ref()
                .map(|s| s.is_listening)
                .unwrap_or(false)
        }
    }

    /// Write a byte
    pub fn write_byte(&mut self, byte: u8) {
        unsafe {
            if let Some(state) = &INSTANCES[self.instance_id] {
                // Disable interrupts for precise timing
                let sreg = read_volatile(0x5F as *const u8);
                core::arch::asm!("cli", options(nomem, nostack));

                let tx_port = state.tx_port;
                let bit_mask = state.tx_bit_mask;
                let inverse = state.inverse_logic;

                // Send start bit
                let port_val = read_volatile(tx_port);
                if inverse {
                    write_volatile(tx_port, port_val | bit_mask);
                } else {
                    write_volatile(tx_port, port_val & !bit_mask);
                }
                tuned_delay(state.tx_delay);

                // Send 8 data bits
                for i in 0..8 {
                    let bit_val = (byte >> i) & 0x01;
                    let port_val = read_volatile(tx_port);

                    if inverse {
                        if bit_val == 1 {
                            write_volatile(tx_port, port_val & !bit_mask);
                        } else {
                            write_volatile(tx_port, port_val | bit_mask);
                        }
                    } else {
                        if bit_val == 1 {
                            write_volatile(tx_port, port_val | bit_mask);
                        } else {
                            write_volatile(tx_port, port_val & !bit_mask);
                        }
                    }

                    tuned_delay(state.tx_delay);
                }

                // Send stop bit (restore idle state)
                let port_val = read_volatile(tx_port);
                if inverse {
                    write_volatile(tx_port, port_val & !bit_mask);
                } else {
                    write_volatile(tx_port, port_val | bit_mask);
                }
                tuned_delay(state.tx_delay);

                // Restore interrupts
                write_volatile(0x5F as *mut u8, sreg);
            }
        }
    }

    /// Write a string
    pub fn write_str(&mut self, s: &str) {
        for byte in s.bytes() {
            self.write_byte(byte);
        }
    }

    /// Read a byte from the receive buffer
    ///
    /// Returns -1 if no data available
    pub fn read(&mut self) -> i16 {
        unsafe {
            if let Some(state) = &mut INSTANCES[self.instance_id] {
                if state.rx_buffer_head == state.rx_buffer_tail {
                    -1
                } else {
                    let byte = state.rx_buffer[state.rx_buffer_tail];
                    state.rx_buffer_tail = (state.rx_buffer_tail + 1) % RX_BUFFER_SIZE;
                    byte as i16
                }
            } else {
                -1
            }
        }
    }

    /// Get number of bytes available in receive buffer
    pub fn available(&self) -> usize {
        unsafe {
            if let Some(state) = &INSTANCES[self.instance_id] {
                (RX_BUFFER_SIZE + state.rx_buffer_head - state.rx_buffer_tail) % RX_BUFFER_SIZE
            } else {
                0
            }
        }
    }

    /// Peek at the next byte without removing it
    pub fn peek(&self) -> i16 {
        unsafe {
            if let Some(state) = &INSTANCES[self.instance_id] {
                if state.rx_buffer_head == state.rx_buffer_tail {
                    -1
                } else {
                    state.rx_buffer[state.rx_buffer_tail] as i16
                }
            } else {
                -1
            }
        }
    }

    /// Check if buffer overflow occurred
    pub fn overflow(&mut self) -> bool {
        unsafe {
            if let Some(state) = &mut INSTANCES[self.instance_id] {
                let overflow = state.buffer_overflow;
                state.buffer_overflow = false;
                overflow
            } else {
                false
            }
        }
    }

    // Helper to enable PCINT for a pin
    fn enable_pcint(&self, pin: u8) {
        unsafe {
            let (pcie_bit, pcmsk) = match pin {
                0..=7 => (2, PCMSK2),   // PORTD
                8..=13 => (0, PCMSK0),  // PORTB
                14..=19 => (1, PCMSK1), // PORTC
                _ => return,
            };

            let bit = digital_pin_to_bit_mask(pin);

            // Enable pin in mask
            let mask = read_volatile(pcmsk);
            write_volatile(pcmsk, mask | bit);

            // Enable PCIE
            let pcicr = read_volatile(PCICR);
            write_volatile(PCICR, pcicr | (1 << pcie_bit));
        }
    }

    // Helper to disable PCINT for a pin
    fn disable_pcint(&self, pin: u8) {
        unsafe {
            let pcmsk = match pin {
                0..=7 => PCMSK2,
                8..=13 => PCMSK0,
                14..=19 => PCMSK1,
                _ => return,
            };

            let bit = digital_pin_to_bit_mask(pin);
            let mask = read_volatile(pcmsk);
            write_volatile(pcmsk, mask & !bit);
        }
    }
}

/// Tuned delay function for precise timing
/// Delays for (4 * cycles) clock cycles
#[inline(never)]
fn tuned_delay(cycles: u16) {
    if cycles == 0 {
        return;
    }

    unsafe {
        // AVR assembly delay loop - each iteration is 4 cycles
        core::arch::asm!(
            "1:",
            "sbiw {0}, 1",
            "brne 1b",
            inout(reg_iw) cycles => _,
            options(nomem, nostack)
        );
    }
}

/// Internal receive handler called from PCINT ISR
pub fn handle_interrupt() {
    critical_section::with(|cs| {
        if let Some(instance_id) = ACTIVE_INSTANCE.borrow(cs).get() {
            unsafe {
                if let Some(state) = &mut INSTANCES[instance_id] {
                    recv_data(state);
                }
            }
        }
    });
}

/// Receive data (called from interrupt)
unsafe fn recv_data(state: &mut SoftwareSerialState) {
    // Check for start bit
    let rx_val = read_volatile(state.rx_port);
    let start_bit = (rx_val & state.rx_bit_mask) != 0;

    let expected_start = if state.inverse_logic { true } else { false };
    if start_bit != expected_start {
        return;  // Not a valid start bit
    }

    // Wait to center of first data bit
    tuned_delay(state.rx_delay_centering);

    let mut data: u8 = 0;

    // Read 8 data bits
    for i in 0..8 {
        tuned_delay(state.rx_delay_intrabit);

        let rx_val = read_volatile(state.rx_port);
        let bit = ((rx_val & state.rx_bit_mask) != 0) as u8;

        let bit_val = if state.inverse_logic { !bit & 0x01 } else { bit };
        data |= bit_val << i;
    }

    // Wait for stop bit
    tuned_delay(state.rx_delay_stopbit);

    // Store in buffer
    let next_head = (state.rx_buffer_head + 1) % RX_BUFFER_SIZE;
    if next_head != state.rx_buffer_tail {
        state.rx_buffer[state.rx_buffer_head] = data;
        state.rx_buffer_head = next_head;
    } else {
        state.buffer_overflow = true;
    }
}

// Export interrupt handler hooks for PCINT
#[no_mangle]
pub unsafe extern "C" fn software_serial_pcint_hook() {
    handle_interrupt();
}
