# Milestone v0.1.0 - "Hello World"

**Goal**: Enable developers to create and run a working "Hello World" program on any supported Arduino device.

**Status**: 🚧 In Progress (Phase 1 Starting)
**Target Date**: TBD
**Last Updated**: 2025-10-10

---

## 🎯 Success Criteria

The project is considered **complete for v0.1.0** when:

### ✅ Core Requirement: Hello World Works

A developer can write this program and it works on **all supported boards**:

```rust
#![no_std]
#![no_main]

use ossidata::prelude::*;

#[ossidata::entry]
fn main() -> ! {
    let dp = Peripherals::take().unwrap();
    let mut serial = dp.serial.begin(9600);

    serial.println("Hello, World!");

    loop {
        // Keep running
    }
}
```

**Output on serial monitor**: `Hello, World!`

### ✅ Supported Boards (Minimum for v0.1.0)

At least **3 boards** from different architectures:

1. **Arduino Uno** (AVR - ATmega328P) - Primary target ✅
2. **Arduino Mega 2560** (AVR - ATmega2560) - Secondary AVR
3. **Arduino Due** (ARM - SAM3X8E) - ARM validation

**Stretch**: Arduino Zero (ARM SAMD21), Arduino Nano (AVR)

### ✅ Features Required

#### 1. Serial/UART Communication
- `Serial.begin(baud_rate)` - Initialize serial
- `Serial.print(str)` - Print string without newline
- `Serial.println(str)` - Print string with newline
- `Serial.write(byte)` - Write single byte
- Works over USB serial on boards that support it

#### 2. Basic GPIO (for LED blink validation)
- `pin.into_output()` - Configure as output
- `pin.set_high()` / `pin.set_low()` - Set pin state
- Works on all digital pins

#### 3. Delay Functions
- `delay_ms(milliseconds)` - Millisecond delay
- `delay_us(microseconds)` - Microsecond delay

#### 4. Build & Flash Tooling
- Single command to build: `cargo build --release`
- Single command to flash: `cargo run --release`
- Works on macOS, Linux, Windows

---

## 📋 Deliverables

### 1. Working Examples

**Required Examples** (must work on all supported boards):

- ✅ `examples/hello_world.rs` - Print "Hello, World!" to serial
- ✅ `examples/blink.rs` - Blink onboard LED
- ✅ `examples/echo.rs` - Echo serial input back
- ✅ `examples/counter.rs` - Print incrementing counter

### 2. Documentation

**User Documentation**:
- ✅ `docs/GETTING_STARTED.md` - Complete setup guide
- ✅ `docs/BOARD_SETUP.md` - Per-board setup instructions
- ✅ `docs/API_REFERENCE.md` - Basic API documentation
- ✅ `README.md` - Updated with working examples

**Developer Documentation**:
- ✅ `CONTRIBUTING.md` - How to add new boards
- ✅ API docs on docs.rs (rustdoc)

### 3. Tooling

**Build System**:
- ✅ Workspace builds without errors
- ✅ CI/CD validates all boards
- ✅ Examples compile for all targets

**Flashing Tools**:
- ✅ `scripts/flash-uno.sh` - Flash Arduino Uno
- ✅ `scripts/flash-mega.sh` - Flash Arduino Mega
- ✅ `scripts/flash-due.sh` - Flash Arduino Due
- ✅ Cross-platform support (bash + PowerShell)

### 4. Testing

**Validation**:
- ✅ All examples tested on real hardware
- ✅ Serial output verified
- ✅ LED blink verified
- ✅ Works on macOS, Linux, Windows

---

## 🏗️ Implementation Plan

### Phase 1: Arduino Uno (Weeks 1-4)

**Week 1**: Foundation
- [x] Project planning complete
- [x] Hardware validation complete (avr-hal blink works!)
- [ ] Workspace setup
- [ ] CI/CD pipeline
- [ ] Core types (`ossidata-core`)

**Week 2**: GPIO
- [ ] Pin type-state pattern
- [ ] Digital I/O implementation
- [ ] Blink example working
- [ ] ✅ **Checkpoint: LED blinks on Uno**

**Week 3**: Serial Communication
- [ ] UART abstraction
- [ ] `Serial.begin()`, `Serial.print()`
- [ ] Formatted output (ufmt integration)
- [ ] ✅ **Checkpoint: "Hello, World!" prints**

**Week 4**: Polish & Validation
- [ ] Delay functions
- [ ] All 4 examples working
- [ ] Documentation complete
- [ ] ✅ **Checkpoint: Uno fully functional**

### Phase 2: Arduino Mega (Week 5)

- [ ] Create `ossidata-mega` BSP
- [ ] Port all Uno code
- [ ] Handle 4x serial ports
- [ ] Test all examples
- [ ] ✅ **Checkpoint: Mega fully functional**

### Phase 3: Arduino Due (Weeks 6-7)

- [ ] Create `ossidata-due` BSP
- [ ] ARM HAL implementation
- [ ] Handle 3.3V vs 5V differences
- [ ] USB serial support
- [ ] Test all examples
- [ ] ✅ **Checkpoint: Due fully functional**

### Phase 4: Final Release (Week 8)

- [ ] Cross-platform testing
- [ ] Documentation review
- [ ] Performance benchmarks
- [ ] Release v0.1.0 to crates.io
- [ ] 🎉 **MILESTONE COMPLETE**

---

## ✅ Acceptance Checklist

### Technical Requirements

- [ ] **Builds successfully** on all platforms (macOS, Linux, Windows)
- [ ] **Compiles** for all supported boards without warnings
- [ ] **Flashes** to all supported boards with provided scripts
- [ ] **Serial output** works on all boards
- [ ] **Examples run** without errors on real hardware
- [ ] **CI/CD passes** all checks
- [ ] **Documentation** is complete and accurate
- [ ] **Zero compiler warnings** (`cargo clippy` clean)
- [ ] **Formatted** (`cargo fmt` applied)
- [ ] **No unsafe code** except in PAC interaction (clearly documented)

### User Experience Requirements

- [ ] **Installation takes < 30 minutes** (including toolchain setup)
- [ ] **First example runs in < 5 minutes** after setup
- [ ] **Error messages are helpful** and guide users
- [ ] **Documentation has no broken links**
- [ ] **Examples are self-explanatory**
- [ ] **Works out-of-box** on Arduino Uno (most common)

### Quality Requirements

- [ ] **Binary size < 2KB** for Hello World (AVR)
- [ ] **RAM usage < 100 bytes** for Hello World (AVR)
- [ ] **Compile time < 30 seconds** for examples (incremental)
- [ ] **No heap allocation** (unless explicitly enabled)
- [ ] **Panic handler** is minimal and informative

---

## 🎯 Hello World Variants

Different "Hello World" implementations to validate functionality:

### 1. Serial Hello World (Primary)
```rust
#[ossidata::entry]
fn main() -> ! {
    let dp = Peripherals::take().unwrap();
    let mut serial = dp.serial.begin(9600);

    serial.println("Hello, World!");

    loop {}
}
```

### 2. Blink Hello World (Hardware validation)
```rust
#[ossidata::entry]
fn main() -> ! {
    let dp = Peripherals::take().unwrap();
    let mut led = dp.pins.d13.into_output();

    loop {
        led.set_high();
        delay_ms(500);
        led.set_low();
        delay_ms(500);
    }
}
```

### 3. Interactive Hello World (Full validation)
```rust
#[ossidata::entry]
fn main() -> ! {
    let dp = Peripherals::take().unwrap();
    let mut serial = dp.serial.begin(9600);
    let mut led = dp.pins.d13.into_output();

    serial.println("Hello! Press any key to blink LED.");

    loop {
        if serial.available() > 0 {
            let _ = serial.read();
            led.toggle();
            serial.println("LED toggled!");
        }
    }
}
```

### 4. Formatted Hello World (ufmt validation)
```rust
use ufmt::uwriteln;

#[ossidata::entry]
fn main() -> ! {
    let dp = Peripherals::take().unwrap();
    let mut serial = dp.serial.begin(9600);

    let name = "Arduino";
    let version = 1;

    uwriteln!(serial, "Hello from {}! Version: {}", name, version).ok();

    loop {}
}
```

---

## 📊 Progress Tracking

### Overall Progress: 15%

| Component | Status | Progress |
|-----------|--------|----------|
| **Planning** | ✅ Complete | 100% |
| **Hardware Validation** | ✅ Complete | 100% |
| **Workspace Setup** | 📋 Not Started | 0% |
| **ossidata-core** | 📋 Not Started | 0% |
| **GPIO (Uno)** | 📋 Not Started | 0% |
| **Serial (Uno)** | 📋 Not Started | 0% |
| **Uno Examples** | 📋 Not Started | 0% |
| **Arduino Mega** | 📋 Not Started | 0% |
| **Arduino Due** | 📋 Not Started | 0% |
| **Documentation** | 🚧 In Progress | 20% |
| **Testing** | 📋 Not Started | 0% |

### Board Support Status

| Board | GPIO | Serial | Examples | Flash Scripts | Status |
|-------|------|--------|----------|---------------|--------|
| Arduino Uno | ❌ | ❌ | ❌ | ❌ | 📋 Not Started |
| Arduino Mega | ❌ | ❌ | ❌ | ❌ | 📋 Not Started |
| Arduino Due | ❌ | ❌ | ❌ | ❌ | 📋 Not Started |

---

## 🚨 Risks & Mitigation

### High Risk

**Risk**: Serial UART implementation too complex
**Impact**: Can't print "Hello, World!"
**Mitigation**: Start with polling (not interrupts), reference avr-hal implementation, simplify API initially

**Risk**: Flash size exceeds 2KB on AVR
**Impact**: Won't fit on Uno
**Mitigation**: Aggressive size optimization, minimal panic handler, no formatting in critical path

### Medium Risk

**Risk**: Cross-platform build scripts don't work
**Impact**: Hard to use on Windows/Linux
**Mitigation**: Provide both bash and PowerShell, document manual process, consider ravedude

**Risk**: ARM board support takes too long
**Impact**: Delays milestone
**Mitigation**: Focus on Uno+Mega first, Due is stretch goal

### Low Risk

**Risk**: ufmt formatting doesn't work
**Impact**: No formatted output
**Mitigation**: Fall back to basic print, add formatting in v0.2

---

## 🎉 Success Metrics

### Quantitative

- ✅ **3+ boards supported** (Uno, Mega, Due minimum)
- ✅ **4+ examples working** on all boards
- ✅ **< 5 minute** setup time for first example
- ✅ **< 2KB** binary size for Hello World (AVR)
- ✅ **100%** example success rate on hardware
- ✅ **Zero** compiler warnings

### Qualitative

- ✅ **Feels easier than Arduino C++** to experienced Rust devs
- ✅ **Feels safer than Arduino C++** (type safety visible)
- ✅ **Documentation is clear** (no questions in first user test)
- ✅ **Error messages are helpful** (guide to solution)
- ✅ **Community excited** (positive GitHub discussions)

---

## 📅 Timeline

**Optimistic**: 6 weeks
**Realistic**: 8 weeks
**Pessimistic**: 10 weeks

**Target Completion**: End of Q1 2025

---

## 🎯 Post-v0.1.0 Roadmap

After achieving "Hello World" milestone, next goals:

### v0.2.0 - "Blink & Read"
- PWM output (LED fading)
- ADC input (read sensors)
- Button input with debouncing

### v0.3.0 - "Communication"
- I2C support
- SPI support
- Multi-board examples

### v1.0.0 - "Production Ready"
- Full Arduino API coverage
- 10+ board support
- Comprehensive documentation
- Stable API guarantee

---

## 📝 Notes

### Why "Hello World" as Milestone?

1. **Universal Test**: Every developer understands "Hello World"
2. **Validates Toolchain**: Build, flash, serial all working
3. **Proves Concept**: Shows Rust Arduino is viable
4. **Foundation**: Serial is required for debugging all other features
5. **Demonstrable**: Easy to show working demo

### What "Hello World" Validates

- ✅ Rust nightly AVR compilation works
- ✅ Cross-compilation to ARM works
- ✅ Linker scripts are correct
- ✅ UART hardware abstraction works
- ✅ Formatted output works
- ✅ Flash/upload tooling works
- ✅ Serial monitor connection works
- ✅ Multi-board support is feasible

---

## Last Updated

2025-10-10 - Hardware validation complete, Phase 1 starting
