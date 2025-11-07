//! Sleep mode test - demonstrates power saving modes
//!
//! This example demonstrates different sleep modes:
//! 1. Idle mode - CPU stops, peripherals continue
//! 2. Power-down mode - Maximum power savings
//! 3. Wake-up from external interrupt (pin D2)
//!
//! The LED blinks to show the system is awake, then enters sleep mode.
//! Press a button on D2 to wake the system from sleep.
//!
//! Hardware:
//! - Arduino Uno with LED on D13
//! - Button connected to D2 (with pull-up resistor)
//!
//! Serial Monitor: 9600 baud

#![no_std]
#![no_main]
#![feature(asm_experimental_arch)]
#![feature(abi_avr_interrupt)]

use arduino_uno::{Peripherals, Serial, Sleep, SleepMode, millis, attach_interrupt, ExternalInterrupt, InterruptMode};
use panic_halt as _;
use core::sync::atomic::{AtomicBool, Ordering};

// Flag to track button press
static BUTTON_PRESSED: AtomicBool = AtomicBool::new(false);

#[no_mangle]
pub extern "C" fn main() -> ! {
    let peripherals = Peripherals::take().unwrap();

    // Configure D13 (LED) as output
    let mut led = peripherals.pins.d13.into_output();

    // D2 is already input by default (used for INT0 interrupt)

    // Initialize serial at 9600 baud
    let mut serial = Serial::new(9600);

    serial.println("");
    serial.println("==================");
    serial.println("Sleep Mode Test");
    serial.println("==================");
    serial.println("");
    serial.println("Press button on D2 to wake from sleep");
    serial.println("");

    // Button interrupt handler
    fn button_handler() {
        BUTTON_PRESSED.store(true, Ordering::SeqCst);
    }

    // Attach interrupt to D2 (FALLING edge when button pressed)
    attach_interrupt(ExternalInterrupt::Int0, InterruptMode::Falling, button_handler);

    let mut cycle = 0u8;

    loop {
        cycle = cycle.wrapping_add(1);

        // Blink LED to show we're awake
        serial.write_str("Cycle ");
        print_number(&mut serial, cycle as u32);
        serial.println(": Awake - LED blinking");

        for _ in 0..3 {
            led.set_high();
            delay_ms(200);
            led.set_low();
            delay_ms(200);
        }

        // Determine which sleep mode to use
        let sleep_mode = if cycle % 2 == 0 {
            serial.println("Entering IDLE mode...");
            SleepMode::Idle
        } else {
            serial.println("Entering POWER_DOWN mode...");
            SleepMode::PowerDown
        };

        serial.println("(Press button to wake)");
        serial.println("");

        // Small delay to let serial finish
        delay_ms(100);

        // Clear button flag
        BUTTON_PRESSED.store(false, Ordering::SeqCst);

        // Enter sleep mode
        Sleep::sleep_mode(sleep_mode);

        // *** CPU SLEEPS HERE ***
        // Execution continues after wake-up interrupt

        // Check if we woke from button press
        if BUTTON_PRESSED.load(Ordering::SeqCst) {
            serial.println("");
            serial.println("*** WOKE UP FROM BUTTON PRESS ***");
            serial.println("");
            delay_ms(500);
        }
    }
}

// Helper function to delay in milliseconds
fn delay_ms(ms: u32) {
    let start = millis();
    while millis().wrapping_sub(start) < ms {
        unsafe { core::arch::asm!("nop"); }
    }
}

// Helper function to print numbers
fn print_number(serial: &mut arduino_uno::Serial, mut n: u32) {
    if n == 0 {
        serial.write_str("0");
        return;
    }

    let mut buf = [0u8; 10];
    let mut i = 0;

    while n > 0 {
        buf[i] = b'0' + (n % 10) as u8;
        n /= 10;
        i += 1;
    }

    // Print in reverse order
    while i > 0 {
        i -= 1;
        serial.write_byte(buf[i]);
    }
}
