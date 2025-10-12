# Implementation Plan - Ossidata SDK

**Date**: 2025-10-10
**Status**: Phase 1 Starting
**Phase**: Phase 1 - AVR Foundation (Hardware Validated)

This document outlines the concrete, step-by-step implementation plan based on current best practices and tooling as of 2024-2025.

---

## Current Toolchain Status (2024-2025)

### ✅ Good News: Stable Foundation

1. **embedded-hal 1.0 Released!** (2024)
   - ✅ Stable traits - no breaking changes planned for 2.0
   - ✅ Async support available (`embedded-hal-async`)
   - ✅ Full ecosystem adoption underway

2. **avr-hal Mature & Active**
   - ✅ Actively maintained by Rahix
   - ✅ Production-ready `arduino-hal` crate
   - ✅ Template and examples available
   - ✅ `ravedude` tool for easy flashing

3. **Rust 2024 Edition Available**
   - ✅ Stable as of recent releases
   - ✅ Improved error messages and diagnostics

### ⚠️ AVR-Specific Considerations

1. **Nightly Rust Required**
   - **VALIDATED**: `nightly-2025-04-27` (tested on real hardware 2025-10-10)
   - Previous recommendation: `nightly-2024-05-01` (may still work)
   - **Why**: AVR target still requires nightly features
   - **Strategy**: Pin version in `rust-toolchain.toml`

2. **Required Components**
   ```toml
   [toolchain]
   channel = "nightly-2025-04-27"
   components = ["rust-src"]
   ```

3. **AVR-GCC Toolchain**
   - **Critical**: Most package managers have outdated versions
   - **Solution**: Get from Microchip/Atmel official sources
   - Needed for: Assembler and linker

---

## Decision: Build Strategy

### Option A: Build on Top of avr-hal ❌
**Pros**:
- Faster initial development
- Proven, working implementation
- Less reinventing the wheel

**Cons**:
- Learn by using, not by building
- Less control over architecture
- Our API would be a wrapper
- Defeats educational purpose

### Option B: Build From Scratch (Our Choice) ✅
**Pros**:
- Deep understanding of every component
- Full control over API design
- Educational value for us and users
- Can innovate with better patterns
- True ownership of codebase

**Cons**:
- Longer initial development
- More opportunity for bugs
- Need to solve already-solved problems

**Decision**: Build from scratch, but **study avr-hal as reference**
- We own our architecture
- We learn embedded Rust deeply
- We can reference their solutions when stuck
- We can improve upon their design where we see opportunities

---

## Phase 0: Foundation Setup (Week 1)

### Task 1.1: Workspace Initialization (Day 1)

**Create Cargo Workspace Structure**:

```bash
# Initialize git (already done)
# Create workspace root Cargo.toml

# Create crates
cargo new --lib ossidata-core
cargo new --lib ossidata-hal
cargo new --lib ossidata-uno
cargo new --lib ossidata
```

**Root `Cargo.toml`**:
```toml
[workspace]
resolver = "2"
members = [
    "ossidata-core",
    "ossidata-hal",
    "ossidata-uno",
    "ossidata",
]

[workspace.package]
version = "0.1.0"
authors = ["Your Team"]
edition = "2021"
license = "MIT OR Apache-2.0"

[workspace.dependencies]
# Shared dependencies
embedded-hal = "1.0"
critical-section = "1.1"
nb = "1.1"

# AVR specific
avr-device = "0.5"

[profile.dev]
panic = "abort"
lto = false
opt-level = "s"

[profile.release]
panic = "abort"
lto = true
opt-level = "z"      # Optimize for size (critical for AVR)
codegen-units = 1    # Better optimization
strip = true         # Strip symbols
```

**Create `rust-toolchain.toml`** (root):
```toml
[toolchain]
channel = "nightly-2025-04-27"
components = ["rust-src", "rustfmt", "clippy"]
```

### Task 1.2: CI/CD Setup (Day 1-2)

**Create `.github/workflows/ci.yml`**:

```yaml
name: CI

on:
  push:
    branches: [ main, develop ]
  pull_request:
    branches: [ main, develop ]

jobs:
  check:
    name: Check
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@master
        with:
          toolchain: nightly-2025-04-27
          components: rustfmt, clippy
      - name: Check formatting
        run: cargo fmt --all -- --check
      - name: Clippy
        run: cargo clippy --all-targets --all-features

  build-avr:
    name: Build for AVR (Arduino Uno)
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@master
        with:
          toolchain: nightly-2025-04-27
          components: rust-src
      - name: Install AVR GCC
        run: sudo apt-get update && sudo apt-get install -y gcc-avr avr-libc
      - name: Build for AVR
        run: |
          cd ossidata-uno
          cargo build -Z build-std=core --target avr-atmega328p.json --release

  build-examples:
    name: Build Examples
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@master
        with:
          toolchain: nightly-2025-04-27
          components: rust-src
      - name: Install AVR GCC
        run: sudo apt-get update && sudo apt-get install -y gcc-avr avr-libc
      - name: Build all examples
        run: |
          for example in ossidata-uno/examples/*.rs; do
            cargo build -Z build-std=core --target avr-atmega328p.json --release --example $(basename $example .rs)
          done

  docs:
    name: Documentation
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - name: Build docs
        run: cargo doc --no-deps --all-features
        env:
          RUSTDOCFLAGS: "-D warnings"
```

### Task 1.3: Project Documentation (Day 2)

**Create `README.md`** (root):
```markdown
# Ossidata - Rust SDK for Arduino

A safe, ergonomic Rust SDK for programming Arduino boards.

## Supported Boards
- ✅ Arduino Uno (ATmega328P)
- 🚧 Arduino Mega 2560
- 📋 Arduino Nano
- 📋 Arduino Due
- 📋 Arduino Zero

## Features
- 🦀 100% Rust, no C/C++
- 🔒 Memory safe by design
- 🎯 Type-safe pin operations
- ⚡ Zero-cost abstractions
- 📦 embedded-hal compatible

## Quick Start

(Coming soon)

## Documentation

See [Getting Started Guide](docs/GETTING_STARTED.md)

## License

Licensed under either of Apache License, Version 2.0 or MIT license at your option.
```

**Create `.gitignore`**:
```gitignore
# Rust
/target
**/*.rs.bk
Cargo.lock

# IDE
.vscode/
.idea/
*.swp
*.swo
*~

# OS
.DS_Store
Thumbs.db

# Build artifacts
*.hex
*.elf
*.lst
*.map
```

**Create `CONTRIBUTING.md`**:
```markdown
# Contributing to Ossidata

We love your input! We want to make contributing as easy as possible.

## Development Setup

1. Install Rust nightly: `rustup toolchain install nightly-2025-04-27`
2. Install AVR GCC toolchain
3. Clone the repository
4. Run tests: `cargo test`

## Pull Request Process

1. Update documentation
2. Ensure CI passes
3. Update CHANGELOG.md
4. Follow our code style (cargo fmt)

## Code of Conduct

Be respectful and inclusive.
```

### Task 1.4: AVR Target Specification (Day 2)

**Create `avr-specs/avr-atmega328p.json`**:

```json
{
  "arch": "avr",
  "atomic-cas": false,
  "cpu": "atmega328p",
  "data-layout": "e-P1-p:16:8-i8:8-i16:8-i32:8-i64:8-f32:8-f64:8-n8-a:8",
  "eh-frame-header": false,
  "exe-suffix": ".elf",
  "executables": true,
  "late-link-args": {
    "gcc": [
      "-lgcc"
    ]
  },
  "linker": "avr-gcc",
  "linker-flavor": "gcc",
  "linker-is-gnu": true,
  "llvm-target": "avr-unknown-unknown",
  "max-atomic-width": 8,
  "no-default-libraries": false,
  "pre-link-args": {
    "gcc": [
      "-mmcu=atmega328p",
      "-Wl,--as-needed"
    ]
  },
  "target-c-int-width": "16",
  "target-pointer-width": "16"
}
```

---

## Phase 1: Core Types (Week 1-2)

### Task 2.1: ossidata-core Implementation

**`ossidata-core/Cargo.toml`**:
```toml
[package]
name = "ossidata-core"
version.workspace = true
edition.workspace = true
license.workspace = true

[dependencies]
# None - pure core types
```

**`ossidata-core/src/lib.rs`**:
```rust
#![no_std]
#![deny(unsafe_code)]
#![deny(missing_docs)]

//! Core types and traits for the Ossidata SDK

use core::marker::PhantomData;

/// Pin modes
pub mod pin_mode {
    /// Input mode (floating)
    pub struct Input;

    /// Input with pull-up resistor
    pub struct InputPullUp;

    /// Output mode
    pub struct Output;

    /// PWM output mode
    pub struct Pwm;
}

/// Logic levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogicLevel {
    /// Low (0V or GND)
    Low,
    /// High (5V or VCC on AVR)
    High,
}

/// Errors that can occur in GPIO operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    /// Invalid pin number
    InvalidPin,
    /// Bus error during communication
    BusError,
    /// Operation timed out
    Timeout,
}

/// Result type for ossidata operations
pub type Result<T> = core::result::Result<T, Error>;

/// Prelude for convenient imports
pub mod prelude {
    pub use super::pin_mode::*;
    pub use super::{Error, LogicLevel, Result};
}
```

### Task 2.2: Pin Trait Definitions

**Add to `ossidata-core/src/lib.rs`**:
```rust
/// Digital pin trait
pub trait DigitalPin {
    /// Pin number (0-based)
    const NUMBER: u8;

    /// Port register address (board-specific)
    const PORT: usize;

    /// Bit mask for this pin
    const MASK: u8;
}
```

---

## Phase 1: AVR HAL (Week 2-3)

### Task 3.1: GPIO Implementation for Uno

**`ossidata-uno/Cargo.toml`**:
```toml
[package]
name = "ossidata-uno"
version.workspace = true
edition.workspace = true

[dependencies]
ossidata-core = { path = "../ossidata-core" }
embedded-hal = { workspace = true }
avr-device = { workspace = true }

[dev-dependencies]
# For examples
panic-halt = "0.2"
```

**`ossidata-uno/.cargo/config.toml`**:
```toml
[build]
target = "../../avr-specs/avr-atmega328p.json"

[unstable]
build-std = ["core"]
```

**`ossidata-uno/src/lib.rs`** (skeleton):
```rust
#![no_std]
#![feature(abi_avr_interrupt)]

pub use avr_device::atmega328p;
pub use ossidata_core::prelude::*;

pub mod gpio;
pub mod delay;

use core::marker::PhantomData;

/// Peripherals singleton
pub struct Peripherals {
    /// GPIO pins
    pub pins: gpio::Pins,
    /// PAC peripherals (escape hatch)
    pub pac: atmega328p::Peripherals,
}

impl Peripherals {
    /// Take peripherals (can only be called once)
    pub fn take() -> Option<Self> {
        atmega328p::Peripherals::take().map(|pac| Self {
            pins: gpio::Pins::new(),
            pac,
        })
    }
}

/// Entry point attribute
pub use avr_device::entry;
```

### Task 3.2: Pin Mapping

**`ossidata-uno/src/gpio.rs`**:
```rust
use core::marker::PhantomData;
use ossidata_core::{pin_mode::*, LogicLevel};
use crate::atmega328p;

/// Pin with type-state mode
pub struct Pin<const N: u8, MODE> {
    _mode: PhantomData<MODE>,
}

// Pin definitions for Arduino Uno
// D0-D7 -> PORTD (PD0-PD7)
// D8-D13 -> PORTB (PB0-PB5)
// A0-A5 (D14-D19) -> PORTC (PC0-PC5)

/// All GPIO pins
pub struct Pins {
    pub d0: Pin<0, Input>,
    pub d1: Pin<1, Input>,
    // ... more pins
    pub d13: Pin<13, Input>,
}

impl Pins {
    pub(crate) fn new() -> Self {
        Self {
            d0: Pin { _mode: PhantomData },
            d1: Pin { _mode: PhantomData },
            // ...
            d13: Pin { _mode: PhantomData },
        }
    }
}

// Implementation coming in detailed phase
```

---

## Phase 1: First Example (Week 3)

### Task 4.1: Blink Example

**`ossidata-uno/examples/blink.rs`**:
```rust
#![no_std]
#![no_main]

use ossidata_uno::{entry, Peripherals};
use panic_halt as _;

#[entry]
fn main() -> ! {
    let dp = Peripherals::take().unwrap();

    // Type-state: must convert to output
    let mut led = dp.pins.d13.into_output();

    loop {
        led.set_high();
        ossidata_uno::delay::delay_ms(1000);
        led.set_low();
        ossidata_uno::delay::delay_ms(1000);
    }
}
```

### Task 4.2: Build and Flash Scripts

**Create `scripts/flash-uno.sh`**:
```bash
#!/bin/bash
set -e

EXAMPLE=${1:-blink}
PORT=${2:-/dev/ttyACM0}

echo "Building example: $EXAMPLE"
cd ossidata-uno
cargo build -Z build-std=core --target ../avr-specs/avr-atmega328p.json --release --example $EXAMPLE

echo "Converting to hex..."
avr-objcopy -O ihex \
    target/avr-atmega328p/release/examples/$EXAMPLE.elf \
    target/avr-atmega328p/release/examples/$EXAMPLE.hex

echo "Flashing to Arduino Uno at $PORT..."
avrdude -p atmega328p -c arduino -P $PORT -b 115200 \
    -U flash:w:target/avr-atmega328p/release/examples/$EXAMPLE.hex:i

echo "✅ Done!"
```

---

## Implementation Priority

### Week 1: Setup ✅
- [x] Research current tooling
- [ ] Initialize workspace
- [ ] Set up CI/CD
- [ ] Create documentation
- [ ] AVR target specs

### Week 2: Core Types
- [ ] `ossidata-core` crate
- [ ] Pin traits and modes
- [ ] Error types
- [ ] Documentation

### Week 3: GPIO
- [ ] Pin type-state implementation
- [ ] Port mapping for Uno
- [ ] `into_output()`, `into_input()` transitions
- [ ] `set_high()`, `set_low()`, `is_high()`, `is_low()`
- [ ] embedded-hal trait implementations

### Week 4: Delays & Blink
- [ ] Delay implementation (busy-wait initially)
- [ ] Blink example
- [ ] Build scripts
- [ ] Flash scripts
- [ ] **MILESTONE: LED blinks on real hardware!** 🎉

### Week 5-6: More GPIO Features
- [ ] All 20 pins defined
- [ ] Input with pull-up
- [ ] Button example
- [ ] Multi-blink example

---

## Success Criteria

### Phase 0 Complete When:
- ✅ Workspace builds successfully
- ✅ CI pipeline passes
- ✅ Documentation structure in place
- ✅ Team aligned on approach

### Phase 1 Complete When:
- ✅ `ossidata-core` has core types
- ✅ Pin type-state pattern implemented
- ✅ GPIO works for at least pin D13
- ✅ Blink example compiles
- ✅ **Blink example runs on real Uno** ← The big milestone!
- ✅ Code is well-documented
- ✅ CI validates everything

---

## Key Decisions Summary

| Decision | Choice | Rationale |
|----------|--------|-----------|
| **Build approach** | From scratch | Deep learning, full control |
| **Reference** | Study avr-hal | Learn from proven implementation |
| **Toolchain** | nightly-2025-04-27 | Validated on hardware 2025-10-10 |
| **HAL traits** | embedded-hal 1.0 | Stable, ecosystem compatible |
| **Workspace** | Monorepo | Easier management, shared deps |
| **First target** | Arduino Uno | Most popular, well-documented |
| **First feature** | GPIO (blink) | Simplest, validates toolchain |

---

## Risk Mitigation

### Risk: Nightly Rust breaks
**Mitigation**: Pin exact version in `rust-toolchain.toml`

### Risk: Flash size too large
**Mitigation**: Aggressive size optimization in profile.release

### Risk: Don't understand AVR registers
**Mitigation**: Reference avr-hal and Arduino C++ sources

### Risk: Type-state pattern too complex
**Mitigation**: Start simple, iterate based on usage

### Risk: CI fails on AVR cross-compile
**Mitigation**: Install AVR-GCC in CI, test locally first

---

## Next Immediate Actions

1. **Initialize workspace** (30 min)
   ```bash
   cargo new --lib ossidata-core
   cargo new --lib ossidata-hal
   cargo new --lib ossidata-uno
   cargo new --lib ossidata
   ```

2. **Create root Cargo.toml** (15 min)
   - Define workspace
   - Set shared dependencies
   - Configure profiles

3. **Create rust-toolchain.toml** (5 min)
   - Pin nightly-2025-04-27
   - Add components

4. **Set up CI** (1 hour)
   - GitHub Actions workflow
   - Format, clippy, build checks
   - AVR cross-compilation

5. **Implement ossidata-core** (2 hours)
   - Basic types
   - Pin traits
   - Error types

6. **Test build** (30 min)
   - `cargo build --workspace`
   - Fix any issues

**Estimated Time to First Blink**: 2-3 weeks of focused work

---

## Resources for Implementation

### Study Materials
- [avr-hal source](https://github.com/Rahix/avr-hal) - Reference implementation
- [Arduino wiring_digital.c](https://github.com/arduino/ArduinoCore-avr/blob/master/cores/arduino/wiring_digital.c)
- [ATmega328P datasheet](http://ww1.microchip.com/downloads/en/DeviceDoc/Atmel-7810-Automotive-Microcontrollers-ATmega328P_Datasheet.pdf)

### Quick References
- Our [RESEARCH.md](./RESEARCH.md) - Arduino implementation analysis
- Our [RUST_EMBEDDED_GUIDE.md](./RUST_EMBEDDED_GUIDE.md) - Rust patterns
- Our [API_DESIGN.md](./API_DESIGN.md) - Target API

---

## Last Updated
2025-10-10 - Updated with validated toolchain (nightly-2025-04-27), Phase 0.5 complete
