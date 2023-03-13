fn main() {
    let mut c8 = chip8_rs::Chip8::new();
    c8.load("space.ch8").unwrap();
    loop {
        c8.cycle();
    }
}
