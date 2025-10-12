fn main() {
    // Set the MCU for AVR compilation
    println!("cargo:rustc-env=AVR_CPU=atmega328p");
}