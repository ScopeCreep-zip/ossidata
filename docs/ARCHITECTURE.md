---
tags:
  - deepwiki/ossidata
  - architecture
  - design
  - technical
  - embedded
---

# Ossidata Architecture

**Last Updated**: 2025-11-05

This document describes the technical architecture of the Ossidata Rust SDK for Arduino. It explains how the various components work together to provide a safe, ergonomic interface for embedded programming.

## Table of Contents

- [Overview](#overview)
- [Architecture Layers](#architecture-layers)
- [Crate Structure](#crate-structure)
- [Type-State Pattern](#type-state-pattern)
- [Build & Flash Pipeline](#build--flash-pipeline)
- [Memory Management](#memory-management)
- [Safety Guarantees](#safety-guarantees)

## Overview

Ossidata is built on a layered architecture that transforms low-level hardware registers into high-level, type-safe Rust APIs.

```mermaid
graph TB
    subgraph "User Space"
        A[Your Application Code]
    end

    subgraph "SDK Layer"
        B[ossidata Crate<br/>High-Level API & Prelude]
    end

    subgraph "Board Support"
        C[Board Support Packages<br/>ossidata-uno, ossidata-mega, etc.]
    end

    subgraph "Hardware Abstraction"
        D[ossidata-hal<br/>GPIO, Serial, PWM, ADC, I2C, SPI, LCD, RTC]
        E[embedded-hal Traits<br/>Standard Interfaces]
    end

    subgraph "Low-Level Access"
        F[Peripheral Access Crates<br/>avr-device, atsamd-pac]
        G[Hardware Registers<br/>Direct Memory-Mapped I/O]
    end

    A --> B
    B --> C
    C --> D
    D --> E
    E --> F
    F --> G

    style A fill:#98FB98
    style B fill:#87CEEB
    style C fill:#FFD700
    style D fill:#FFB6C1
    style G fill:#696969,color:#FFF
```

### Design Philosophy

1. **Safety First**: Leverage Rust's type system to prevent errors at compile-time
2. **Zero-Cost Abstractions**: All abstractions compile away to direct register access
3. **Progressive Disclosure**: Simple things are easy, complex things are possible
4. **Ecosystem Integration**: Built on `embedded-hal` for compatibility

## Architecture Layers

### Layer 1: Hardware Registers

At the bottom, we have direct hardware access through memory-mapped I/O:

```rust
// Direct register access (PAC level)
unsafe {
    (*PORTB::ptr()).ddrb.write(|w| w.bits(0xFF));  // Configure all pins as output
    (*PORTB::ptr()).portb.write(|w| w.bits(0x20)); // Set pin 5 high
}
```

**Characteristics:**
- `unsafe` required
- Direct memory access
- Zero abstraction overhead
- Easy to make mistakes

### Layer 2: Peripheral Access Crates (PAC)

PACs provide type-safe register access:

```rust
// PAC level - safer, but still low-level
let peripherals = avr_device::atmega328p::Peripherals::take().unwrap();
peripherals.PORTB.ddrb.write(|w| w.pb5().set_bit());
```

**Characteristics:**
- Type-safe register access
- Still requires hardware knowledge
- Minimal documentation
- Board-agnostic

### Layer 3: Hardware Abstraction Layer (HAL)

Our HAL implements `embedded-hal` traits and provides Arduino-friendly abstractions:

```rust
// HAL level - familiar Arduino concepts
let mut pin13 = pins.d13.into_output();
pin13.set_high();
```

**Characteristics:**
- Arduino-style pin names
- Type-state pattern for safety
- Implements standard traits
- Board-specific implementations

### Layer 4: Board Support Packages (BSP)

BSPs configure everything for specific boards:

```rust
// BSP level - board-specific setup
use arduino_uno::{Peripherals, Serial, Delay};

let peripherals = Peripherals::take().unwrap();
let mut led = peripherals.pins.d13.into_output();  // Built-in LED
let mut serial = Serial::new(9600);                 // USB serial
```

**Characteristics:**
- Pre-configured for specific hardware
- Pin mappings match board silk screen
- Simplified initialization
- Board-specific features

### Layer 5: User Application

Your code uses the ergonomic APIs:

```rust
// Application level - simple and safe!
#![no_std]
#![no_main]

use arduino_uno::{Peripherals, Delay};

#[avr_device::entry]
fn main() -> ! {
    let peripherals = Peripherals::take().unwrap();
    let mut led = peripherals.pins.d13.into_output();
    let mut delay = Delay::new();

    loop {
        led.set_high();
        delay.delay_ms(1000);
        led.set_low();
        delay.delay_ms(1000);
    }
}
```

## Crate Structure

```mermaid
graph TD
    subgraph "Published Crates"
        A[ossidata<br/>Main SDK Crate]
        B[ossidata-core<br/>Shared Types]
    end

    subgraph "Board Support Packages"
        C[arduino-uno<br/>ATmega328P]
        D[arduino-mega<br/>ATmega2560]
        E[arduino-due<br/>ARM SAM3X]
    end

    subgraph "Hardware Abstraction"
        F[ossidata-hal<br/>Generic HAL]
    end

    subgraph "External Dependencies"
        G[avr-device<br/>AVR PAC]
        H[atsamd-pac<br/>ARM PAC]
        I[embedded-hal<br/>Standard Traits]
    end

    A --> C
    A --> D
    A --> E
    C --> F
    D --> F
    E --> F
    F --> B
    F --> I
    C --> G
    E --> H

    style A fill:#98FB98
    style B fill:#FFE4B5
    style I fill:#87CEEB
```

### Crate Responsibilities

| Crate | Purpose | Dependencies | Status |
|-------|---------|--------------|--------|
| **ossidata** | User-facing API, re-exports | All BSPs | 📋 Planned |
| **ossidata-core** | Common types, no hardware deps | None | ✅ Implemented |
| **arduino-uno** | Arduino Uno BSP | avr-device, ossidata-core | 🚧 In Progress (75%) |
| **arduino-mega** | Arduino Mega BSP | avr-device, ossidata-core | 📋 Planned |
| **arduino-due** | Arduino Due BSP | atsamd-pac, ossidata-core | 📋 Planned |
| **ossidata-hal** | Generic HAL implementations | embedded-hal | 📋 Future |

## Type-State Pattern

One of Ossidata's key safety features is the **type-state pattern** for GPIO pins.

### The Problem

In C/C++ Arduino:

```cpp
pinMode(13, OUTPUT);
digitalWrite(13, HIGH);  // OK

pinMode(13, INPUT);
digitalWrite(13, HIGH);  // Still compiles! But wrong!
```

### The Solution

In Rust, pin modes are tracked in the **type system**:

```rust
let pin = pins.d13.into_output();  // Pin<D13, Output>
pin.set_high();  // ✅ OK - compiler knows this is an output pin

let pin = pin.into_input();        // Pin<D13, Input>
pin.set_high();  // ❌ COMPILE ERROR - inputs can't be set!
```

### How It Works

```mermaid
stateDiagram-v2
    [*] --> Unconfigured: Pin<D13, Unconfigured>

    Unconfigured --> Output: .into_output()
    Unconfigured --> Input: .into_input()
    Unconfigured --> InputPullUp: .into_input_pullup()

    Output --> Input: .into_input()
    Output --> InputPullUp: .into_input_pullup()

    Input --> Output: .into_output()
    Input --> InputPullUp: .into_input_pullup()

    InputPullUp --> Output: .into_output()
    InputPullUp --> Input: .into_input()

    note right of Output
        Methods:
        - set_high()
        - set_low()
        - toggle()
    end note

    note right of Input
        Methods:
        - is_high()
        - is_low()
    end note

    note right of InputPullUp
        Methods:
        - is_high()
        - is_low()
        (with internal pull-up resistor)
    end note
```

### Type-State Implementation

```rust
// Pin mode marker types (zero-sized!)
pub struct Output;
pub struct Input;
pub struct InputPullUp;

// Pin generic over number and mode
pub struct Pin<const N: u8, MODE> {
    _mode: PhantomData<MODE>,  // Zero runtime size!
}

// Only Output pins can be set
impl<const N: u8> Pin<N, Output> {
    pub fn set_high(&mut self) {
        // Safe: compiler guarantees this is an output pin
        unsafe { /* set register bit */ }
    }
}

// Only Input pins can be read
impl<const N: u8> Pin<N, Input> {
    pub fn is_high(&self) -> bool {
        // Safe: compiler guarantees this is an input pin
        unsafe { /* read register bit */ }
    }
}
```

**Key Insight**: The mode information exists only at compile-time and has **zero runtime cost**!

## Build & Flash Pipeline

```mermaid
flowchart TD
    A[Source Code<br/>main.rs] --> B[Cargo Build]

    B --> C{Target?}

    C -->|AVR| D[rustc with<br/>AVR backend]
    C -->|ARM| E[rustc with<br/>ARM backend]

    D --> F[ELF Binary<br/>avr-none]
    E --> G[ELF Binary<br/>thumbv7em]

    F --> H[avr-objcopy<br/>ELF → HEX]
    G --> I[arm-objcopy<br/>ELF → BIN]

    H --> J[firmware.hex]
    I --> K[firmware.bin]

    J --> L{OS Detection}
    K --> L

    L -->|macOS| M[flash-macos.sh<br/>Terminal.app]
    L -->|Linux| N[flash-linux.sh<br/>xterm]
    L -->|Windows| O[flash-windows.bat<br/>cmd.exe]

    M --> P[flash-impl.sh]
    N --> P
    O --> P

    P --> Q[avrdude/openocd<br/>Upload to Board]

    Q --> R[Arduino Runs!]

    style A fill:#FFE4B5
    style F fill:#87CEEB
    style G fill:#FFB6C1
    style R fill:#98FB98
    style Q fill:#FFA500
```

### Build Configuration

Each BSP has its own `.cargo/config.toml`:

```toml
[build]
target = "avr-atmega328p.json"  # Custom target spec

[target.avr-atmega328p]
runner = "../../flash.sh"        # Custom flash script

[unstable]
build-std = ["core"]             # Build core library from source
```

### Flash System Architecture

Our cross-platform flash system solves two problems:

1. **avrdude minimal output** (solved with `-q -q` flags for avrdude 8.1+ compatibility)
2. **Claude Code terminal hanging** (solved with external terminals)

```mermaid
sequenceDiagram
    participant User
    participant CargoRunner as Cargo Runner
    participant FlashSh as flash.sh
    participant OSScript as OS-Specific Script
    participant Terminal as External Terminal
    participant Arduino

    User->>CargoRunner: cargo run
    CargoRunner->>FlashSh: Run flash script
    FlashSh->>FlashSh: Detect OS (uname)
    FlashSh->>OSScript: Launch OS-specific script

    alt macOS
        OSScript->>Terminal: Open Terminal.app
    else Linux
        OSScript->>Terminal: Open xterm
    else Windows
        OSScript->>Terminal: Open cmd.exe
    end

    Terminal->>Terminal: Run flash-impl.sh
    Terminal->>Arduino: avrdude -q -q flash
    Arduino-->>Terminal: Flash complete
    Terminal->>Terminal: Write DONE status file
    Terminal->>Terminal: Close automatically

    OSScript->>OSScript: Monitor status file
    OSScript-->>FlashSh: Flash complete
    FlashSh-->>CargoRunner: Success
    CargoRunner-->>User: Done (~15 seconds)

    Note over User,Arduino: Claude Code remains<br/>responsive throughout!
```

See [FLASHING_SOLUTION.md](FLASHING_SOLUTION.md) for complete details.

## Memory Management

Embedded systems have severe memory constraints:

| Board | Flash | RAM | Notes |
|-------|-------|-----|-------|
| Arduino Uno | 32 KB | 2 KB | Very constrained! |
| Arduino Mega | 256 KB | 8 KB | More comfortable |
| Arduino Due | 512 KB | 96 KB | Plenty of space |

### Memory Layout (AVR)

```mermaid
graph TD
    subgraph "Flash Memory (32KB)"
        A[Bootloader<br/>2KB]
        B[Your Program<br/>~2-20KB]
        C[Unused<br/>10-28KB]
    end

    subgraph "RAM (2KB)"
        D[Stack<br/>~512B]
        E[Static Variables<br/>~200B]
        F[Heap<br/>NONE by default]
        G[Free<br/>~1.3KB]
    end

    style A fill:#FF6B6B
    style B fill:#4ECDC4
    style C fill:#95E1D3
    style D fill:#FFA07A
    style E fill:#FFB6C1
    style F fill:#DDD
    style G fill:#90EE90
```

### No Heap Allocation

By default, Ossidata uses **no heap allocation**:

```rust
// ❌ These DON'T work (require heap)
let s = String::from("hello");    // Needs heap
let v = vec![1, 2, 3];            // Needs heap

// ✅ These DO work (no heap)
let s = "hello";                  // String slice (in flash)
let arr = [1, 2, 3];              // Fixed-size array (on stack)
let mut buf = [0u8; 64];          // Fixed-size buffer
```

### Memory Optimization

```toml
# Cargo.toml profile for AVR
[profile.release]
opt-level = "z"          # Optimize for size
lto = true               # Link-time optimization
codegen-units = 1        # Better optimization
```

**Binary Sizes** (Arduino Uno examples):
- **blink**: 844 bytes
- **hello_world**: 2.6 KB
- **button**: 728 bytes
- **serial_echo**: 1.2 KB
- **led_pattern**: 1.5 KB

These are **tiny** compared to Arduino C++ equivalents!

## Safety Guarantees

### Compile-Time Guarantees

Ossidata provides several safety guarantees at **compile-time**:

```mermaid
graph LR
    A[Rust Code] --> B{Compile-Time Checks}

    B --> C[Type Safety<br/>Can't use pin wrong]
    B --> D[Memory Safety<br/>No buffer overflows]
    B --> E[Thread Safety<br/>No data races]
    B --> F[Lifetime Safety<br/>No dangling pointers]

    C --> G[Safe Binary]
    D --> G
    E --> G
    F --> G

    G --> H[Upload to Arduino]

    style B fill:#87CEEB
    style G fill:#98FB98
```

### Examples

**1. Type Safety**

```rust
let mut pin = pins.d13.into_output();
pin.is_high();  // ❌ Compile error: outputs can't be read
```

**2. Ownership & Borrowing**

```rust
let peripherals = Peripherals::take().unwrap();
let peripherals2 = Peripherals::take();  // ❌ Returns None!
// Can't have two instances - compiler enforces singleton
```

**3. Memory Safety**

```rust
let mut buffer = [0u8; 10];
buffer[15] = 42;  // ❌ Compile error: index out of bounds
```

**4. No Undefined Behavior**

In C/C++:
```cpp
int* ptr = nullptr;
*ptr = 42;  // ☠️ Undefined behavior (segfault on PC, weird bugs on Arduino)
```

In Rust:
```rust
let ptr: Option<&mut i32> = None;
*ptr.unwrap() = 42;  // ⚠️ Explicit panic (can catch in debug)
```

### Runtime Safety

For cases that can't be checked at compile-time:

```rust
// Explicit Result types
pub fn serial_read() -> Result<u8, Error> {
    if data_available() {
        Ok(read_byte())
    } else {
        Err(Error::NoData)
    }
}

// Usage
match serial.read() {
    Ok(byte) => process(byte),
    Err(e) => handle_error(e),
}
```

## Interrupt Handling (Future)

```mermaid
sequenceDiagram
    participant Main as Main Loop
    participant ISR as Interrupt Handler
    participant Hardware as Hardware

    Main->>Main: Running normally
    Hardware->>ISR: Interrupt triggered!
    ISR->>ISR: Execute handler (fast!)
    ISR->>Main: Return
    Main->>Main: Continue execution

    Note over ISR: Must be fast!<br/>No allocations!<br/>Critical section for shared data
```

Planned interrupt API (future enhancement):

```rust
// Future: Safe interrupt handling
interrupts::on_pin_change(pins.d2, || {
    // Interrupt handler
    LED.toggle();  // Must use atomic operations
});
```

## Cross-Platform Support

```mermaid
graph TD
    A[Ossidata SDK] --> B{Target Architecture}

    B -->|AVR| C[AVR Backend]
    B -->|ARM| D[ARM Backend]
    B -->|ESP32| E[Xtensa Backend]

    C --> F[Arduino Uno<br/>Arduino Mega<br/>Arduino Nano]
    D --> G[Arduino Due<br/>Arduino Zero<br/>MKR Series]
    E --> H[Arduino Nano 33 IoT<br/>Arduino Nano ESP32]

    style A fill:#98FB98
    style C fill:#87CEEB
    style D fill:#FFB6C1
    style E fill:#FFD700
```

### Architecture Differences

| Feature | AVR (8-bit) | ARM Cortex-M (32-bit) |
|---------|-------------|----------------------|
| **Word Size** | 8-bit | 32-bit |
| **Flash** | 2-256 KB | 32-512 KB |
| **RAM** | 0.5-16 KB | 4-96 KB |
| **Clock** | 8-20 MHz | 48-120 MHz |
| **FPU** | No | Some models |
| **DMA** | No | Yes |
| **USB** | External | Built-in |

## Performance

### Zero-Cost Abstractions

Ossidata compiles to **identical assembly** as hand-written C:

```rust
// Rust
pin.set_high();
```

```asm
; Generated assembly (AVR)
sbi 0x05, 5  ; Set bit 5 in PORTB register
```

This is **exactly the same** as Arduino C++!

### Benchmarks (Coming Soon)

We'll provide benchmarks comparing:
- Binary size (Rust vs C++)
- RAM usage (Rust vs C++)
- Execution speed (Rust vs C++)
- Compile time (Rust vs C++)

Early indications show Rust is **equal or better** in all metrics.

## Extensibility

### Adding New Boards

```mermaid
flowchart LR
    A[Choose Board] --> B[Create BSP Crate]
    B --> C[Define Pin Mapping]
    C --> D[Implement HAL Traits]
    D --> E[Add Examples]
    E --> F[Test on Hardware]
    F --> G[Document & Publish]

    style A fill:#FFE4B5
    style G fill:#98FB98
```

See [Contributing Guide](../CONTRIBUTING.md) for details.

### Adding New Peripherals

To add support for new peripherals (I2C, SPI, etc.):

1. Implement `embedded-hal` traits
2. Add to HAL crate
3. Expose in BSPs
4. Add examples
5. Test on hardware

## Current Implementation Status

The Arduino Uno BSP is currently **82% complete** with the following features implemented:

**Core I/O:**
- ✅ **GPIO**: Digital input/output with type-state pattern
- ✅ **PWM**: Pulse-width modulation on 6 channels (D3, D5, D6, D9, D10, D11)
- ✅ **ADC**: Analog-to-digital conversion with 10-bit resolution on A0-A5

**Communication:**
- ✅ **Serial Communication**: UART with formatted output
- ✅ **I2C**: Two-wire interface for sensor communication (master mode, 100kHz)
- ✅ **SPI**: Serial peripheral interface for high-speed devices (master mode, transaction-based)

**Peripherals:**
- ✅ **LCD**: Character LCD display support (HD44780 via I2C backpack)
- ✅ **RTC**: Real-time clock integration (DS1307/DS3231)

**Timing:**
- ✅ **Timing**: millis() and micros() functions for precise timing using Timer0

**Advanced Features:**
- ✅ **Interrupts**: External interrupts on D2/D3 with RISING/FALLING/CHANGE modes
- ✅ **EEPROM**: Read/write persistent storage (1024 bytes)
- ✅ **Tone Generation**: Audio output with tone/tone_duration/no_tone using Timer2
- ✅ **Pulse Measurement**: pulse_in/pulse_in_long for sensors (ultrasonic, RC)
- ✅ **Shift Registers**: shift_out/shift_in for I/O expansion (74HC595/74HC165)

**Planned:**
- ⏳ **delayMicroseconds()**: Microsecond delays
- ⏳ **Watchdog Timer**: System reset and supervision
- ⏳ **Sleep Modes**: Power management

## Future Architecture

### Planned Improvements (v0.2+)

```mermaid
graph TD
    A[Current: Blocking APIs] --> B[Add Non-Blocking]
    B --> C[Add Interrupts]
    C --> D[Add DMA]
    D --> E[Add async/await]
    E --> F[Embassy Integration]

    style A fill:#87CEEB
    style F fill:#98FB98
```

1. **Non-blocking I/O**: For better responsiveness
2. **Interrupt Support**: Safe interrupt handling
3. **DMA** (ARM only): Efficient data transfers
4. **async/await**: Modern concurrency (via Embassy)
5. **defmt**: Efficient logging for debugging

## References

- **Rust Embedded Book**: https://docs.rust-embedded.org/book/
- **embedded-hal Documentation**: https://docs.rs/embedded-hal/
- **AVR HAL Reference**: https://github.com/Rahix/avr-hal
- **ARM Cortex-M Reference**: https://docs.rust-embedded.org/cortex-m/

## Summary

Ossidata's architecture provides:

- ✅ **Layered abstraction** from registers to high-level APIs
- ✅ **Type-state pattern** for compile-time safety
- ✅ **Zero-cost abstractions** with no runtime overhead
- ✅ **Cross-platform support** for multiple architectures
- ✅ **Memory efficiency** optimized for embedded constraints
- ✅ **Safety guarantees** preventing entire classes of bugs

All while maintaining **full compatibility** with existing embedded Rust ecosystem!

---

**Next**: [API Reference](API_REFERENCE.md) | [Getting Started](GETTING_STARTED.md) | [Flashing Solution](FLASHING_SOLUTION.md)
