//! The PPU: LCD timing, STAT/LYC interrupts and scanline rendering.
//!
//! **Timing** is modelled per dot (4 per M-cycle) as a small set of events per
//! line, placed where hardware tests put them (mooneye `lcdon_timing`,
//! `lcdon_write_timing`, `stat_lyc_onoff`, `intr_*`; cross-checked against
//! SameBoy's DMG state machine). The CPU accesses memory when the line-relative
//! dot counter is ≡ 3 (mod 4) — a phase fixed by the 449-dot first line after
//! the LCD is switched on — so an event at dot `d` is visible to a read at the
//! first access point ≥ `d`.
//!
//! **Rendering** happens in one pass per line at the start of mode 3, using the
//! registers current at that moment; mode 3 is stretched by the documented
//! SCX/window/object penalties so STAT timing stays exact (ADR-0002).
//! Mid-scanline raster effects need a pixel FIFO, which is on the roadmap.

use crate::state::{StateError, StateReader, StateWriter};

pub const WIDTH: usize = 160;
pub const HEIGHT: usize = 144;
const DOTS_PER_LINE: u16 = 456;
/// The first line after the LCD is switched on is 7 dots short.
const FIRST_LINE_DOTS: u16 = 449;
const LINES_PER_FRAME: u8 = 154;
/// Minimum mode 3 duration (SCX%8 = 0, no window, no objects).
const MODE3_BASE: u16 = 172;
/// Dots per frame; also the cadence of blank frames while the LCD is off.
pub const DOTS_PER_FRAME: u32 = 70_224;

/// IF bits the PPU raises.
const IRQ_VBLANK: u8 = 0x01;
const IRQ_STAT: u8 = 0x02;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum Mode {
    #[default]
    HBlank = 0,
    VBlank = 1,
    OamScan = 2,
    Drawing = 3,
}

impl Mode {
    fn from_u8(v: u8) -> Option<Self> {
        Some(match v {
            0 => Self::HBlank,
            1 => Self::VBlank,
            2 => Self::OamScan,
            3 => Self::Drawing,
            _ => return None,
        })
    }
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

/// Interrupt requests produced during one M-cycle, as IF bits.
///
/// The CPU samples interrupt lines two dots before it samples the data bus,
/// so requests raised in the last two dots of an M-cycle (`late`) can only be
/// dispatched after the following M-cycle — although they already show in IF.
#[derive(Clone, Copy, Debug, Default)]
pub struct PpuIrq {
    pub now: u8,
    pub late: u8,
}

pub struct Ppu {
    pub(crate) vram: [u8; 0x2000],
    pub(crate) oam: [u8; 0xA0],
    lcdc: u8,
    /// STAT interrupt-select bits 3..6.
    stat_select: u8,
    scy: u8,
    scx: u8,
    lyc: u8,
    bgp: u8,
    obp0: u8,
    obp1: u8,
    wy: u8,
    wx: u8,

    /// Mode reported in STAT bits 0–1.
    stat_mode: Mode,
    /// Mode feeding the STAT interrupt (None = no mode source asserted).
    irq_mode: Option<Mode>,
    /// Value LY is compared against (None = comparison momentarily off).
    ly_compare: Option<u8>,
    /// STAT bit 2. Latched: it keeps its value while the LCD is off.
    lyc_flag: bool,
    /// LYC half of the STAT interrupt line; holds while the comparison is off.
    lyc_line: bool,
    /// Combined STAT interrupt line (interrupts fire on rising edges).
    stat_line: bool,

    oam_read_block: bool,
    oam_write_block: bool,
    vram_read_block: bool,
    vram_write_block: bool,

    /// Internal line counter (0..=153).
    line: u8,
    /// Value exposed through the LY register (lags/leads `line` briefly).
    ly: u8,
    dot: u16,
    line_len: u16,
    /// This is the shortened first line after the LCD was switched on.
    first_line: bool,
    /// Dot at which mode 3 ends on the current line.
    mode3_end: u16,
    /// Internal window line counter.
    window_line: u8,
    /// WY matched LY at some point this frame (with the window enabled).
    wy_triggered: bool,
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
            .field("mode", &self.stat_mode)
            .field("line", &self.line)
            .field("dot", &self.dot)
            .field("lcdc", &self.lcdc)
            .finish_non_exhaustive()
    }
}

impl Ppu {
    /// State the DMG boot ROM leaves behind at PC=0x0100: LCD on, line 153
    /// with LY already reading 0 (STAT = 0x85), timed so that the CPU's
    /// accesses land on the same dot phase as after any LCD enable.
    pub fn new() -> Self {
        let mut p = Self::power_on();
        p.lcdc = 0x91;
        p.bgp = 0xFC;
        p.line = 153;
        p.ly = 0;
        p.dot = POST_BOOT_DOT;
        p.line_len = DOTS_PER_LINE;
        p.stat_mode = Mode::VBlank;
        p.irq_mode = Some(Mode::VBlank);
        p.ly_compare = Some(0);
        p.lyc_flag = true;
        p.lyc_line = true;
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
            stat_mode: Mode::HBlank,
            irq_mode: None,
            ly_compare: Some(0),
            lyc_flag: false,
            lyc_line: false,
            stat_line: false,
            oam_read_block: false,
            oam_write_block: false,
            vram_read_block: false,
            vram_write_block: false,
            line: 0,
            ly: 0,
            dot: 0,
            line_len: DOTS_PER_LINE,
            first_line: false,
            mode3_end: 84 + MODE3_BASE,
            window_line: 0,
            wy_triggered: false,
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

    /// Mode as reported in STAT (0 while the LCD is off).
    pub fn mode(&self) -> Mode {
        if self.lcd_on() { self.stat_mode } else { Mode::HBlank }
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

    pub fn vram_readable(&self) -> bool {
        !self.vram_read_block
    }
    pub fn vram_writable(&self) -> bool {
        !self.vram_write_block
    }
    pub fn oam_readable(&self) -> bool {
        !self.oam_read_block
    }
    pub fn oam_writable(&self) -> bool {
        !self.oam_write_block
    }

    pub fn read_register(&self, addr: u16) -> u8 {
        match addr {
            0xFF40 => self.lcdc,
            0xFF41 => 0x80 | self.stat_select | if self.lyc_flag { 0x04 } else { 0 } | self.mode() as u8,
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

    /// Returns IF bits raised by the write (spurious STAT interrupts).
    pub fn write_register(&mut self, addr: u16, value: u8) -> u8 {
        match addr {
            0xFF40 => {
                let was_on = self.lcd_on();
                self.lcdc = value;
                match (was_on, self.lcd_on()) {
                    (true, false) => self.turn_off(),
                    (false, true) => self.turn_on(),
                    _ => self.wy_check(),
                }
            }
            0xFF41 => {
                // DMG quirk: the write behaves as if 0xFF was written for one
                // cycle first, which can fire a spurious STAT interrupt.
                let mut irq = 0;
                if self.lcd_on() {
                    self.stat_select = 0x78;
                    irq |= self.update_stat();
                }
                self.stat_select = value & 0x78;
                irq |= self.update_stat();
                return irq;
            }
            0xFF42 => self.scy = value,
            0xFF43 => self.scx = value,
            0xFF44 => {} // read-only
            0xFF45 => {
                self.lyc = value;
                return self.update_stat();
            }
            0xFF47 => self.bgp = value,
            0xFF48 => self.obp0 = value,
            0xFF49 => self.obp1 = value,
            0xFF4A => {
                self.wy = value;
                self.wy_check();
            }
            0xFF4B => self.wx = value,
            _ => {}
        }
        0
    }

    fn turn_off(&mut self) {
        self.line = 0;
        self.ly = 0;
        self.dot = 0;
        self.stat_mode = Mode::HBlank;
        // The comparison clock stops: STAT bit 2 keeps its last value.
        self.ly_compare = Some(0);
        self.set_blocks(false);
        self.off_dots = 0;
        self.wy_triggered = false;
        self.framebuffer.fill(0);
    }

    fn turn_on(&mut self) {
        self.line = 0;
        self.ly = 0;
        self.dot = 0;
        self.line_len = FIRST_LINE_DOTS;
        self.first_line = true;
        self.stat_mode = Mode::HBlank;
        self.irq_mode = None;
        self.ly_compare = Some(0);
        self.set_blocks(false);
        self.window_line = 0;
        self.wy_triggered = false;
        self.skip_frame = true;
    }

    fn set_blocks(&mut self, blocked: bool) {
        self.oam_read_block = blocked;
        self.oam_write_block = blocked;
        self.vram_read_block = blocked;
        self.vram_write_block = blocked;
    }

    /// Recomputes STAT bit 2 and the interrupt line; returns IRQ_STAT on a
    /// rising edge.
    fn update_stat(&mut self) -> u8 {
        if !self.lcd_on() {
            return 0;
        }
        match self.ly_compare {
            Some(ly) => {
                self.lyc_flag = ly == self.lyc;
                self.lyc_line = self.lyc_flag;
            }
            // DMG: the flag reads 0 but the interrupt line holds its level.
            None => self.lyc_flag = false,
        }
        let s = self.stat_select;
        let mode_line = match self.irq_mode {
            Some(Mode::HBlank) => s & 0x08 != 0,
            Some(Mode::VBlank) => s & 0x10 != 0,
            Some(Mode::OamScan) => s & 0x20 != 0,
            _ => false,
        };
        let line = mode_line || (s & 0x40 != 0 && self.lyc_line);
        let rising = line && !self.stat_line;
        self.stat_line = line;
        if rising { IRQ_STAT } else { 0 }
    }

    fn wy_check(&mut self) {
        if !self.lcd_on() || self.wy_triggered || self.lcdc & 0x20 == 0 {
            return;
        }
        if self.wy == self.ly_compare.unwrap_or(self.line) {
            self.wy_triggered = true;
        }
    }

    // --- timing ------------------------------------------------------------------

    /// Advances one M-cycle (4 dots).
    #[inline]
    pub fn tick(&mut self) -> PpuIrq {
        let mut irq = PpuIrq::default();
        if !self.lcd_on() {
            self.off_dots += 4;
            if self.off_dots >= DOTS_PER_FRAME {
                self.off_dots -= DOTS_PER_FRAME;
                self.frame_ready = true;
                self.frame_count += 1;
            }
            return irq;
        }
        for i in 0..4 {
            self.dot += 1;
            if self.dot == self.line_len {
                self.dot = 0;
                self.next_line();
            }
            let raised = self.dot_event();
            if i < 2 {
                irq.now |= raised;
            } else {
                irq.late |= raised;
            }
        }
        irq
    }

    fn next_line(&mut self) {
        self.first_line = false;
        self.line_len = DOTS_PER_LINE;
        self.line += 1;
        if self.line == LINES_PER_FRAME {
            self.line = 0;
            self.window_line = 0;
            self.wy_triggered = false;
        }
    }

    /// Everything that happens at the current dot. Returns IF bits raised.
    fn dot_event(&mut self) -> u8 {
        let d = self.dot;
        let mut irq = 0;
        match self.line {
            0..=143 if self.first_line => match d {
                1 => irq |= self.update_stat(),
                77 => self.oam_write_block = true,
                79 => self.start_drawing(79),
                _ if d == self.mode3_end => self.end_drawing(),
                _ if d == self.mode3_end + 1 => {
                    self.irq_mode = Some(Mode::HBlank);
                    irq |= self.update_stat();
                }
                _ => {}
            },
            0..=143 => match d {
                0 => self.wy_check(),
                3 => {
                    self.ly = self.line;
                    self.oam_read_block = true;
                    self.stat_mode = Mode::HBlank;
                    if self.line == 0 {
                        self.ly_compare = Some(0);
                    } else {
                        self.ly_compare = None;
                        // The OAM interrupt fires a dot before STAT shows mode 2.
                        self.irq_mode = Some(Mode::OamScan);
                    }
                    irq |= self.update_stat();
                }
                4 => {
                    self.stat_mode = Mode::OamScan;
                    self.irq_mode = Some(Mode::OamScan);
                    self.oam_write_block = true;
                    self.ly_compare = Some(self.ly);
                    self.wy_check();
                    irq |= self.update_stat();
                    // The mode-2 source is a pulse, not a level.
                    self.irq_mode = None;
                    irq |= self.update_stat();
                }
                80 => {
                    // DMG: VRAM reads lock early; OAM writes briefly reopen.
                    self.vram_read_block = true;
                    self.oam_write_block = false;
                }
                84 => self.start_drawing(84),
                _ if d == self.mode3_end => self.end_drawing(),
                _ if d == self.mode3_end + 1 => {
                    self.irq_mode = Some(Mode::HBlank);
                    irq |= self.update_stat();
                }
                _ => {}
            },
            144..=152 => match d {
                0 => {
                    self.ly_compare = None;
                    irq |= self.update_stat();
                }
                2 => {
                    self.ly = self.line;
                    if self.line == 144 && !self.stat_line && self.stat_select & 0x20 != 0 {
                        irq |= IRQ_STAT;
                    }
                }
                4 => {
                    self.ly_compare = Some(self.line);
                    irq |= self.update_stat();
                }
                5 if self.line == 144 => {
                    self.stat_mode = Mode::VBlank;
                    irq |= IRQ_VBLANK;
                    // Entering VBlank also triggers the OAM (mode 2) source.
                    if !self.stat_line && self.stat_select & 0x20 != 0 {
                        irq |= IRQ_STAT;
                    }
                    self.irq_mode = Some(Mode::VBlank);
                    irq |= self.update_stat();
                    self.finish_frame();
                }
                _ => {}
            },
            _ => match d {
                // Line 153: LY reads 153 only briefly before wrapping to 0.
                0 => {
                    self.ly_compare = None;
                    irq |= self.update_stat();
                }
                2 => self.ly = 153,
                6 => {
                    self.ly = 0;
                    self.ly_compare = Some(153);
                    irq |= self.update_stat();
                }
                8 => {
                    self.ly_compare = None;
                    irq |= self.update_stat();
                }
                12 => {
                    self.ly_compare = Some(0);
                    irq |= self.update_stat();
                }
                _ => {}
            },
        }
        irq
    }

    fn finish_frame(&mut self) {
        self.frame_ready = true;
        self.frame_count += 1;
        if self.skip_frame {
            self.skip_frame = false;
            self.framebuffer.fill(0);
        }
    }

    fn start_drawing(&mut self, dot: u16) {
        self.stat_mode = Mode::Drawing;
        self.irq_mode = None;
        self.set_blocks(true);
        self.wy_check();
        let mut objects = [LineObject::default(); 10];
        let count = self.scan_oam(&mut objects);
        let objects = &objects[..count];
        let window = self.window_visible_on_line();
        self.mode3_end = dot + self.mode3_length(objects, window);
        self.render_line(objects, window);
        if window {
            self.window_line = self.window_line.wrapping_add(1);
        }
    }

    fn end_drawing(&mut self) {
        self.stat_mode = Mode::HBlank;
        self.set_blocks(false);
    }

    #[inline]
    fn window_visible_on_line(&self) -> bool {
        self.lcdc & 0x20 != 0 && self.wy_triggered && self.wx <= 166
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
        let mut len = MODE3_BASE + u16::from(self.scx & 7);
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
        len.min(DOTS_PER_LINE - 84 - 8)
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
            self.stat_mode as u8,
            self.irq_mode.map_or(0xFF, |m| m as u8),
            self.ly_compare.unwrap_or(0),
            u8::from(self.ly_compare.is_some()),
            self.line,
            self.ly,
            self.window_line,
        ]);
        for flag in [
            self.lyc_flag,
            self.lyc_line,
            self.stat_line,
            self.oam_read_block,
            self.oam_write_block,
            self.vram_read_block,
            self.vram_write_block,
            self.first_line,
            self.wy_triggered,
            self.skip_frame,
        ] {
            w.bool(flag);
        }
        w.u16(self.dot);
        w.u16(self.line_len);
        w.u16(self.mode3_end);
        w.u32(self.off_dots);
        w.u8s(&self.framebuffer);
        w.u64(self.frame_count);
    }

    pub(crate) fn load(&mut self, r: &mut StateReader) -> Result<(), StateError> {
        r.u8s(&mut self.vram)?;
        r.u8s(&mut self.oam)?;
        let mut b = [0u8; 17];
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
        self.stat_mode = Mode::from_u8(b[10]).ok_or(StateError::Corrupt("ppu mode"))?;
        self.irq_mode = match b[11] {
            0xFF => None,
            v => Some(Mode::from_u8(v).ok_or(StateError::Corrupt("ppu irq mode"))?),
        };
        self.ly_compare = (b[13] != 0).then_some(b[12]);
        if b[14] >= LINES_PER_FRAME {
            return Err(StateError::Corrupt("ppu line"));
        }
        [self.line, self.ly, self.window_line] = [b[14], b[15], b[16]];
        let mut flags = [false; 10];
        for f in &mut flags {
            *f = r.bool()?;
        }
        [
            self.lyc_flag,
            self.lyc_line,
            self.stat_line,
            self.oam_read_block,
            self.oam_write_block,
            self.vram_read_block,
            self.vram_write_block,
            self.first_line,
            self.wy_triggered,
            self.skip_frame,
        ] = flags;
        self.dot = r.u16()?;
        self.line_len = r.u16()?;
        self.mode3_end = r.u16()?;
        if self.line_len != DOTS_PER_LINE && self.line_len != FIRST_LINE_DOTS || self.dot >= self.line_len {
            return Err(StateError::Corrupt("ppu dot"));
        }
        self.off_dots = r.u32()?;
        r.u8s(&mut self.framebuffer)?;
        self.frame_count = r.u64()?;
        self.frame_ready = false;
        Ok(())
    }
}

/// Dot within line 153 at which the CPU first runs cartridge code after the
/// boot ROM (calibrated against mooneye `boot_hwio-dmgABCmgb`; ≡ 3 mod 4).
const POST_BOOT_DOT: u16 = 403;

impl Default for Ppu {
    fn default() -> Self {
        Self::new()
    }
}
