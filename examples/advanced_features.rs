//! Advanced Features Demo
//!
//! This example demonstrates all the advanced features of the Ossidata SDK:
//! - SoftwareSerial (UART on any pins)
//! - ArduinoString (safe string operations)
//! - PROGMEM/Flash strings (F! macro)
//! - Pin Change Interrupts (PCINT)
//! - Timer configuration
//! - Memory inspection
//! - Low-level port access

#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]

use arduino_uno::*;
use panic_halt as _;

static mut BUTTON_PRESSED: bool = false;

fn button_handler() {
    unsafe {
        BUTTON_PRESSED = true;
    }
}

#[arduino_uno::entry]
fn main() -> ! {
    let mut peripherals = Peripherals::take().unwrap();
    let mut serial = Serial::new(9600);
    let mut delay = Delay::new();

    // Configure LED pin
    peripherals.pins.d13.into_output();

    serial.write_str("\r\n=== Ossidata Advanced Features Demo ===\r\n\r\n");

    // 1. PROGMEM / Flash Strings Demo
    serial.write_str("1. PROGMEM Flash Strings:\r\n");
    serial.write_flash_str(&F!("  This string is stored in flash memory!\r\n"));
    serial.writeln_flash_str(&F!("  Saves precious RAM on AVR"));
    serial.write_str("\r\n");

    // 2. String Class Demo
    serial.write_str("2. Arduino String Class:\r\n");
    let mut s = String::new();
    s.push_str("  Value: ");
    s.concat_int(42, DEC);
    s.push_str(" (decimal), ");
    s.concat_int(42, HEX);
    s.push_str(" (hex), ");
    s.concat_int(42, BIN);
    s.push_str(" (binary)\r\n");
    serial.write_str(s.as_str());

    let mut float_str = String::new();
    float_str.push_str("  Pi ≈ ");
    float_str.concat_float(3.14159, 4);
    float_str.push_str("\r\n");
    serial.write_str(float_str.as_str());
    serial.write_str("\r\n");

    // 3. Memory Inspection Demo
    serial.write_str("3. Memory Information:\r\n");
    let mem_info = memory_info();
    serial.write_str("  Total RAM: ");
    serial.print_uint(mem_info.total_ram as u32, DEC);
    serial.write_str(" bytes\r\n  Free RAM: ");
    serial.print_uint(mem_info.free_ram as u32, DEC);
    serial.write_str(" bytes\r\n  Data section: ");
    serial.print_uint(mem_info.data_section as u32, DEC);
    serial.write_str(" bytes\r\n  BSS section: ");
    serial.print_uint(mem_info.bss_section as u32, DEC);
    serial.write_str(" bytes\r\n\r\n");

    // 4. SoftwareSerial Demo
    serial.write_str("4. SoftwareSerial:\r\n");
    serial.write_str("  Creating software UART on pins 2 (RX), 3 (TX)\r\n");
    let mut sw_serial = SoftwareSerial::new(2, 3, false);
    sw_serial.begin(9600);
    sw_serial.write_str("Hello from SoftwareSerial!\r\n");
    serial.write_str("  Sent message on software UART\r\n\r\n");

    // 5. Pin Change Interrupt Demo
    serial.write_str("5. Pin Change Interrupts:\r\n");
    serial.write_str("  Attaching PCINT to pin 2\r\n");
    pin_mode(2, INPUT_PULLUP);
    pcint_attach(2, button_handler);
    serial.write_str("  Press button on pin 2 to test\r\n\r\n");

    // 6. Timer Functions Demo
    serial.write_str("6. Timer Configuration:\r\n");
    let timer_val = timer_read(Timer::Timer1);
    serial.write_str("  Timer1 value: ");
    serial.print_uint(timer_val as u32, DEC);
    serial.write_str("\r\n");
    serial.write_str("  Setting Timer1 compare value to 1000\r\n");
    timer_set_compare_a(Timer::Timer1, 1000);
    serial.write_str("\r\n");

    // 7. Low-Level Port Access Demo
    serial.write_str("7. Low-Level Port Access:\r\n");
    serial.write_str("  Reading Port B: 0x");
    serial.print_uint(port_read(Port::B) as u32, HEX);
    serial.write_str("\r\n  Pin 13 is on Port B, bit ");
    serial.print_uint(digital_pin_to_bit_mask(13) as u32, DEC);
    serial.write_str("\r\n\r\n");

    // 8. Print with Number Bases
    serial.write_str("8. Number Base Printing:\r\n");
    serial.write_str("  255 in different bases:\r\n");
    serial.write_str("    DEC: ");
    serial.print_uint(255, DEC);
    serial.write_str("\r\n    HEX: ");
    serial.print_uint(255, HEX);
    serial.write_str("\r\n    OCT: ");
    serial.print_uint(255, OCT);
    serial.write_str("\r\n    BIN: ");
    serial.print_uint(255, BIN);
    serial.write_str("\r\n\r\n");

    // 9. Float Precision Printing
    serial.write_str("9. Float Precision:\r\n");
    serial.write_str("  3.14159 with different precision:\r\n");
    serial.write_str("    2 digits: ");
    serial.print_float(3.14159, 2);
    serial.write_str("\r\n    4 digits: ");
    serial.print_float(3.14159, 4);
    serial.write_str("\r\n    6 digits: ");
    serial.print_float(3.14159, 6);
    serial.write_str("\r\n\r\n");

    serial.write_str("=== Demo Complete ===\r\n");
    serial.write_str("Entering main loop (LED blink + button check)...\r\n\r\n");

    let mut count = 0u32;

    loop {
        // Blink LED
        peripherals.pins.d13.toggle();

        // Check button press via PCINT
        unsafe {
            if BUTTON_PRESSED {
                BUTTON_PRESSED = false;
                serial.write_str("Button pressed via PCINT!\r\n");
            }
        }

        // Print loop counter every 10 iterations
        if count % 10 == 0 {
            let mut msg = String::new();
            msg.push_str("Loop ");
            msg.concat_uint(count, DEC);
            msg.push_str(", Free RAM: ");
            msg.concat_uint(free_memory() as u32, DEC);
            msg.push_str(" bytes\r\n");
            serial.write_str(msg.as_str());
        }

        count += 1;
        delay.delay_ms(500);
    }
}
