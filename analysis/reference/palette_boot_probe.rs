//! Controlled corpus probe, not an alternate emulator initialization.
//!
//! Build after `cargo build --release -p matcha-core`:
//! rustc --edition=2024 -C panic=abort -C lto=fat -C opt-level=3 analysis/reference/palette_boot_probe.rs \
//!   --extern matcha_core=target/release/libmatcha_core.rlib \
//!   -L dependency=target/release/deps -o target/reference/palette_boot_probe
//! Pass Bomber.gbc and headache-boy.gb paths. Both runs use the first 600
//! frames of the corpus monkey schedule; only initial OBJ palette RAM differs.

use matcha_core::{Buttons, GameBoy, Model};

fn main() {
    for path in std::env::args().skip(1) {
        for initialized in [false, true] {
            let rom = std::fs::read(&path).expect("read ROM");
            let mut gb = GameBoy::new_with_model(rom, Model::Cgb).expect("load ROM");
            if initialized {
                let index = gb.peek(0xFF6A);
                gb.poke(0xFF6A, 0x80);
                for _ in 0..64 {
                    gb.poke(0xFF6B, 0);
                }
                gb.poke(0xFF6A, index);
            }
            let mut rgba = vec![0; 160 * 144 * 4];
            let mut non_uniform = 0;
            for frame in 0..600 {
                gb.set_buttons(match frame % 120 {
                    60..=65 => Buttons::START,
                    90..=95 => Buttons::A,
                    _ => Buttons::NONE,
                });
                gb.run_frame();
                gb.render_rgba(&[0; 4], &mut rgba);
                non_uniform += usize::from(rgba.chunks_exact(4).any(|pixel| pixel != &rgba[..4]));
            }
            let mut indices = [0; 4];
            for &pixel in gb.framebuffer() {
                indices[pixel as usize] += 1;
            }
            println!("{path}: zero_obj={initialized}, non_uniform={non_uniform}/600, indices={indices:?}");
        }
    }
}
