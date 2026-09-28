//! The PPU: LCD timing, STAT/LYC interrupts and scanline rendering.
//!
//! Timing is tracked per dot (4 per M-cycle). Each visible line is rendered in
//! one pass at the start of mode 3 with the register values current at that
//! moment, and mode 3 is stretched by the documented SCX/window/object
//! penalties so STAT timing matches hardware (ADR-0002). Mid-scanline raster
//! effects need a pixel FIFO, which is on the roadmap.

use crate::state::{StateError, StateReader, StateWriter};

pub const WIDTH: usize = 160;
pub const HEIGHT: usize = 144;
const DOTS_PER_LINE: u16 = 456;
const LINES_PER_FRAME: u8 = 154;
const OAM_SCAN_DOTS: u16 = 80;
/// Dots per frame; also the cadence of blank frames while the LCD is off.
pub const DOTS_PER_FRAME: u32 = 70_224;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum Mode {
    #[default]
    HBlank = 0,
    VBlank = 1,
    OamScan = 2,
    Drawing = 3,
}

/// One entry of the per-line object buffer (at most 10).
#[derive(Clone, Copy, Debug, Default)]
struct LineObject {
    x: u8,
    y: u8,
    tile: u8,
    attr: u8,
    index: u8,
}

/// Interrupt requests produced during a tick.
#[derive(Clone, Copy, Debug, Default)]
pub struct PpuEvents {
    pub vblank: bool,
    pub stat: bool,
}

pub struct Ppu {
    pub(crate) vram: [u8; 0x2000],
    pub(crate) oam: [u8; 0xA0],
    lcdc: u8,
    /// STAT interrupt-select bits 3..6 (the rest is computed).
    stat_select: u8,
    scy: u8,
    scx: u8,
    lyc: u8,
    bgp: u8,
    obp0: u8,
    obp1: u8,
    wy: u8,
    wx: u8,

    mode: Mode,
    /// Internal line counter (0..=153).
    line: u8,
    /// Value exposed through the LY register (differs on line 153).
    ly: u8,
    dot: u16,
    /// Dot at which mode 3 ends on the current line.
    mode3_end: u16,
    /// Internal window line counter.
    window_line: u8,
    /// WY matched LY at some point this frame.
    window_y_hit: bool,
    /// Previous state of the STAT interrupt line (interrupts fire on rising edges).
    stat_line: bool,
    /// The line being drawn is the first after the LCD was switched on.
    first_line_after_on: bool,
    /// First frame after LCD-on is not shown (hardware outputs blank).
    skip_frame: bool,
    /// Dots counted while the LCD is off, to keep emitting frames.
    off_dots: u32,

    framebuffer: [u8; WIDTH * HEIGHT],
    frame_ready: bool,
    frame_count: u64,
}

impl core::fmt::Debug for Ppu {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Ppu")
            .field("mode", &self.mode)
            .field("line", &self.line)
            .field("dot", &self.dot)
            .field("lcdc", &self.lcdc)
            .finish_non_exhaustive()
    }
}

impl Ppu {
    /// State after the DMG boot ROM hands over (LCD on, start of VBlank's end).
    pub fn new() -> Self {
        let mut p = Self::power_on();
        p.lcdc = 0x91;
        p.bgp = 0xFC;
        // The boot ROM leaves the PPU at line 0 of a fresh frame.
        p.mode = Mode::OamScan;
        p
    }

    pub fn power_on() -> Self {
        Self {
            vram: [0; 0x2000],
            oam: [0; 0xA0],
            lcdc: 0,
            stat_select: 0,
            scy: 0,
            scx: 0,
            lyc: 0,
            bgp: 0,
            obp0: 0xFF,
            obp1: 0xFF,
            wy: 0,
            wx: 0,
            mode: Mode::HBlank,
            line: 0,
            ly: 0,
            dot: 0,
            mode3_end: OAM_SCAN_DOTS + 172,
            window_line: 0,
            window_y_hit: false,
            stat_line: false,
            first_line_after_on: false,
            skip_frame: false,
            off_dots: 0,
            framebuffer: [0; WIDTH * HEIGHT],
            frame_ready: false,
            frame_count: 0,
        }
    }

    #[inline]
    fn lcd_on(&self) -> bool {
        self.lcdc & 0x80 != 0
    }

    pub fn mode(&self) -> Mode {
        if self.lcd_on() { self.mode } else { Mode::HBlank }
    }

    /// Shade indices (0 = lightest, 3 = darkest), row-major 160x144.
    pub fn framebuffer(&self) -> &[u8; WIDTH * HEIGHT] {
        &self.framebuffer
    }

    pub fn take_frame_ready(&mut self) -> bool {
        core::mem::take(&mut self.frame_ready)
    }

    pub fn frame_count(&self) -> u64 {
        self.frame_count
    }

    // --- CPU access ------------------------------------------------------------

    pub fn vram_accessible(&self) -> bool {
        !self.lcd_on() || self.mode != Mode::Drawing
    }

    pub fn oam_accessible(&self) -> bool {
        !self.lcd_on() || matches!(self.mode, Mode::HBlank | Mode::VBlank)
    }

    #[inline]
    fn lyc_match(&self) -> bool {
        self.ly == self.lyc
    }

    pub fn read_register(&self, addr: u16) -> u8 {
        match addr {
            0xFF40 => self.lcdc,
            0xFF41 => {
                let mode = self.mode() as u8;
                let coincidence = if self.lcd_on() && self.lyc_match() { 0x04 } else { 0 };
                0x80 | self.stat_select | coincidence | mode
            }
            0xFF42 => self.scy,
            0xFF43 => self.scx,
            0xFF44 => {
                if self.lcd_on() {
                    self.ly
                } else {
                    0
                }
            }
            0xFF45 => self.lyc,
            0xFF47 => self.bgp,
            0xFF48 => self.obp0,
            0xFF49 => self.obp1,
            0xFF4A => self.wy,
            0xFF4B => self.wx,
            _ => 0xFF,
        }
    }

    /// Returns true if the write raised a STAT interrupt.
    pub fn write_register(&mut self, addr: u16, value: u8) -> bool {
        match addr {
            0xFF40 => {
                let was_on = self.lcd_on();
                self.lcdc = value;
                match (was_on, self.lcd_on()) {
                    (true, false) => self.turn_off(),
                    (false, true) => self.turn_on(),
                    _ => {}
                }
                return self.update_stat_line();
            }
            0xFF41 => {
                // DMG quirk: the write behaves as if 0xFF was written for one
                // cycle first, which can fire a spurious STAT interrupt.
                let mut fired = false;
                if self.lcd_on() {
                    self.stat_select = 0x78;
                    fired = self.update_stat_line();
                }
                self.stat_select = value & 0x78;
                fired |= self.update_stat_line();
                return fired;
            }
            0xFF42 => self.scy = value,
            0xFF43 => self.scx = value,
            0xFF44 => {} // read-only
            0xFF45 => {
                self.lyc = value;
                return self.update_stat_line();
            }
            0xFF47 => self.bgp = value,
            0xFF48 => self.obp0 = value,
            0xFF49 => self.obp1 = value,
            0xFF4A => self.wy = value,
            0xFF4B => self.wx = value,
            _ => {}
        }
        false
    }

    fn turn_off(&mut self) {
        self.line = 0;
        self.ly = 0;
        self.dot = 0;
        self.mode = Mode::HBlank;
        self.stat_line = false;
        self.off_dots = 0;
        self.framebuffer.fill(0);
    }

    fn turn_on(&mut self) {
        self.line = 0;
        self.ly = 0;
        self.dot = 0;
        self.mode = Mode::HBlank;
        self.window_line = 0;
        self.window_y_hit = false;
        self.first_line_after_on = true;
        self.skip_frame = true;
    }

    /// Recomputes the STAT interrupt line; returns true on a rising edge.
    fn update_stat_line(&mut self) -> bool {
        if !self.lcd_on() {
            self.stat_line = false;
            return false;
        }
        let s = self.stat_select;
        let line = (s & 0x40 != 0 && self.lyc_match())
            || match self.mode {
                Mode::HBlank => s & 0x08 != 0,
                Mode::VBlank => {
                    // The mode-2 source also fires as VBlank begins.
                    s & 0x10 != 0 || (s & 0x20 != 0 && self.line == 144 && self.dot < 4)
                }
                Mode::OamScan => s & 0x20 != 0,
                Mode::Drawing => false,
            };
        let rising = line && !self.stat_line;
        self.stat_line = line;
        rising
    }

    // --- timing ------------------------------------------------------------------

    /// Advances one M-cycle (4 dots).
    #[inline]
    pub fn tick(&mut self) -> PpuEvents {
        let mut ev = PpuEvents::default();
        if !self.lcd_on() {
            self.off_dots += 4;
            if self.off_dots >= DOTS_PER_FRAME {
                self.off_dots -= DOTS_PER_FRAME;
                self.frame_ready = true;
                self.frame_count += 1;
            }
            return ev;
        }
        for _ in 0..4 {
            self.dot += 1;
            if self.dot == DOTS_PER_LINE {
                self.dot = 0;
                self.next_line(&mut ev);
            } else {
                self.step_within_line();
            }
        }
        ev.stat |= self.update_stat_line();
        ev
    }

    fn next_line(&mut self, ev: &mut PpuEvents) {
        self.line += 1;
        if self.line == LINES_PER_FRAME {
            self.line = 0;
            self.window_line = 0;
            self.window_y_hit = false;
        }
        self.ly = self.line;
        self.first_line_after_on = false;
        match self.line {
            0..=143 => self.mode = Mode::OamScan,
            144 => {
                self.mode = Mode::VBlank;
                ev.vblank = true;
                self.frame_ready = true;
                self.frame_count += 1;
                if self.skip_frame {
                    self.skip_frame = false;
                    self.framebuffer.fill(0);
                }
            }
            _ => {}
        }
    }

    fn step_within_line(&mut self) {
        match self.line {
            0..=143 => {
                if self.dot == OAM_SCAN_DOTS {
                    self.start_drawing();
                } else if self.dot == self.mode3_end && self.mode == Mode::Drawing {
                    self.mode = Mode::HBlank;
                }
            }
            153 if self.dot == 4 => self.ly = 0,
            _ => {}
        }
    }

    fn start_drawing(&mut self) {
        if !self.window_y_hit && self.wy == self.line {
            self.window_y_hit = true;
        }
        let mut objects = [LineObject::default(); 10];
        let count = self.scan_oam(&mut objects);
        let objects = &mut objects[..count];
        let window = self.window_visible_on_line();
        self.mode3_end = OAM_SCAN_DOTS + self.mode3_length(objects, window);
        self.mode = Mode::Drawing;
        self.render_line(objects, window);
        if window {
            self.window_line = self.window_line.wrapping_add(1);
        }
    }

    #[inline]
    fn window_visible_on_line(&self) -> bool {
        self.lcdc & 0x20 != 0 && self.window_y_hit && self.wx <= 166
    }

    #[inline]
    fn obj_height(&self) -> u8 {
        if self.lcdc & 0x04 != 0 { 16 } else { 8 }
    }

    /// Selects up to 10 objects overlapping this line, in OAM order.
    fn scan_oam(&self, out: &mut [LineObject; 10]) -> usize {
        let height = self.obj_height();
        let ly = self.line.wrapping_add(16);
        let mut n = 0;
        for (index, e) in self.oam.chunks_exact(4).enumerate() {
            let y = e[0];
            if ly >= y && ly < y.wrapping_add(height) && y < 160 + 16 {
                out[n] = LineObject { y, x: e[1], tile: e[2], attr: e[3], index: index as u8 };
                n += 1;
                if n == 10 {
                    break;
                }
            }
        }
        n
    }

    /// Mode 3 duration in dots: 172 + SCX%8 + window + object penalties
    /// (Pan Docs, "Mode 3 length").
    fn mode3_length(&self, objects: &[LineObject], window: bool) -> u16 {
        let mut len = 172 + u16::from(self.scx & 7);
        let wx_start = i32::from(self.wx) - 7;
        if window {
            len += 6;
        }
        if self.lcdc & 0x02 == 0 || objects.is_empty() {
            return len;
        }
        // Objects are considered left to right, ties broken by OAM index.
        let mut order: [LineObject; 10] = [LineObject::default(); 10];
        order[..objects.len()].copy_from_slice(objects);
        let order = &mut order[..objects.len()];
        order.sort_unstable_by_key(|o| (o.x, o.index));
        // (is_window, tile column) of the last tile that already paid its penalty.
        let mut last_tile: Option<(bool, i32)> = None;
        for o in order.iter() {
            if o.x >= 168 {
                continue; // fully off-screen right: never fetched
            }
            if o.x == 0 {
                len += 11;
                continue;
            }
            let pixel = i32::from(o.x) - 8;
            let (tile, offset) = if window && pixel >= wx_start {
                let p = pixel - wx_start;
                ((true, p.div_euclid(8)), p.rem_euclid(8))
            } else {
                let p = pixel + i32::from(self.scx & 7);
                ((false, p.div_euclid(8)), p.rem_euclid(8))
            };
            if last_tile != Some(tile) {
                last_tile = Some(tile);
                len += (7 - offset - 2).max(0) as u16;
            }
            len += 6;
        }
        len.min(DOTS_PER_LINE - OAM_SCAN_DOTS - 4)
    }

    #[inline]
    fn tile_row(&self, tile_addr: usize, row: usize) -> (u8, u8) {
        let base = tile_addr + row * 2;
        (self.vram[base], self.vram[base + 1])
    }

    /// VRAM offset of a BG/window tile's pixel data.
    #[inline]
    fn bg_tile_addr(&self, tile: u8) -> usize {
        if self.lcdc & 0x10 != 0 {
            usize::from(tile) * 16
        } else {
            (0x1000 + i32::from(tile as i8) * 16) as usize
        }
    }

    fn render_line(&mut self, objects: &[LineObject], window: bool) {
        let y = usize::from(self.line);
        let mut bg_index = [0u8; WIDTH];
        let row_start = y * WIDTH;

        if self.lcdc & 0x01 != 0 {
            let wx_start = i32::from(self.wx) - 7;
            let bg_map = if self.lcdc & 0x08 != 0 { 0x1C00 } else { 0x1800 };
            let win_map = if self.lcdc & 0x40 != 0 { 0x1C00 } else { 0x1800 };
            let bg_y = usize::from(self.line.wrapping_add(self.scy));
            for (x, slot) in bg_index.iter_mut().enumerate() {
                let (map, map_x, map_y) = if window && x as i32 >= wx_start {
                    (win_map, (x as i32 - wx_start) as usize, usize::from(self.window_line))
                } else {
                    (bg_map, (x + usize::from(self.scx)) & 0xFF, bg_y)
                };
                let tile = self.vram[map + (map_y / 8) * 32 + map_x / 8];
                let (lo, hi) = self.tile_row(self.bg_tile_addr(tile), map_y % 8);
                let bit = 7 - (map_x % 8);
                *slot = ((hi >> bit) & 1) << 1 | ((lo >> bit) & 1);
            }
            for (x, &idx) in bg_index.iter().enumerate() {
                self.framebuffer[row_start + x] = (self.bgp >> (idx * 2)) & 3;
            }
        } else {
            self.framebuffer[row_start..row_start + WIDTH].fill(0);
        }

        if self.lcdc & 0x02 == 0 || objects.is_empty() {
            return;
        }
        // DMG priority: lower X wins, then lower OAM index. Walk objects from
        // lowest to highest priority so higher-priority pixels overwrite.
        let mut order: [LineObject; 10] = [LineObject::default(); 10];
        order[..objects.len()].copy_from_slice(objects);
        let order = &mut order[..objects.len()];
        order.sort_unstable_by_key(|o| (o.x, o.index));
        let height = self.obj_height();
        // Which object owns each pixel (by position in `order`), if any.
        let mut owner = [u8::MAX; WIDTH];
        let mut color = [0u8; WIDTH];
        for (rank, o) in order.iter().enumerate().rev() {
            let mut row = self.line.wrapping_add(16).wrapping_sub(o.y);
            if o.attr & 0x40 != 0 {
                row = height - 1 - row;
            }
            let tile = if height == 16 { o.tile & 0xFE } else { o.tile };
            let (lo, hi) = self.tile_row(usize::from(tile) * 16, usize::from(row));
            for px in 0..8u8 {
                let sx = i32::from(o.x) - 8 + i32::from(px);
                if !(0..WIDTH as i32).contains(&sx) {
                    continue;
                }
                let bit = if o.attr & 0x20 != 0 { px } else { 7 - px };
                let c = ((hi >> bit) & 1) << 1 | ((lo >> bit) & 1);
                if c != 0 {
                    owner[sx as usize] = rank as u8;
                    color[sx as usize] = c;
                }
            }
        }
        for x in 0..WIDTH {
            if owner[x] == u8::MAX {
                continue;
            }
            let o = order[usize::from(owner[x])];
            if o.attr & 0x80 != 0 && bg_index[x] != 0 {
                continue; // behind BG colours 1–3
            }
            let pal = if o.attr & 0x10 != 0 { self.obp1 } else { self.obp0 };
            self.framebuffer[row_start + x] = (pal >> (color[x] * 2)) & 3;
        }
    }

    // --- debugger views ------------------------------------------------------------

    /// Raw register values for debuggers: LCDC, STAT, SCY, SCX, LY, LYC, BGP,
    /// OBP0, OBP1, WY, WX.
    pub fn registers(&self) -> [u8; 11] {
        [
            self.lcdc,
            self.read_register(0xFF41),
            self.scy,
            self.scx,
            self.read_register(0xFF44),
            self.lyc,
            self.bgp,
            self.obp0,
            self.obp1,
            self.wy,
            self.wx,
        ]
    }

    /// Dot position within the current line (0..455) and the internal line.
    pub fn position(&self) -> (u8, u16) {
        (self.line, self.dot)
    }

    /// Decodes tile `index` (0..384) of VRAM into 64 colour indices.
    pub fn decode_tile(&self, index: usize, out: &mut [u8; 64]) {
        let base = (index % 384) * 16;
        for row in 0..8 {
            let (lo, hi) = (self.vram[base + row * 2], self.vram[base + row * 2 + 1]);
            for px in 0..8 {
                let bit = 7 - px;
                out[row * 8 + px] = ((hi >> bit) & 1) << 1 | ((lo >> bit) & 1);
            }
        }
    }

    /// Current BG palette register (for mapping debugger views).
    pub fn bgp(&self) -> u8 {
        self.bgp
    }

    pub(crate) fn save(&self, w: &mut StateWriter) {
        w.u8s(&self.vram);
        w.u8s(&self.oam);
        w.u8s(&[
            self.lcdc,
            self.stat_select,
            self.scy,
            self.scx,
            self.lyc,
            self.bgp,
            self.obp0,
            self.obp1,
            self.wy,
            self.wx,
            self.mode as u8,
            self.line,
            self.ly,
            self.window_line,
        ]);
        w.u16(self.dot);
        w.u16(self.mode3_end);
        w.bool(self.window_y_hit);
        w.bool(self.stat_line);
        w.bool(self.first_line_after_on);
        w.bool(self.skip_frame);
        w.u32(self.off_dots);
        w.u8s(&self.framebuffer);
        w.u64(self.frame_count);
    }

    pub(crate) fn load(&mut self, r: &mut StateReader) -> Result<(), StateError> {
        r.u8s(&mut self.vram)?;
        r.u8s(&mut self.oam)?;
        let mut b = [0u8; 14];
        r.u8s(&mut b)?;
        [
            self.lcdc,
            self.stat_select,
            self.scy,
            self.scx,
            self.lyc,
            self.bgp,
            self.obp0,
            self.obp1,
            self.wy,
            self.wx,
        ] = [b[0], b[1] & 0x78, b[2], b[3], b[4], b[5], b[6], b[7], b[8], b[9]];
        self.mode = match b[10] {
            0 => Mode::HBlank,
            1 => Mode::VBlank,
            2 => Mode::OamScan,
            3 => Mode::Drawing,
            _ => return Err(StateError::Corrupt("ppu mode")),
        };
        if b[11] >= LINES_PER_FRAME {
            return Err(StateError::Corrupt("ppu line"));
        }
        [self.line, self.ly, self.window_line] = [b[11], b[12], b[13]];
        self.dot = r.u16()?;
        self.mode3_end = r.u16()?;
        if self.dot >= DOTS_PER_LINE {
            return Err(StateError::Corrupt("ppu dot"));
        }
        self.window_y_hit = r.bool()?;
        self.stat_line = r.bool()?;
        self.first_line_after_on = r.bool()?;
        self.skip_frame = r.bool()?;
        self.off_dots = r.u32()?;
        r.u8s(&mut self.framebuffer)?;
        self.frame_count = r.u64()?;
        self.frame_ready = false;
        Ok(())
    }
}

impl Default for Ppu {
    fn default() -> Self {
        Self::new()
    }
}
