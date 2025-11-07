//! Servo motor control library
//!
//! This module provides interrupt-driven control for RC servo motors using Timer1.
//! Standard servos expect a pulse every 20ms (50Hz), with pulse widths
//! ranging from ~1ms (0°) to ~2ms (180°).
//!
//! The Arduino Servo library uses:
//! - MIN_PULSE_WIDTH: 544 microseconds (0°)
//! - MAX_PULSE_WIDTH: 2400 microseconds (180°)
//! - REFRESH_INTERVAL: 20000 microseconds (20ms)
//!
//! This implementation uses Timer1 with interrupts to generate servo pulses
//! in the background without blocking the main program.

use core::ptr::{read_volatile, write_volatile};
use core::cell::Cell;
use critical_section::Mutex;

// Timer1 registers (16-bit timer)
const TCCR1A: *mut u8 = 0x80 as *mut u8;
const TCCR1B: *mut u8 = 0x81 as *mut u8;
const TCNT1H: *mut u8 = 0x85 as *mut u8;
const TCNT1L: *mut u8 = 0x84 as *mut u8;
const OCR1AH: *mut u8 = 0x89 as *mut u8;
const OCR1AL: *mut u8 = 0x88 as *mut u8;
const TIMSK1: *mut u8 = 0x6F as *mut u8;
const TIFR1: *mut u8 = 0x36 as *mut u8;

// Servo timing constants (in microseconds)
const MIN_PULSE_WIDTH: u16 = 544;   // Minimum pulse width (0 degrees)
const MAX_PULSE_WIDTH: u16 = 2400;  // Maximum pulse width (180 degrees)
const DEFAULT_PULSE_WIDTH: u16 = 1500; // Center position (90 degrees)
const REFRESH_INTERVAL: u32 = 20000; // Standard servo refresh interval (20ms)
const SERVOS_PER_TIMER: usize = 12; // Maximum servos on one timer

// Timer1 prescaler: 8 at 16MHz gives 0.5us per tick
const TIMER_PRESCALER: u16 = 8;
const CPU_FREQ_MHZ: u16 = 16;
// Calculate ticks per microsecond: (CPU_FREQ_MHZ * 1000000 / TIMER_PRESCALER) / 1000000
// = CPU_FREQ_MHZ / TIMER_PRESCALER = 16 / 8 = 2 ticks per microsecond
const TICKS_PER_US: u16 = CPU_FREQ_MHZ / TIMER_PRESCALER;

/// Servo state
#[derive(Clone, Copy)]
struct ServoState {
    pin: u8,
    pulse_width: u16,  // In microseconds
    min_pulse: u16,    // Minimum pulse width
    max_pulse: u16,    // Maximum pulse width
    is_attached: bool,
}

/// Global servo instances
static SERVOS: Mutex<Cell<[Option<ServoState>; SERVOS_PER_TIMER]>> =
    Mutex::new(Cell::new([None; SERVOS_PER_TIMER]));

static SERVO_COUNT: Mutex<Cell<usize>> = Mutex::new(Cell::new(0));
static TIMER_INITIALIZED: Mutex<Cell<bool>> = Mutex::new(Cell::new(false));
static CURRENT_SERVO_INDEX: Mutex<Cell<usize>> = Mutex::new(Cell::new(0));
static SERVO_FRAME_CYCLE_ACTIVE: Mutex<Cell<bool>> = Mutex::new(Cell::new(false));

/// Servo motor controller
///
/// Controls RC servo motors using Timer1 interrupts. Supports up to 12 servos.
/// Uses interrupt-driven PWM generation for smooth, non-blocking operation.
///
/// # Example
/// ```no_run
/// use arduino_uno::Servo;
///
/// let mut servo = Servo::new();
/// servo.attach(9);  // Attach to pin 9
/// servo.write(90);  // Move to 90 degrees
/// // Servo continues to hold position via interrupts
/// ```
pub struct Servo {
    index: usize,
}

impl Servo {
    /// Create a new Servo instance
    pub fn new() -> Self {
        let index = critical_section::with(|cs| {
            let count = SERVO_COUNT.borrow(cs).get();
            if count >= SERVOS_PER_TIMER {
                panic!("Maximum number of servos exceeded");
            }

            let idx = count;
            SERVO_COUNT.borrow(cs).set(count + 1);

            // Initialize servo state
            let mut servos = SERVOS.borrow(cs).get();
            servos[idx] = Some(ServoState {
                pin: 0,
                pulse_width: DEFAULT_PULSE_WIDTH,
                min_pulse: MIN_PULSE_WIDTH,
                max_pulse: MAX_PULSE_WIDTH,
                is_attached: false,
            });
            SERVOS.borrow(cs).set(servos);

            idx
        });

        Self { index }
    }

    /// Attach servo to a pin
    pub fn attach(&mut self, pin: u8) -> u8 {
        self.attach_with_limits(pin, MIN_PULSE_WIDTH, MAX_PULSE_WIDTH)
    }

    /// Attach servo to a pin with custom pulse width limits
    pub fn attach_with_limits(&mut self, pin: u8, min: u16, max: u16) -> u8 {
        critical_section::with(|cs| {
            // Initialize timer if needed
            if !TIMER_INITIALIZED.borrow(cs).get() {
                init_timer1_for_servos();
                TIMER_INITIALIZED.borrow(cs).set(true);
            }

            let mut servos = SERVOS.borrow(cs).get();
            if let Some(servo) = &mut servos[self.index] {
                servo.pin = pin;
                servo.min_pulse = min;
                servo.max_pulse = max;
                servo.is_attached = true;

                // Set pin as output
                crate::pin_mode(pin, crate::OUTPUT);

                // Start servo frame generation if not already active
                if !SERVO_FRAME_CYCLE_ACTIVE.borrow(cs).get() {
                    SERVO_FRAME_CYCLE_ACTIVE.borrow(cs).set(true);
                    start_servo_cycle();
                }
            }
            SERVOS.borrow(cs).set(servos);
        });

        pin
    }

    /// Detach servo from its pin
    pub fn detach(&mut self) {
        critical_section::with(|cs| {
            let mut servos = SERVOS.borrow(cs).get();
            if let Some(servo) = &mut servos[self.index] {
                servo.is_attached = false;
                crate::digital_write(servo.pin, crate::PinState::Low);
            }
            SERVOS.borrow(cs).set(servos);
        });
    }

    /// Write angle to servo (0-180 degrees)
    pub fn write(&mut self, angle: u16) {
        let angle = angle.min(180);

        let pulse_width = critical_section::with(|cs| {
            let servos = SERVOS.borrow(cs).get();
            if let Some(servo) = &servos[self.index] {
                // Map angle (0-180) to pulse width using servo's min/max limits
                let range = (servo.max_pulse - servo.min_pulse) as u32;
                Some(servo.min_pulse + ((angle as u32 * range) / 180) as u16)
            } else {
                None
            }
        });

        if let Some(pw) = pulse_width {
            self.write_microseconds(pw);
        }
    }

    /// Write pulse width to servo in microseconds
    pub fn write_microseconds(&mut self, microseconds: u16) {
        critical_section::with(|cs| {
            let mut servos = SERVOS.borrow(cs).get();
            if let Some(servo) = &mut servos[self.index] {
                // Constrain to servo's min/max limits
                servo.pulse_width = microseconds.max(servo.min_pulse).min(servo.max_pulse);
            }
            SERVOS.borrow(cs).set(servos);
        });
    }

    /// Read current angle from servo
    pub fn read(&self) -> u16 {
        critical_section::with(|cs| {
            let servos = SERVOS.borrow(cs).get();
            if let Some(servo) = &servos[self.index] {
                // Map pulse width back to angle using servo's min/max limits
                let range = (servo.max_pulse - servo.min_pulse) as u32;
                let offset = (servo.pulse_width - servo.min_pulse) as u32;
                ((offset * 180) / range) as u16
            } else {
                0
            }
        })
    }

    /// Read current pulse width in microseconds
    pub fn read_microseconds(&self) -> u16 {
        critical_section::with(|cs| {
            let servos = SERVOS.borrow(cs).get();
            if let Some(servo) = &servos[self.index] {
                servo.pulse_width
            } else {
                DEFAULT_PULSE_WIDTH
            }
        })
    }

    /// Check if servo is attached to a pin
    pub fn attached(&self) -> bool {
        critical_section::with(|cs| {
            let servos = SERVOS.borrow(cs).get();
            servos[self.index]
                .as_ref()
                .map(|s| s.is_attached)
                .unwrap_or(false)
        })
    }
}

impl Default for Servo {
    fn default() -> Self {
        Self::new()
    }
}

/// Initialize Timer1 for servo pulse generation
fn init_timer1_for_servos() {
    unsafe {
        // Stop timer
        write_volatile(TCCR1B, 0);

        // Clear any pending interrupt flags
        write_volatile(TIFR1, 0xFF);  // Write 1 to clear flags

        // Set CTC mode (Clear Timer on Compare Match) - WGM12 = 1
        let tccr1a = read_volatile(TCCR1A);
        write_volatile(TCCR1A, tccr1a & 0xFC);  // WGM11:10 = 00

        // Set prescaler to TIMER_PRESCALER (8): CS11 = 1, CS12:CS10 = 0
        // This gives us TICKS_PER_US = CPU_FREQ_MHZ / TIMER_PRESCALER = 2 ticks/microsecond
        write_volatile(TCCR1B, (1 << 3) | (1 << 1));  // WGM12 = 1, CS11 = 1 (prescaler 8)

        // Enable Timer1 Compare A interrupt
        let timsk1 = read_volatile(TIMSK1);
        write_volatile(TIMSK1, timsk1 | (1 << 1));  // OCIE1A = 1
    }
}

/// Start the servo refresh cycle
fn start_servo_cycle() {
    critical_section::with(|cs| {
        CURRENT_SERVO_INDEX.borrow(cs).set(0);

        unsafe {
            // Set compare match for first servo
            let servos = SERVOS.borrow(cs).get();
            if let Some(servo) = &servos[0] {
                if servo.is_attached {
                    // Set pin high to start pulse
                    crate::digital_write(servo.pin, crate::PinState::High);

                    // Set timer to fire when pulse should end
                    let ticks = servo.pulse_width * TICKS_PER_US;
                    write_volatile(OCR1AH, (ticks >> 8) as u8);
                    write_volatile(OCR1AL, (ticks & 0xFF) as u8);

                    // Reset timer
                    write_volatile(TCNT1H, 0);
                    write_volatile(TCNT1L, 0);
                }
            }
        }
    });
}

/// Timer1 Compare A interrupt handler for servo pulse generation
#[no_mangle]
pub unsafe extern "avr-interrupt" fn __vector_11() {
    critical_section::with(|cs| {
        let current_index = CURRENT_SERVO_INDEX.borrow(cs).get();
        let servos = SERVOS.borrow(cs).get();

        // End current servo's pulse
        if let Some(servo) = &servos[current_index] {
            if servo.is_attached {
                crate::digital_write(servo.pin, crate::PinState::Low);
            }
        }

        // Move to next servo
        let mut next_index = current_index + 1;

        // Find next attached servo or wrap around
        while next_index < SERVOS_PER_TIMER {
            if let Some(servo) = &servos[next_index] {
                if servo.is_attached {
                    // Start pulse for next servo
                    crate::digital_write(servo.pin, crate::PinState::High);

                    // Set timer for pulse width
                    let ticks = servo.pulse_width * TICKS_PER_US;
                    write_volatile(OCR1AH, (ticks >> 8) as u8);
                    write_volatile(OCR1AL, (ticks & 0xFF) as u8);
                    write_volatile(TCNT1H, 0);
                    write_volatile(TCNT1L, 0);

                    CURRENT_SERVO_INDEX.borrow(cs).set(next_index);
                    return;
                }
            }
            next_index += 1;
        }

        // All servos done, wait for next frame
        // Calculate time remaining in 20ms frame
        let total_pulse_time: u32 = (0..SERVOS_PER_TIMER)
            .filter_map(|i| servos[i])
            .filter(|s| s.is_attached)
            .map(|s| s.pulse_width as u32)
            .sum();

        let remaining_time = REFRESH_INTERVAL.saturating_sub(total_pulse_time);
        let ticks = (remaining_time * TICKS_PER_US as u32) as u16;

        write_volatile(OCR1AH, (ticks >> 8) as u8);
        write_volatile(OCR1AL, (ticks & 0xFF) as u8);
        write_volatile(TCNT1H, 0);
        write_volatile(TCNT1L, 0);

        // Restart cycle
        CURRENT_SERVO_INDEX.borrow(cs).set(0);

        // Start first servo on next interrupt
    });
}
