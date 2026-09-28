//! PNG helpers for screenshots and reference-image comparison.

use matcha_core::{GameBoy, HEIGHT, WIDTH};
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

/// Writes the current frame, scaled by an integer factor, as a PNG file.
pub fn write_screenshot(gb: &GameBoy, palette: &[u32; 4], scale: u32, path: &Path) -> Result<(), String> {
    let scale = scale.clamp(1, 8) as usize;
    let mut rgba = vec![0u8; WIDTH * HEIGHT * 4];
    gb.render_rgba(palette, &mut rgba);
    let (w, h) = (WIDTH * scale, HEIGHT * scale);
    let mut scaled = vec![0u8; w * h * 4];
    for y in 0..h {
        for x in 0..w {
            let src = ((y / scale) * WIDTH + x / scale) * 4;
            let dst = (y * w + x) * 4;
            scaled[dst..dst + 4].copy_from_slice(&rgba[src..src + 4]);
        }
    }
    let file = File::create(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let mut enc = png::Encoder::new(BufWriter::new(file), w as u32, h as u32);
    enc.set_color(png::ColorType::Rgba);
    enc.set_depth(png::BitDepth::Eight);
    let mut writer = enc.write_header().map_err(|e| e.to_string())?;
    writer.write_image_data(&scaled).map_err(|e| e.to_string())
}

/// Decodes a PNG into (width, height, RGB triples), whatever its colour type.
pub fn read_png_rgb(path: &Path) -> Result<(u32, u32, Vec<[u8; 3]>), String> {
    let file = File::open(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let mut decoder = png::Decoder::new(file);
    decoder.set_transformations(png::Transformations::EXPAND | png::Transformations::STRIP_16);
    let mut reader = decoder.read_info().map_err(|e| e.to_string())?;
    let mut buf = vec![0; reader.output_buffer_size()];
    let info = reader.next_frame(&mut buf).map_err(|e| e.to_string())?;
    let data = &buf[..info.buffer_size()];
    let px: Vec<[u8; 3]> = match info.color_type {
        png::ColorType::Rgb => data.chunks_exact(3).map(|c| [c[0], c[1], c[2]]).collect(),
        png::ColorType::Rgba => data.chunks_exact(4).map(|c| [c[0], c[1], c[2]]).collect(),
        png::ColorType::Grayscale => data.iter().map(|&g| [g, g, g]).collect(),
        png::ColorType::GrayscaleAlpha => data.chunks_exact(2).map(|c| [c[0], c[0], c[0]]).collect(),
        png::ColorType::Indexed => return Err("indexed PNG was not expanded".into()),
    };
    Ok((info.width, info.height, px))
}

/// Number of pixels that differ between the frame (grey palette) and `expected`.
pub fn diff_against(gb: &GameBoy, expected: &Path) -> Result<usize, String> {
    let (w, h, px) = read_png_rgb(expected)?;
    if (w as usize, h as usize) != (WIDTH, HEIGHT) {
        return Err(format!("reference image is {w}x{h}, expected {WIDTH}x{HEIGHT}"));
    }
    let mut rgba = vec![0u8; WIDTH * HEIGHT * 4];
    gb.render_rgba(&matcha_core::palettes::GREY, &mut rgba);
    Ok(rgba.chunks_exact(4).zip(px.iter()).filter(|(a, b)| a[..3] != b[..]).count())
}
