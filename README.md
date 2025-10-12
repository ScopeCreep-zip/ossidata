---
tags:
  - deepwiki/ossidata
  - rust
  - arduino
  - embedded
  - sdk
---

# Ossidata - Rust SDK for Arduino

A safe, ergonomic, and modern Rust SDK for programming Arduino boards.

[![deepwiki](https://img.shields.io/badge/docs-deepwiki-blue.svg)](https://github.com/ScopeCreep-zip/ossidata)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE)
[![Status](https://img.shields.io/badge/status-in%20development-yellow)](https://github.com/ScopeCreep-zip/ossidata)

---

## 🎯 Project Goals

Build a comprehensive Rust SDK that:
- 🦀 Provides 100% safe Rust alternatives to Arduino C++
- 🔒 Leverages Rust's type system for compile-time safety
- ⚡ Achieves zero-cost abstractions (no performance penalty)
- 📦 Integrates with the embedded-hal ecosystem
- 🎨 Offers an ergonomic, intuitive API

## 🚀 Quick Start

**Coming Soon!** We're currently in active development.

```rust
// Future usage example
#![no_std]
#![no_main]

use ossidata::prelude::*;

#[ossidata::entry]
fn main() -> ! {
    let dp = Peripherals::take().unwrap();
    let mut led = dp.pins.d13.into_output();

    loop {
        led.set_high();
        delay_ms(1000);
        led.set_low();
        delay_ms(1000);
    }
}
```

## 📋 Supported Boards

| Board | Status | MCU | Notes |
|-------|--------|-----|-------|
| Arduino Uno | 🚧 In Progress | ATmega328P | Primary target |
| Arduino Mega 2560 | 📋 Planned | ATmega2560 | |
| Arduino Nano | 📋 Planned | ATmega328P | |
| Arduino Due | 📋 Planned | SAM3X8E (ARM) | |
| Arduino Zero | 📋 Planned | SAMD21 (ARM) | |
| ESP32 Arduino | 📋 Planned | ESP32 | |

**Legend**: ✅ Complete | 🚧 In Progress | 📋 Planned

## ✨ Features

### Current (v0.1.0 - In Development)
- ✅ GPIO (Digital I/O) for Arduino Uno
- ✅ Type-safe pin operations with type-state pattern
- ✅ Serial (UART) communication
- ✅ Basic delay functions

### Planned
- 📋 PWM output
- 📋 Analog input (ADC)
- 📋 I2C protocol
- 📋 SPI protocol
- 📋 Timers and interrupts
- 📋 Multi-board support
- 📋 embedded-hal trait implementations

## 📚 Documentation

### User Documentation

| Document | Description | Status |
|----------|-------------|--------|
| **[Getting Started](docs/GETTING_STARTED.md)** | Complete setup guide and first steps | ✅ Complete |
| **[API Reference](docs/API_REFERENCE.md)** | Comprehensive API documentation | ✅ Complete |
| **[Architecture](docs/ARCHITECTURE.md)** | System architecture with diagrams | ✅ Complete |
| **[Flashing Solution](docs/FLASHING_SOLUTION.md)** | Cross-platform flash guide | ✅ Complete |
| **[Contributing Guide](CONTRIBUTING.md)** | How to contribute | 🚧 In Progress |

### Developer Documentation (Internal)

Developer and planning documentation is in [`/agentdocs`](agentdocs/README.md).

**Legend**: ✅ Complete | 🚧 In Progress | 📋 Planned

## 🛠️ Development Status

**Current Phase**: Phase 1 - AVR Foundation 🚧 (50% Complete)
**Previous Phase**: Phase 0.5 - Hardware Validation ✅ COMPLETE
**Target Completion**: 2025-11-07 (4 weeks from Phase 1 start)

### Our First Major Milestone

**Goal**: Create and run a working "Hello World" program on Arduino Uno.

```rust
serial.println("Hello, World!");
```

**Status**: ✅ Complete! The `hello_world` example works on Arduino Uno with `cargo run`.

### Progress Tracker

- [x] Project planning and research (100%)
- [x] Architecture design (100%)
- [x] Success criteria defined (100%)
- [x] Hardware validation - LED blinks! (100%) ✨
- [x] Toolchain validated (nightly-2025-04-27) (100%)
- [x] Workspace setup (100%) ✅
- [x] Core types implementation (100%) ✅
- [x] GPIO for Arduino Uno (100%) ✅
- [x] Serial/UART for Arduino Uno (100%) ✅
- [x] Hello World example (100%) ✅
- [x] Cross-platform flash system (100%) ✅
- [ ] CI/CD configuration (0%)
- [ ] Multi-board support (0%)
- [ ] User documentation (80%)
- [ ] embedded-hal traits (0%)

**Overall Progress**: 50% (GPIO, Serial, Cross-platform Flash System complete, 5 working examples tested)

## 🔧 Building from Source

### Prerequisites

1. **Rust Toolchain** (nightly required for AVR):
   ```bash
   rustup toolchain install nightly-2025-04-27
   rustup component add rust-src --toolchain nightly-2025-04-27
   ```
   ⚠️ **Note**: This exact version has been validated with hardware. See [VALIDATED_TOOLCHAIN.md](agentdocs/VALIDATED_TOOLCHAIN.md)

2. **AVR-GCC Toolchain**:
   - **macOS**: `brew install avr-gcc`
   - **Linux**: `sudo apt-get install gcc-avr avr-libc`
   - **Windows**: Download from Microchip

3. **AVRDUDE** (for flashing):
   - **macOS**: `brew install avrdude`
   - **Linux**: `sudo apt-get install avrdude`
   - **Windows**: Included with Arduino IDE

### Build

```bash
# Clone the repository
git clone https://github.com/ScopeCreep-zip/ossidata.git
cd ossidata

# Build workspace
cargo build --workspace

# Build for Arduino Uno (AVR)
cd boards/arduino-uno
cargo build --release
```

### Flash to Arduino

```bash
# Using cargo run (recommended - won't hang!)
cd boards/arduino-uno
cargo run --release --bin blink

# Or using our helper script
./flash.sh

# Or manually with safe flags (won't hang)
avrdude -p atmega328p -c arduino -P /dev/cu.usbmodem14401 -b 115200 \
  -q -q -D -U flash:w:target/avr-none/release/blink.hex:i
```

**Note**: We use `-q -q` flags with avrdude to prevent terminal hanging (compatible with avrdude 8.1+). See [FLASHING_SOLUTION.md](docs/FLASHING_SOLUTION.md) for details.

## 🤝 Contributing

We welcome contributions! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for details.

### Quick Contribution Guide

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes
4. Ensure CI passes (`cargo test`, `cargo clippy`, `cargo fmt`)
5. Commit your changes (`git commit -m 'Add amazing feature'`)
6. Push to the branch (`git push origin feature/amazing-feature`)
7. Open a Pull Request

## 📖 Design Philosophy

### Type Safety First
```rust
// Compile-time pin mode checking
let pin = pins.d13.into_output();  // Pin<13, Output>
pin.set_high();  // ✅ OK

let pin = pins.d13.into_input();   // Pin<13, Input>
pin.set_high();  // ❌ Compile error!
```

### Zero-Cost Abstractions
All abstractions optimize away to direct register access - no runtime overhead compared to hand-written C code.

### Ecosystem Integration
Built with `embedded-hal` 1.0 compatibility in mind. Trait implementations coming soon!

## 📊 Comparison with Arduino C++

| Feature | Arduino C++ | Ossidata Rust |
|---------|-------------|---------------|
| Memory Safety | ❌ Manual | ✅ Compile-time guaranteed |
| Pin Mode Checking | ❌ Runtime | ✅ Compile-time |
| Peripheral Access | ❌ Global mutable | ✅ Ownership-based |
| Error Handling | ❌ Return codes/void | ✅ Result types |
| Documentation | ✅ Good | ✅ Excellent (rustdoc) |
| Performance | ✅ Excellent | ✅ Equivalent (zero-cost) |

## 🔬 Architecture

```
┌─────────────────────────────────────────────┐
│         User Application Code               │
├─────────────────────────────────────────────┤
│      High-Level API (ossidata crate)       │
├─────────────────────────────────────────────┤
│         Board Support Packages              │
│  (ossidata-uno, ossidata-mega, etc.)       │
├─────────────────────────────────────────────┤
│    Hardware Abstraction Layer (ossidata-hal)│
├─────────────────────────────────────────────┤
│      Peripheral Access Crates (PAC)         │
│  (avr-device, atsamd-hal, etc.)            │
└─────────────────────────────────────────────┘
```

## 📜 License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.

## 🙏 Acknowledgments

This project builds upon the excellent work of:
- The [Rust Embedded Working Group](https://github.com/rust-embedded)
- [Rahix's avr-hal](https://github.com/Rahix/avr-hal)
- The [Arduino team](https://www.arduino.cc/)
- The entire Rust embedded community

## 📞 Contact & Community

- **Issues**: [GitHub Issues](https://github.com/ScopeCreep-zip/ossidata/issues)
- **Discussions**: [GitHub Discussions](https://github.com/ScopeCreep-zip/ossidata/discussions)

---

**Status**: 🚧 Active Development | **Version**: 0.1.0-dev | **Last Updated**: 2025-10-12
