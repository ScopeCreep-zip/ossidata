---
tags:
  - deepwiki/ossidata
  - contributing
  - development
  - community
---

# Contributing to Ossidata

**Last Updated**: 2025-10-12

Thank you for your interest in contributing to Ossidata! We welcome contributions from everyone.

## 🤝 Code of Conduct

Be respectful, inclusive, and constructive. We're all here to learn and build something great together.

## 🚀 Getting Started

### Prerequisites

1. **Rust Nightly Toolchain** (for AVR support):
   ```bash
   rustup toolchain install nightly-2025-04-27
   rustup component add rust-src --toolchain nightly-2025-04-27
   ```
   > ⚠️ **Important**: Use exactly `nightly-2025-04-27` - this version is hardware-validated

2. **AVR-GCC Toolchain**:
   - **macOS**: `brew install avr-gcc avrdude`
   - **Ubuntu/Debian**: `sudo apt-get install gcc-avr avr-libc avrdude`
   - **Windows**: Download from [Microchip](https://www.microchip.com/en-us/tools-resources/develop/microchip-studio/gcc-compilers)

3. **Arduino Hardware** (for testing):
   - Arduino Uno recommended for initial development

### Development Setup

```bash
# Clone your fork
git clone https://github.com/YOUR_USERNAME/ossidata.git
cd ossidata

# Add upstream remote
git remote add upstream https://github.com/radicalkjax/ossidata.git

# Create a feature branch
git checkout -b feature/my-feature

# Verify everything builds
cargo build --workspace
cargo test --workspace
cargo clippy --workspace
cargo fmt --all -- --check
```

## 📝 Development Workflow

### 1. Find Something to Work On

- Check [GitHub Issues](https://github.com/radicalkjax/ossidata/issues) for open tasks
- Look for issues labeled `good-first-issue` or `help-wanted`
- Propose new features via GitHub Discussions first

### 2. Make Your Changes

Follow these guidelines:

#### Code Style
- **Format**: Run `cargo fmt --all` before committing
- **Linting**: Ensure `cargo clippy --workspace` passes with no warnings
- **Documentation**: Add rustdoc comments for all public APIs
- **Tests**: Write tests for new functionality

#### Commit Messages
```
<type>: <short summary>

<optional detailed description>

<optional footer>
```

**Types**:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code style/formatting
- `refactor`: Code restructuring
- `test`: Adding tests
- `chore`: Maintenance tasks

**Examples**:
```
feat: add PWM support for Arduino Uno

Implements SetDutyCycle trait from embedded-hal 1.0
for pins 3, 5, 6, 9, 10, 11 using Timer0, Timer1, and Timer2.

Closes #42
```

### 3. Testing

```bash
# Run tests
cargo test --workspace

# Test on target (requires Arduino Uno)
cd ossidata-uno
cargo build -Z build-std=core --target ../avr-specs/avr-atmega328p.json --release --example blink
./scripts/flash-uno.sh blink /dev/ttyACM0
```

### 4. Submit a Pull Request

1. Push your branch to your fork
2. Open a Pull Request against `main`
3. Fill out the PR template completely
4. Wait for CI checks to pass
5. Address review feedback
6. Once approved, a maintainer will merge

## 🏗️ Project Structure

```
ossidata/
├── boards/              # Board Support Packages
│   ├── arduino-uno/    # Arduino Uno BSP (✅ 45% complete)
│   ├── arduino-mega/   # Arduino Mega BSP (📋 planned)
│   └── arduino-due/    # Arduino Due BSP (📋 planned)
├── ossidata-core/       # Core types and traits
├── docs/                # User documentation
│   ├── GETTING_STARTED.md
│   ├── API_REFERENCE.md
│   ├── ARCHITECTURE.md
│   └── FLASHING_SOLUTION.md
├── agentdocs/           # Internal planning docs
├── flash.sh             # Cross-platform flash entry point
├── flash-impl.sh        # Flash implementation
├── flash-macos.sh       # macOS launcher
├── flash-linux.sh       # Linux launcher
└── flash-windows.bat    # Windows launcher
```

## 📋 Contribution Areas

### High Priority (Phase 1)
- ✅ GPIO implementation for Arduino Uno (COMPLETE)
- ✅ Serial (UART) communication (COMPLETE)
- 📋 PWM support (IN PROGRESS)
- 📋 ADC (analog input)
- 📋 Timer/interrupt abstractions

### Medium Priority
- 📋 I2C protocol support
- 📋 SPI protocol support
- 📋 Timer/interrupt abstractions
- 📋 Additional board support

### Always Welcome
- 🐛 Bug fixes
- 📖 Documentation improvements
- ✅ Test coverage
- 🎨 Example programs
- 🔧 Tooling improvements

## 🎯 Code Guidelines

### Embedded Rust Best Practices

1. **Use Type-State Pattern**:
   ```rust
   pub struct Pin<const N: u8, MODE> {
       _mode: PhantomData<MODE>,
   }

   impl<const N: u8> Pin<N, Input> {
       pub fn into_output(self) -> Pin<N, Output> { /* ... */ }
   }
   ```

2. **Implement embedded-hal Traits**:
   ```rust
   use embedded_hal::digital::OutputPin;

   impl<const N: u8> OutputPin for Pin<N, Output> {
       type Error = Infallible;
       fn set_high(&mut self) -> Result<(), Self::Error> { /* ... */ }
   }
   ```

3. **Use Volatile Access for Registers**:
   ```rust
   use core::ptr::{read_volatile, write_volatile};

   unsafe {
       write_volatile(PORTB as *mut u8, value);
   }
   ```

4. **Document Safety Requirements**:
   ```rust
   /// # Safety
   ///
   /// Must only be called once during initialization.
   /// Caller must ensure no other code accesses GPIO during execution.
   pub unsafe fn init_gpio() { /* ... */ }
   ```

### Documentation Standards

All public APIs must have:
- **Summary**: One-line description
- **Description**: Detailed explanation if needed
- **Examples**: Usage example in rustdoc
- **Errors**: Document error conditions
- **Safety**: Document unsafe requirements

Example:
```rust
/// Sets the pin to logic HIGH (5V on AVR)
///
/// # Example
///
/// ```no_run
/// # use ossidata_uno::prelude::*;
/// let mut led = pins.d13.into_output();
/// led.set_high();
/// ```
///
/// # Errors
///
/// Returns `Infallible` - this operation cannot fail.
pub fn set_high(&mut self) -> Result<(), Infallible> {
    // Implementation
}
```

## 🔍 Review Process

### What We Look For

✅ **Code Quality**:
- Follows Rust idioms and best practices
- Properly documented
- Well-tested
- Clippy warnings addressed

✅ **Design**:
- Fits project architecture
- Uses appropriate abstractions
- Type-safe where possible
- Zero-cost abstractions

✅ **Compatibility**:
- Works with embedded-hal 1.0
- No breaking changes (unless major version)
- Cross-platform where applicable

### CI Checks

All PRs must pass:
- `cargo fmt --all -- --check`
- `cargo clippy --workspace -- -D warnings`
- `cargo test --workspace`
- `cargo build` for all targets
- Documentation builds (`cargo doc`)

## 🐛 Reporting Bugs

### Before Submitting

1. Check if the bug is already reported
2. Verify it's not a hardware issue
3. Test with the latest `main` branch

### Bug Report Template

```markdown
**Description**: Brief description of the bug

**Steps to Reproduce**:
1. Step one
2. Step two
3. ...

**Expected Behavior**: What should happen

**Actual Behavior**: What actually happens

**Environment**:
- Board: Arduino Uno
- Rust Version: nightly-2024-05-01
- OS: macOS 14.5
- Ossidata Version: 0.1.0

**Additional Context**: Any other relevant information
```

## 💡 Feature Requests

We welcome feature requests! Please:

1. **Check existing issues/discussions** first
2. **Start a discussion** in GitHub Discussions
3. **Describe the use case** clearly
4. **Consider the scope** - does it fit the project goals?

## ❓ Questions?

- **Documentation**: Check [docs/](docs/)
- **Issues**: Ask in GitHub Issues
- **Discussions**: Use GitHub Discussions
- **Chat**: Join our Matrix chat *(coming soon)*

## 📚 Resources

### Essential Reading
- [Implementation Plan](docs/IMPLEMENTATION_PLAN.md)
- [Architecture Design](agentdocs/ARCHITECTURE.md)
- [Rust Embedded Book](https://docs.rust-embedded.org/book/)
- [embedded-hal docs](https://docs.rs/embedded-hal/)

### Arduino References
- [Arduino Language Reference](https://www.arduino.cc/reference/en/)
- [ArduinoCore-avr Source](https://github.com/arduino/ArduinoCore-avr)
- [ATmega328P Datasheet](http://ww1.microchip.com/downloads/en/DeviceDoc/Atmel-7810-Automotive-Microcontrollers-ATmega328P_Datasheet.pdf)

### Rust References
- [avr-hal Source](https://github.com/Rahix/avr-hal) - Great reference!
- [The Rustonomicon](https://doc.rust-lang.org/nomicon/)
- [Rust Core Library](https://doc.rust-lang.org/core/)

## 🏆 Recognition

Contributors will be:
- Listed in [AUTHORS.md](AUTHORS.md)
- Credited in release notes
- Mentioned in project announcements

## 📄 License

By contributing, you agree that your contributions will be licensed under the same terms as the project (MIT OR Apache-2.0).

---

Thank you for contributing to Ossidata! 🦀🚀
