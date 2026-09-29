//! The PPU: LCD timing, STAT/LYC interrupts and pixel FIFO rendering.
//!
//! **Timing** is modelled per dot (4 per M-cycle) as a small set of events per
//! line, placed where hardware tests put them (mooneye `lcdon_timing`,
//! `lcdon_write_timing`, `stat_lyc_onoff`, `intr_*`; cross-checked against
//! SameBoy's DMG state machine). The CPU accesses memory when the line-relative
//! dot counter is ≡ 3 (mod 4) — a phase fixed by the 449-dot first line after
//! the LCD is switched on — so an event at dot `d` is visible to a read at the
//! first access point ≥ `d`.
//!
//! **Rendering** advances one dot at a time during mode 3. The tile fetcher
//! samples addresses and bitplanes separately; background and object FIFOs
//! hold colour indices until the live palettes are applied at the LCD output.
//! Outside mode 3 the event scheduler retains its M-cycle fast path.

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

/// Raw LCD output retained for DMG palette-write bus conflicts. A CPU write
/// reaches the palette latch before the normal end-of-M-cycle bus phase.
#[derive(Clone, Copy)]
struct OutputPixel {
    offset: u16,
    color: u8,
    palette: u8,
}

impl Default for OutputPixel {
    fn default() -> Self {
        Self { offset: u16::MAX, color: 0, palette: 0 }
    }
}

/// State of the two pixel queues and the in-flight VRAM reads. Kept separate
/// from LCD timing so a save in the middle of a tile fetch resumes exactly.
///
/// Sources: Pan Docs, "Pixel FIFO" and "Rendering overview";
/// https://github.com/gbdev/pandocs/blob/master/src/pixel_fifo.md
/// SameBoy's DMG fetcher documents the address/read phases and initial junk
/// pixels: https://github.com/LIJI32/SameBoy/blob/master/Core/display.c
#[derive(Clone)]
struct PixelPipeline {
    output: [OutputPixel; 2],
    bg: [u8; 8],
    bg_head: u8,
    bg_len: u8,
    obj: [u8; 8],
    obj_attr: [u8; 8],
    /// Tile address, tile read, low address/read, high address/read, push.
    phase: u8,
    address: u16,
    /// Address phases performed on the last two dots: none, map, low, high.
    /// CPU-visible scroll/control writes can precede the ordinary bus phase.
    fetch_latch: [u8; 2],
    fetch_latch_x: [i32; 2],
    tile: u8,
    low: u8,
    high: u8,
    /// Negative positions discard the initial junk and SCX fine scrolling.
    x: i32,
    startup: u8,
    window: bool,
    window_used: bool,
    window_y: u8,
    window_tile: u8,
    window_first: bool,
    window_carry: bool,
    objects: [LineObject; 10],
    object_count: u8,
    next_object: u8,
    object_stall: u8,
    object_address: u16,
    object_low: u8,
    object_waiting: bool,
    /// OBJ VRAM read on the last dot: none, low plane, high plane.
    object_read: u8,
    /// Before the last high-plane overlay, for a same-dot LCDC.2 collision.
    object_prior: [u8; 8],
    object_prior_attr: [u8; 8],
}

impl Default for PixelPipeline {
    fn default() -> Self {
        Self {
            output: [OutputPixel::default(); 2],
            bg: [0; 8],
            bg_head: 0,
            bg_len: 8,
            obj: [0; 8],
            obj_attr: [0; 8],
            phase: 0,
            address: 0,
            fetch_latch: [0; 2],
            fetch_latch_x: [0; 2],
            tile: 0,
            low: 0,
            high: 0,
            x: -16,
            startup: 4,
            window: false,
            window_used: false,
            window_y: 0,
            window_tile: 0,
            window_first: false,
            window_carry: false,
            objects: [LineObject::default(); 10],
            object_count: 0,
            next_object: 0,
            object_stall: 0,
            object_address: 0,
            object_low: 0,
            object_waiting: false,
            object_read: 0,
            object_prior: [0; 8],
            object_prior_attr: [0; 8],
        }
    }
}

impl PixelPipeline {
    fn save(&self, w: &mut StateWriter) {
        w.u8s(&self.bg);
        w.u8s(&self.obj);
        w.u8s(&self.obj_attr);
        w.u8s(&[
            self.bg_head,
            self.bg_len,
            self.phase,
            self.tile,
            self.low,
            self.high,
            self.startup,
            self.window_y,
            self.window_tile,
            self.object_count,
            self.next_object,
            self.object_stall,
            self.object_low,
        ]);
        w.u16(self.address);
        w.u16(self.object_address);
        w.i32(self.x);
        for flag in [self.window, self.window_used, self.window_first, self.window_carry, self.object_waiting] {
            w.bool(flag);
        }
        for o in self.objects {
            w.u8s(&[o.x, o.y, o.tile, o.attr, o.index]);
        }
        for pixel in self.output {
            w.u16(pixel.offset);
            w.u8(pixel.color);
            w.u8(pixel.palette);
        }
        w.u8s(&self.fetch_latch);
        for x in self.fetch_latch_x {
            w.i32(x);
        }
        w.u8(self.object_read);
        w.u8s(&self.object_prior);
        w.u8s(&self.object_prior_attr);
    }

    fn load(&mut self, r: &mut StateReader) -> Result<(), StateError> {
        r.u8s(&mut self.bg)?;
        r.u8s(&mut self.obj)?;
        r.u8s(&mut self.obj_attr)?;
        let mut b = [0; 13];
        r.u8s(&mut b)?;
        [
            self.bg_head,
            self.bg_len,
            self.phase,
            self.tile,
            self.low,
            self.high,
            self.startup,
            self.window_y,
            self.window_tile,
            self.object_count,
            self.next_object,
            self.object_stall,
            self.object_low,
        ] = b;
        self.address = r.u16()?;
        self.object_address = r.u16()?;
        self.x = r.i32()?;
        self.window = r.bool()?;
        self.window_used = r.bool()?;
        self.window_first = r.bool()?;
        self.window_carry = r.bool()?;
        self.object_waiting = r.bool()?;
        for o in &mut self.objects {
            let mut b = [0; 5];
            r.u8s(&mut b)?;
            [o.x, o.y, o.tile, o.attr, o.index] = b;
            if o.index >= 40 {
                return Err(StateError::Corrupt("ppu object index"));
            }
        }
        for pixel in &mut self.output {
            pixel.offset = r.u16()?;
            pixel.color = r.u8()?;
            pixel.palette = r.u8()?;
            if pixel.offset != u16::MAX && usize::from(pixel.offset) >= WIDTH * HEIGHT
                || pixel.color > 3
                || pixel.palette > 2
            {
                return Err(StateError::Corrupt("ppu palette latch"));
            }
        }
        r.u8s(&mut self.fetch_latch)?;
        for x in &mut self.fetch_latch_x {
            *x = r.i32()?;
        }
        if self.fetch_latch.iter().any(|&kind| kind > 3)
            || self.fetch_latch_x.iter().any(|&x| !(-16..=WIDTH as i32).contains(&x))
            || self.fetch_latch[0] != 0 && self.phase != self.fetch_latch[0] * 2 - 1
        {
            return Err(StateError::Corrupt("ppu fetch address latch"));
        }
        self.object_read = r.u8()?;
        r.u8s(&mut self.object_prior)?;
        r.u8s(&mut self.object_prior_attr)?;
        if self.object_read > 2
            || self.object_read != 0 && self.next_object == 0
            || self.object_read == 1 && self.object_stall != 2
            || self.object_read == 2 && self.object_stall != 0
            || self.object_prior.iter().any(|&color| color > 3)
            || self.object_prior_attr.iter().any(|&attr| attr & !0x90 != 0)
        {
            return Err(StateError::Corrupt("ppu object read latch"));
        }
        if self.bg_head >= 8
            || self.bg_len > 8
            || self.phase > 6
            || self.startup > 4
            || self.window_tile >= 32
            || self.object_count > 10
            || self.next_object > self.object_count
            || self.object_stall > 6
            || self.object_stall != 0 && self.next_object == 0
            || self.address >= 0x2000
            || self.object_address >= 0x1000
            || self.object_address & 1 != 0
            || !(-16..=WIDTH as i32).contains(&self.x)
            || self.object_waiting && self.object_stall != 6
            || self.bg.iter().chain(self.obj.iter()).any(|&c| c > 3)
            || self.obj_attr.iter().any(|&a| a & !0x90 != 0)
        {
            return Err(StateError::Corrupt("ppu pixel pipeline"));
        }
        Ok(())
    }
}

/// The window comparator shares a clock edge with LCDC writes. Keep just
/// the restart dot until that CPU bus phase has settled; disabling WIN_EN
/// on that edge cancels the restart before it clears the background queue.
struct WindowRestart {
    pixels: PixelPipeline,
    window_line: u8,
}

/// Interrupt requests produced during one M-cycle, as IF bits.
///
/// A halted CPU samples interrupt lines halfway through its idle M-cycle;
/// `late` requests miss that sample but remain visible to a running CPU.
/// The same split locates reassertions after the ISR's acknowledge pulse.
#[derive(Clone, Copy, Debug, Default)]
pub struct PpuIrq {
    pub now: u8,
    pub late: u8,
    /// Edges after the first dot, for the DMG CPU's IF-write conflict.
    pub after_first: u8,
}

pub struct Ppu {
    pub(crate) vram: [u8; 0x2000],
    pub(crate) oam: [u8; 0xA0],
    lcdc: u8,
    /// STAT interrupt-select bits 3..6.
    stat_select: u8,
    /// Final STAT interrupt mask after its one-dot DMG write conflict.
    stat_write: u8,
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
    /// Next dot (after `dot`) with a timing event on this line, or `line_len`.
    /// Lets `tick` skip whole M-cycles in which nothing happens.
    next_event: u16,
    /// Internal window line counter.
    window_line: u8,
    /// WY matched LY at some point this frame (with the window enabled).
    wy_triggered: bool,
    /// WY/LCDC writes reach the vertical-window comparison on its next clock.
    wy_check_delay: u8,
    /// First frame after LCD-on is not shown (hardware outputs blank).
    skip_frame: bool,
    /// Dots counted while the LCD is off, to keep emitting frames.
    off_dots: u32,

    pixels: PixelPipeline,
    window_restart: Option<WindowRestart>,
    framebuffer: [u8; WIDTH * HEIGHT],
    frame_ready: bool,
    frame_count: u64,
    /// Per visible line: mode 3 length (bits 0–8), objects on the line
    /// (bits 9–12) and window (bit 13). Debug view only; not saved.
    line_timing: [u16; HEIGHT],
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
        p.next_event = p.compute_next_event();
        p
    }

    /// Writes the VRAM contents the DMG boot ROM leaves behind: the logo from
    /// the cartridge header (0x0104..0x0134) scaled 2x into tiles 1-24, a ®
    /// in tile 25, and the tile map entries that centre them on rows 8-9.
    /// Software that never clears VRAM shows (or scrolls) this logo.
    pub fn load_boot_logo(&mut self, logo: &[u8]) {
        const TRADEMARK: [u8; 8] = [0x3C, 0x42, 0xB9, 0xA5, 0xB9, 0xA5, 0x42, 0x3C];
        let mut addr = 0x0010;
        for &byte in logo.iter().take(48) {
            // Each nibble becomes one 8-pixel row, every bit doubled
            // (b3 b2 b1 b0 -> b3 b3 b2 b2 b1 b1 b0 b0), written twice; only
            // the low bit plane is set, so the logo uses colour 1.
            for nibble in [byte >> 4, byte & 0x0F] {
                let row = (0..4).fold(0u8, |acc, i| (acc << 2) | (((nibble >> (3 - i)) & 1) * 0b11));
                self.vram[addr] = row;
                self.vram[addr + 2] = row;
                addr += 4;
            }
        }
        for (i, &row) in TRADEMARK.iter().enumerate() {
            self.vram[0x0190 + 2 * i] = row;
        }
        self.vram[0x1910] = 25;
        for i in 0..12u8 {
            self.vram[0x1904 + usize::from(i)] = 1 + i;
            self.vram[0x1924 + usize::from(i)] = 13 + i;
        }
    }

    pub fn power_on() -> Self {
        Self {
            vram: [0; 0x2000],
            oam: [0; 0xA0],
            lcdc: 0,
            stat_select: 0,
            stat_write: 0xFF,
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
            next_event: 1,
            window_line: 0,
            wy_triggered: false,
            wy_check_delay: 0,
            skip_frame: false,
            off_dots: 0,
            pixels: PixelPipeline::default(),
            window_restart: None,
            framebuffer: [0; WIDTH * HEIGHT],
            frame_ready: false,
            frame_count: 0,
            line_timing: [0; HEIGHT],
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

    /// A frame completed and nobody has taken it yet.
    pub fn frame_ready_pending(&self) -> bool {
        self.frame_ready
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

    // --- OAM corruption bug (DMG) ---------------------------------------------------
    //
    // During mode 2 the PPU reads OAM one 8-byte row per M-cycle over a 16-bit
    // bus. A CPU access to FE00–FEFF in that window — a read, a write, or just
    // the increment/decrement unit putting such an address on the bus — drives
    // the same lines and garbles the row being read. Pan Docs, "OAM Corruption
    // Bug"; patterns and timing from SameBoy's DMG model (`GB_trigger_oam_bug`,
    // `GB_trigger_oam_bug_read`, `read_high_memory`); checked by Blargg's
    // `oam_bug` tests. Every pattern is bitwise, so the 16-bit words are
    // processed a byte at a time.

    /// Byte offset of the row the object search is reading, as a CPU access at
    /// the current dot sees it: row 0 from the start of a visible line, row 1
    /// from dot 6, one row further per M-cycle, running past the end of OAM
    /// (0xA0) at dot 82; none from mode 3 on, in VBlank, on the first line
    /// after the LCD is switched on, or with the LCD off.
    fn oam_scan_row(&self) -> Option<usize> {
        if !self.lcd_on() || self.first_line || self.line >= 144 || self.dot >= 84 {
            return None;
        }
        Some(if self.dot < 6 { 0 } else { usize::from(self.dot - 2) / 4 * 8 })
    }

    /// A write to FE00–FEFF while OAM is locked, or the increment/decrement
    /// unit driving such an address: the row takes a blend of its first word
    /// with the preceding row's first and third words, and the rest of the
    /// preceding row.
    pub fn oam_bug_write(&mut self) {
        let Some(row) = self.oam_scan_row().filter(|r| (8..0xA0).contains(r)) else {
            return;
        };
        let o = &mut self.oam;
        for k in 0..2 {
            let (a, b, c) = (o[row + k], o[row - 8 + k], o[row - 4 + k]);
            o[row + k] = ((a ^ c) & (b ^ c)) ^ c;
        }
        o.copy_within(row - 6..row, row + 2);
    }

    /// A read of `addr` (FE00–FEFF) while OAM is locked. `dma`: an OAM DMA
    /// transfer is running, which answers reads in the lock gaps itself.
    pub fn oam_bug_read(&mut self, addr: u16, dma: bool) {
        let Some(row) = self.oam_scan_row() else {
            return;
        };
        if self.oam_write_block {
            if (8..0xA0).contains(&row) {
                self.oam_bug_read_row(row);
            }
        } else if self.oam_read_block && !dma && addr < 0xFEA0 {
            // Reads lock a dot before writes at the start of mode 2 and stay
            // locked after writes reopen at its end; a read in either gap
            // disturbs row 0 or the last row, depending on the address.
            match row {
                0 => self.oam_bug_read_first_row(addr),
                0xA0 => self.oam_bug_read_last_row(addr),
                _ => {}
            }
        }
    }

    /// The read corruption proper. What it does depends on the row's position
    /// within each group of four rows (DMG-B values; some cases differ
    /// between individual consoles).
    fn oam_bug_read_row(&mut self, row: usize) {
        let o = &mut self.oam;
        match row & 0x18 {
            0x10 => {
                // Rows 2, 6, 10, 14, 18: the preceding row's first word is
                // blended with its neighbours and copied two rows back.
                for k in 0..2 {
                    let (a, b, c, d) = (o[row - 16 + k], o[row - 8 + k], o[row + k], o[row - 4 + k]);
                    o[row - 8 + k] = (b & (a | c | d)) | (a & c & d);
                }
                o.copy_within(row - 8..row, row - 16);
            }
            0x00 => {
                // Rows 4, 8, 12, 16: as above, reaching four rows back.
                for k in 0..2 {
                    let (a, b, c, d, e) =
                        (o[row + k], o[row - 4 + k], o[row - 8 + k], o[row - 16 + k], o[row - 32 + k]);
                    o[row - 8 + k] = match row {
                        0x20 => (c & (a | b | d | e)) | (a & b & d & e),
                        0x40 => {
                            let (f, g) = (o[row - 6 + k], o[row - 14 + k]);
                            (c & (e | d | (!f & g) | b | a)) | (b & d & e)
                        }
                        0x60 => (c & (a | b | d | e)) | (b & d & e),
                        _ => c | (a & b & d & e),
                    };
                }
                o.copy_within(row - 8..row, row - 16);
                o.copy_within(row - 8..row, row - 32);
            }
            _ => {
                // Odd rows: Pan Docs' plain read corruption, also applied to
                // the preceding row's first word.
                for k in 0..2 {
                    let v = o[row - 8 + k] | (o[row + k] & o[row - 4 + k]);
                    o[row - 8 + k] = v;
                    o[row + k] = v;
                }
            }
        }
        o.copy_within(row - 8..row, row);
        if row == 0x80 {
            o.copy_within(0x80..0x88, 0);
        }
    }

    /// A read at the dot between the read and write locks at the start of
    /// mode 2: the addressed row is copied into row 0, the first word of both
    /// glitched.
    fn oam_bug_read_first_row(&mut self, addr: u16) {
        let (row, word) = (usize::from(addr & 0xF8), usize::from(addr & 0xFE));
        let o = &mut self.oam;
        for k in 0..2 {
            let v = o[row + k] | (o[k] & o[word + k]);
            o[row + k] = v;
            o[k] = v;
        }
        o.copy_within(row + 2..row + 8, 2);
    }

    /// A read after writes reopen at the end of mode 2: the word at the
    /// address's position in the last row is glitched, and the last row is
    /// copied over the addressed row.
    fn oam_bug_read_last_row(&mut self, addr: u16) {
        let target = usize::from(addr & 6) | 0x98;
        let row = usize::from(addr & 0xF8);
        let o = &mut self.oam;
        for k in 0..2 {
            let (a, b) = (o[0x9C + k], o[target + k]);
            let c = if addr & 6 == 2 { o[usize::from(addr & 0xFE) + k] } else { o[row + k] };
            o[target + k] = match addr & 6 {
                0 | 2 => (a & b) | (a & c) | (b & c),
                4 => b,
                _ => b | (a & c),
            };
        }
        o.copy_within(0x98..0xA0, row);
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
                let old_lcdc = self.lcdc;
                self.lcdc = value;
                if was_on && self.lcd_on() && old_lcdc & 0x20 != 0 && value & 0x20 == 0 {
                    self.cancel_window_restart();
                }
                match (was_on, self.lcd_on()) {
                    (true, false) => self.turn_off(),
                    (false, true) => self.turn_on(),
                    _ => {}
                }
                if self.lcd_on() {
                    if was_on {
                        self.refresh_fetch_address();
                        if (old_lcdc ^ value) & 4 != 0 {
                            self.refresh_object_read();
                        }
                    }
                    self.wy_check_delay = 4;
                }
            }
            0xFF41 => {
                if !self.lcd_on() {
                    self.stat_select = value & 0x78;
                    return 0;
                }
                // DMG drives all STAT enables for one dot before the written
                // value takes effect. At the HBlank->OAM boundary an already
                // enabled HBlank source masks that transient OAM enable.
                // SameBoy Core/sm83_cpu.c, GB_CONFLICT_STAT_DMG.
                let boundary = !self.first_line && self.line < 144 && self.dot == 3;
                self.stat_write = value & 0x78;
                self.stat_select = if boundary && self.stat_select & 0x28 == 0x08 { 0x58 } else { 0x78 };
                return self.update_stat();
            }
            0xFF42 => {
                self.scy = value;
                self.refresh_fetch_address();
            }
            0xFF43 => {
                self.scx = value;
                self.refresh_scroll_x();
            }
            0xFF44 => {} // read-only
            0xFF45 => {
                self.lyc = value;
                return self.update_stat();
            }
            0xFF47 => {
                self.palette_write(0, self.bgp, value);
                self.bgp = value;
            }
            0xFF48 => {
                self.palette_write(1, self.obp0, value);
                self.obp0 = value;
            }
            0xFF49 => {
                self.palette_write(2, self.obp1, value);
                self.obp1 = value;
            }
            0xFF4A => {
                self.wy = value;
                self.wy_check_delay = 4;
            }
            0xFF4B => self.wx = value,
            _ => {}
        }
        0
    }

    /// LCDC's window enable reaches the comparator one dot before the normal
    /// write phase (SameBoy GB_CONFLICT_DMG_LCDC). If that dot just restarted
    /// the fetcher, replay only it with the new enable; no LCD pixel had left
    /// the empty window FIFO, so earlier pixels and peripheral timing stand.
    fn cancel_window_restart(&mut self) {
        let Some(restart) = self.window_restart.take() else { return };
        self.pixels = restart.pixels;
        self.window_line = restart.window_line;
        self.draw_dot();
        if self.pixels.x == WIDTH as i32 {
            self.mode3_end = self.dot;
            self.end_drawing();
            self.next_event = self.compute_next_event();
        }
    }

    /// On DMG a palette write shares the bus with two already clocked LCD
    /// samples: old|new for the first dot, then new for the second. Keeping
    /// the raw samples models that conflict without retiming STAT or fetches.
    /// SameBoy Core/sm83_cpu.c, GB_CONFLICT_DMG_PALETTE.
    fn palette_write(&mut self, palette: u8, old: u8, new: u8) {
        for (pixel, value) in self.pixels.output.iter().zip([new, old | new]) {
            if pixel.offset != u16::MAX && pixel.palette == palette {
                self.framebuffer[usize::from(pixel.offset)] = (value >> (pixel.color * 2)) & 3;
            }
        }
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
        self.wy_check_delay = 0;
        self.stat_write = 0xFF;
        self.window_restart = None;
        self.pixels.fetch_latch = [0; 2];
        self.pixels.object_read = 0;
        self.framebuffer.fill(0);
        self.pixels.output = [OutputPixel::default(); 2];
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
        self.next_event = self.compute_next_event();
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
    /// One M-cycle with the system clock stopped (CPU in STOP). On a DMG the
    /// PPU does not advance and the LCD shows no picture; hosts still get a
    /// blank frame every 70,224 dots so their frame loops keep running.
    pub fn tick_stopped(&mut self) {
        self.off_dots += 4;
        if self.off_dots >= DOTS_PER_FRAME {
            self.off_dots -= DOTS_PER_FRAME;
            self.framebuffer.fill(0);
            self.frame_ready = true;
            self.frame_count += 1;
        }
    }

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
        if self.stat_mode != Mode::Drawing
            && self.wy_check_delay == 0
            && self.stat_write == 0xFF
            && self.dot + 4 < self.next_event
        {
            // Fast path: nothing happens during these four dots.
            self.dot += 4;
            self.pixels.output = [OutputPixel::default(); 2];
            self.pixels.fetch_latch = [0; 2];
            self.pixels.object_read = 0;
            self.window_restart = None;
            return irq;
        }
        for i in 0..4 {
            self.pixels.object_read = 0;
            self.pixels.fetch_latch[1] = self.pixels.fetch_latch[0];
            self.pixels.fetch_latch_x[1] = self.pixels.fetch_latch_x[0];
            self.pixels.fetch_latch[0] = 0;
            self.pixels.fetch_latch_x[0] = 0;
            self.window_restart = None;
            self.pixels.output[1] = self.pixels.output[0];
            self.pixels.output[0] = OutputPixel::default();
            self.dot += 1;
            if self.dot == self.line_len {
                self.dot = 0;
                self.next_line();
            }
            if self.wy_check_delay != 0 {
                self.wy_check_delay -= 1;
                if self.wy_check_delay == 0 {
                    self.wy_check();
                }
            }
            if self.stat_mode == Mode::Drawing && self.line < HEIGHT as u8 {
                self.draw_dot();
                // The last pixel leaves on the same dot that opens HBlank.
                // Dynamic window/OBJ writes can change the original estimate.
                if self.pixels.x == WIDTH as i32 {
                    self.mode3_end = self.dot;
                } else if self.dot >= self.mode3_end {
                    self.mode3_end = self.dot + 1;
                }
            }
            let raised = self.dot_event();
            if i != 0 {
                irq.after_first |= raised;
            }
            if i < 2 {
                irq.now |= raised;
            } else {
                irq.late |= raised;
            }
            if self.stat_mode != Mode::Drawing {
                self.next_event = self.compute_next_event();
            }
        }
        irq
    }

    /// The first dot after the current one at which `dot_event` acts.
    fn compute_next_event(&self) -> u16 {
        const FIRST_LINE: [u16; 3] = [1, 77, 79];
        const VISIBLE: [u16; 4] = [3, 4, 80, 84];
        const VBLANK: [u16; 3] = [2, 4, 5];
        const LAST_LINE: [u16; 4] = [2, 6, 8, 13];
        let fixed: &[u16] = match self.line {
            0..=143 if self.first_line => &FIRST_LINE,
            0..=143 => &VISIBLE,
            144..=152 => &VBLANK,
            _ => &LAST_LINE,
        };
        let d = self.dot;
        let mut next = self.line_len;
        if let Some(&e) = fixed.iter().find(|&&e| e > d) {
            next = e;
        }
        if self.line < 144 {
            for e in [self.mode3_end, self.mode3_end + 1] {
                if e > d && e < next {
                    next = e;
                }
            }
        }
        next
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
        // The HBlank/OAM write conflict keeps the OAM enable masked until
        // the boundary pulse has passed; other STAT writes settle before
        // evaluating the next dot's edges.
        let masked_oam = self.stat_select == 0x58 && d == 4 && !self.first_line && self.line < 144;
        if self.stat_write != 0xFF && !masked_oam {
            self.stat_select = core::mem::replace(&mut self.stat_write, 0xFF);
            irq |= self.update_stat();
        }
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
                    // WY samples this mode-2 comparison clock, including on
                    // line 0. Sampling at dot 0 would use the previous frame's
                    // already-zero LY comparator and latch WY too early.
                    // Gambatte window/{arg/,}late_wy_1 changes WY at dot 3.
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
                // The LY=0 comparison edge follows an IF write at dot 12
                // (Gambatte lycint152_lyc0irq_ifw_1), before the next CPU read.
                13 => {
                    self.ly_compare = Some(0);
                    irq |= self.update_stat();
                }
                _ => {}
            },
        }
        if self.stat_write != 0xFF {
            self.stat_select = core::mem::replace(&mut self.stat_write, 0xFF);
            irq |= self.update_stat();
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
        let mut objects = [LineObject::default(); 10];
        let count = self.scan_oam(&mut objects);
        let objects = &objects[..count];
        let window = self.window_visible_on_line();
        let length = self.mode3_length(objects, window);
        self.mode3_end = dot + length;
        if let Some(t) = self.line_timing.get_mut(usize::from(self.line)) {
            *t = length | (count as u16) << 9 | u16::from(window) << 13;
        }
        let carry = self.pixels.window_carry && self.wy_triggered && self.wx == 166 && self.lcdc & 0x20 != 0;
        let window_y = self.pixels.window_y;
        self.pixels = PixelPipeline::default();
        if carry {
            self.pixels.window = true;
            self.pixels.window_used = true;
            self.pixels.window_tile = 1;
            self.pixels.window_y = window_y;
        }
        self.pixels.objects[..count].copy_from_slice(objects);
        self.pixels.objects[..count].sort_unstable_by_key(|o| (o.x, o.index));
        self.pixels.object_count = count as u8;
    }

    fn end_drawing(&mut self) {
        // WX=166 is latched at the right edge and carries the window's
        // second tile into the next scanline (SameBoy's DMG state machine).
        self.pixels.window_carry = self.wy_triggered && self.lcdc & 0x20 != 0 && self.wx == 166;
        if self.pixels.window_carry {
            self.pixels.window_y = self.window_line;
            self.window_line = self.window_line.wrapping_add(1);
        }
        if let Some(t) = self.line_timing.get_mut(usize::from(self.line)) {
            let start = if self.first_line { 79 } else { 84 };
            *t = self.dot.saturating_sub(start)
                | u16::from(self.pixels.object_count) << 9
                | u16::from(self.pixels.window_used) << 13;
        }
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

    /// VRAM offset of a BG/window tile's pixel data.
    #[inline]
    fn bg_tile_addr(&self, tile: u8) -> usize {
        if self.lcdc & 0x10 != 0 { usize::from(tile) * 16 } else { (0x1000 + i32::from(tile as i8) * 16) as usize }
    }

    fn fetch_y(&self) -> u8 {
        if self.pixels.window { self.pixels.window_y } else { self.line.wrapping_add(self.scy) }
    }

    fn map_address(&self, fetch_x: i32) -> u16 {
        let map = if self.lcdc & if self.pixels.window { 0x40 } else { 0x08 } != 0 { 0x1C00 } else { 0x1800 };
        let x = if self.pixels.window {
            self.pixels.window_tile
        } else if fetch_x < -8 {
            self.scx >> 3
        } else {
            ((i32::from(self.scx) + fetch_x + 8) / 8) as u8 & 31
        };
        map + u16::from(self.fetch_y() / 8) * 32 + u16::from(x)
    }

    fn tile_data_address(&self, high: bool) -> u16 {
        (self.bg_tile_addr(self.pixels.tile) + usize::from(self.fetch_y() & 7) * 2 + usize::from(high)) as u16
    }

    /// SCY and LCDC reach the fetcher's address latches one dot before the
    /// ordinary CPU write phase. Correct only the address formed on that
    /// dot: earlier addresses and completed VRAM reads remain latched.
    /// SameBoy Core/sm83_cpu.c: READ_NEW (SCY), DMG_LCDC (full value).
    fn refresh_fetch_address(&mut self) {
        match self.pixels.fetch_latch[0] {
            1 => {
                if self.lcdc & 0x20 == 0 {
                    self.pixels.window = false;
                }
                self.pixels.address = self.map_address(self.pixels.fetch_latch_x[0]);
            }
            2 => self.pixels.address = self.tile_data_address(false),
            3 => self.pixels.address = self.tile_data_address(true),
            _ => {}
        }
    }

    /// SCX reaches the BG map address bus two dots before an ordinary CPU
    /// write (SameBoy GB_CONFLICT_SCX_DMG_AND_CGB_DOUBLE). A map address from
    /// the preceding dot has already completed its VRAM read, so refresh
    /// that tile byte too. Pixel data from older fetches stays in the FIFO.
    fn refresh_scroll_x(&mut self) {
        if self.pixels.window {
            return;
        }
        if self.pixels.fetch_latch[1] == 1 && self.pixels.phase == 2 {
            self.pixels.address = self.map_address(self.pixels.fetch_latch_x[1]);
            self.pixels.tile = self.vram[usize::from(self.pixels.address)];
        } else if self.pixels.fetch_latch[0] == 1 {
            self.pixels.address = self.map_address(self.pixels.fetch_latch_x[0]);
        }
    }

    /// One dot of the fetcher. On DMG, SCY and LCDC.4 are sampled separately
    /// for each bitplane, so a mid-fetch write can mix two rows/tile sets.
    /// See Mealybug's "The Comprehensive Game Boy PPU Documentation".
    fn fetch_dot(&mut self) {
        self.pixels.fetch_latch[0] = 0;
        match self.pixels.phase {
            0 => {
                if self.lcdc & 0x20 == 0 {
                    self.pixels.window = false;
                }
                self.pixels.address = self.map_address(self.pixels.x);
                self.pixels.fetch_latch[0] = 1;
                self.pixels.fetch_latch_x[0] = self.pixels.x;
                self.pixels.phase = 1;
            }
            1 => {
                self.pixels.tile = self.vram[usize::from(self.pixels.address)];
                self.pixels.phase = 2;
            }
            2 | 4 => {
                self.pixels.address = self.tile_data_address(self.pixels.phase == 4);
                self.pixels.fetch_latch[0] = if self.pixels.phase == 4 { 3 } else { 2 };
                self.pixels.fetch_latch_x[0] = self.pixels.x;
                self.pixels.phase += 1;
            }
            3 => {
                self.pixels.low = self.vram[usize::from(self.pixels.address)];
                self.pixels.phase = 4;
            }
            5 => {
                self.pixels.high = self.vram[usize::from(self.pixels.address)];
                if self.pixels.window {
                    self.pixels.window_tile = (self.pixels.window_tile + 1) & 31;
                }
                self.pixels.phase = 6;
                self.push_bg();
            }
            _ => self.push_bg(),
        }
    }

    fn push_bg(&mut self) {
        if self.pixels.bg_len != 0 {
            return;
        }
        for (i, pixel) in self.pixels.bg.iter_mut().enumerate() {
            let bit = 7 - i;
            *pixel = ((self.pixels.low >> bit) & 1) | (((self.pixels.high >> bit) & 1) << 1);
        }
        self.pixels.bg_head = 0;
        self.pixels.bg_len = 8;
        self.pixels.phase = 0;
    }

    fn trigger_window(&mut self) {
        if self.pixels.window || !self.wy_triggered || self.lcdc & 0x20 == 0 {
            return;
        }
        let x = self.pixels.x;
        if self.wx == 166 && x == 159 {
            self.window_line = self.window_line.wrapping_add(1);
        }
        let trigger = if self.wx == 0 {
            x == -7 || (-16..=-8).contains(&x) && (x != -16 || self.scx & 7 != 0)
        } else {
            self.wx <= 165 && i32::from(self.wx) == x + 7
        };
        if trigger {
            self.window_restart = Some(WindowRestart { pixels: self.pixels.clone(), window_line: self.window_line });
            self.pixels.window = true;
            self.pixels.window_used = true;
            self.pixels.window_y = self.window_line;
            self.window_line = self.window_line.wrapping_add(1);
            self.pixels.window_tile = 0;
            self.pixels.window_first = true;
            self.pixels.bg_len = 0;
            self.pixels.phase = 0;
        }
    }

    /// Starts an object fetch at its left edge. The BG fetcher first reaches
    /// its high-byte phase with a nonempty FIFO, then the object takes six
    /// dots. A simultaneous window restart must refill BG before that wait
    /// can complete; its six dots must not be absorbed into the OBJ delay.
    fn begin_object(&mut self) -> bool {
        while self.pixels.next_object < self.pixels.object_count {
            let o = self.pixels.objects[usize::from(self.pixels.next_object)];
            let match_x = (self.pixels.x + 8).max(0);
            if i32::from(o.x) > match_x || o.x >= 168 {
                break;
            }
            self.pixels.next_object += 1;
            if self.lcdc & 2 == 0 || i32::from(o.x) < match_x {
                continue;
            }
            self.pixels.object_waiting = true;
            self.pixels.object_stall = 6;
            self.pixels.object_address = self.object_tile_address(o);
            return true;
        }
        false
    }

    fn object_tile_address(&self, o: LineObject) -> u16 {
        let height = self.obj_height();
        let mut row = self.line.wrapping_add(16).wrapping_sub(o.y) & (height - 1);
        if o.attr & 0x40 != 0 {
            row ^= height - 1;
        }
        let tile = if height == 16 { o.tile & 0xFE } else { o.tile };
        u16::from(tile) * 16 + u16::from(row) * 2
    }

    /// The same -1-dot LCDC bus phase also reaches an OBJ read performed
    /// on the last dot. No LCD pixel was emitted during that fetch stall;
    /// restore its prior OBJ queue before repeating a high-plane overlay.
    fn refresh_object_read(&mut self) {
        if self.pixels.object_read == 0 {
            return;
        }
        let o = self.pixels.objects[usize::from(self.pixels.next_object - 1)];
        self.pixels.object_address = self.object_tile_address(o);
        if self.pixels.object_read == 1 {
            self.pixels.object_low = self.vram[usize::from(self.pixels.object_address)];
        } else {
            self.pixels.obj = self.pixels.object_prior;
            self.pixels.obj_attr = self.pixels.object_prior_attr;
            let high = self.vram[usize::from(self.pixels.object_address) + 1];
            self.overlay_object(o, high);
        }
    }

    fn overlay_object(&mut self, o: LineObject, high: u8) {
        for i in 0..8 {
            let bit = if o.attr & 0x20 == 0 { 7 - i } else { i };
            let color = ((self.pixels.object_low >> bit) & 1) | (((high >> bit) & 1) << 1);
            if self.pixels.obj[i] == 0 && color != 0 {
                self.pixels.obj[i] = color;
                self.pixels.obj_attr[i] = o.attr & 0x90;
            }
        }
    }

    fn object_dot(&mut self) {
        self.pixels.object_read = 0;
        if self.lcdc & 2 == 0 {
            // DMG cancels an in-flight object fetch when OBJ enable clears;
            // already spent dots remain, but no new OBJ pixels are queued.
            self.pixels.object_stall = 0;
            self.pixels.object_waiting = false;
            self.pop_pixel();
            self.fetch_dot();
            return;
        }
        if self.pixels.object_waiting {
            if self.pixels.phase < 5 || self.pixels.bg_len == 0 {
                self.fetch_dot();
                return;
            }
            self.pixels.object_waiting = false;
        }
        let remaining = self.pixels.object_stall;
        if remaining >= 5 {
            self.fetch_dot();
        }
        if remaining == 3 {
            // LCDC.2 is sampled again for each bitplane, including after a
            // BG-fetch wait. SameBoy display.c: get_object_line_address().
            let o = self.pixels.objects[usize::from(self.pixels.next_object - 1)];
            self.pixels.object_address = self.object_tile_address(o);
            self.pixels.object_low = self.vram[usize::from(self.pixels.object_address)];
            self.pixels.object_read = 1;
        }
        if remaining == 1 {
            let o = self.pixels.objects[usize::from(self.pixels.next_object - 1)];
            self.pixels.object_address = self.object_tile_address(o);
            let high = self.vram[usize::from(self.pixels.object_address) + 1];
            self.pixels.object_prior = self.pixels.obj;
            self.pixels.object_prior_attr = self.pixels.obj_attr;
            self.overlay_object(o, high);
            self.pixels.object_read = 2;
        }
        self.pixels.object_stall -= 1;
    }

    fn draw_dot(&mut self) {
        if self.pixels.startup != 0 {
            self.pixels.startup -= 1;
            return;
        }
        if self.pixels.object_stall != 0 {
            self.object_dot();
            return;
        }
        self.trigger_window();
        if self.begin_object() {
            self.object_dot();
            return;
        }
        self.pop_pixel();
        self.fetch_dot();
    }

    fn pop_pixel(&mut self) {
        if self.pixels.bg_len == 0 {
            return;
        }
        let bg = self.pixels.bg[usize::from(self.pixels.bg_head)];
        self.pixels.bg_head = (self.pixels.bg_head + 1) & 7;
        self.pixels.bg_len -= 1;
        let obj = self.pixels.obj[0];
        let attr = self.pixels.obj_attr[0];
        self.pixels.obj.copy_within(1..8, 0);
        self.pixels.obj_attr.copy_within(1..8, 0);
        self.pixels.obj[7] = 0;
        self.pixels.obj_attr[7] = 0;
        if self.pixels.x < -8 {
            if self.pixels.x & 7 == i32::from(self.scx & 7)
                || self.pixels.window_first && self.pixels.x & 7 == 6 && self.scx & 7 == 7
            {
                self.pixels.x = -8;
            } else if self.pixels.x == -9 {
                self.pixels.x = -16;
                return;
            }
        }
        self.pixels.window_first = false;
        if (0..WIDTH as i32).contains(&self.pixels.x) {
            let bg = if self.lcdc & 1 != 0 { bg } else { 0 };
            let mut shade = (self.bgp >> (bg * 2)) & 3;
            let mut color = bg;
            let mut source = 0;
            if self.lcdc & 2 != 0 && obj != 0 && (attr & 0x80 == 0 || bg == 0) {
                let palette = if attr & 0x10 != 0 { self.obp1 } else { self.obp0 };
                shade = (palette >> (obj * 2)) & 3;
                color = obj;
                source = if attr & 0x10 != 0 { 2 } else { 1 };
            }
            let offset = usize::from(self.line) * WIDTH + self.pixels.x as usize;
            self.framebuffer[offset] = shade;
            self.pixels.output[0] = OutputPixel { offset: offset as u16, color, palette: source };
        }
        self.pixels.x += 1;
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

    /// Per-line timing of the most recent frame: mode 3 length in dots
    /// (bits 0–8), objects on the line (bits 9–12), window active (bit 13).
    pub fn line_timing(&self) -> &[u16; HEIGHT] {
        &self.line_timing
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
        self.pixels.save(w);
        w.u8(self.wy_check_delay);
        w.u8(self.stat_write);
        w.bool(self.window_restart.is_some());
        if let Some(restart) = &self.window_restart {
            restart.pixels.save(w);
            w.u8(restart.window_line);
        }
    }

    pub(crate) fn load(&mut self, r: &mut StateReader) -> Result<(), StateError> {
        r.u8s(&mut self.vram)?;
        r.u8s(&mut self.oam)?;
        let mut b = [0u8; 17];
        r.u8s(&mut b)?;
        [self.lcdc, self.stat_select, self.scy, self.scx, self.lyc, self.bgp, self.obp0, self.obp1, self.wy, self.wx] =
            [b[0], b[1] & 0x78, b[2], b[3], b[4], b[5], b[6], b[7], b[8], b[9]];
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
        let drawing_start = if self.first_line { 79 } else { 84 };
        if self.mode3_end < drawing_start || self.mode3_end >= DOTS_PER_LINE {
            return Err(StateError::Corrupt("ppu mode 3 end"));
        }
        self.off_dots = r.u32()?;
        if self.off_dots >= DOTS_PER_FRAME {
            return Err(StateError::Corrupt("ppu off-dot counter"));
        }
        r.u8s(&mut self.framebuffer)?;
        self.frame_count = r.u64()?;
        if self.frame_count >= 1 << 62 {
            return Err(StateError::Corrupt("frame counter"));
        }
        self.pixels.load(r)?;
        self.wy_check_delay = r.u8()?;
        self.stat_write = r.u8()?;
        if self.stat_write != 0xFF && self.stat_write & !0x78 != 0 {
            return Err(StateError::Corrupt("ppu STAT write mask"));
        }
        self.window_restart = if r.bool()? {
            let mut pixels = PixelPipeline::default();
            pixels.load(r)?;
            let window_line = r.u8()?;
            if self.stat_mode != Mode::Drawing || pixels.x != self.pixels.x || pixels.window {
                return Err(StateError::Corrupt("ppu window restart latch"));
            }
            Some(WindowRestart { pixels, window_line })
        } else {
            None
        };
        if self.wy_check_delay > 4 {
            return Err(StateError::Corrupt("ppu window comparison delay"));
        }
        if self.stat_mode == Mode::Drawing
            && (self.line >= HEIGHT as u8
                || self.dot < drawing_start
                || self.dot >= self.mode3_end
                || self.pixels.x >= WIDTH as i32)
        {
            return Err(StateError::Corrupt("ppu drawing position"));
        }
        self.frame_ready = false;
        self.next_event = self.compute_next_event();
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

#[cfg(test)]
mod tests {
    use super::*;

    fn drawing_ppu() -> Ppu {
        let mut p = Ppu::power_on();
        p.lcdc = 0x91;
        p.bgp = 0xE4;
        p.obp0 = 0xE4;
        p.dot = 83;
        p.next_event = 84;
        for row in 0..8 {
            p.vram[row * 2] = 0xFF; // BG tile 0: colour 1.
            p.vram[16 + row * 2 + 1] = 0xFF; // OBJ/window tile 1: colour 2.
        }
        p
    }

    fn finish_line(p: &mut Ppu) {
        for _ in 0..110 {
            p.tick();
            if p.stat_mode == Mode::HBlank && p.dot > 100 {
                return;
            }
        }
        panic!("pixel transfer did not finish");
    }

    #[test]
    fn palette_write_only_changes_pixels_not_yet_output() {
        let mut p = drawing_ppu();
        while p.pixels.x < 40 {
            p.tick();
        }
        let split = p.pixels.x as usize;
        p.write_register(0xFF47, 0xE8);
        finish_line(&mut p);
        assert!(p.framebuffer[..split - 2].iter().all(|&c| c == 1));
        assert_eq!(p.framebuffer[split - 2], 3); // The collision dot sees old|new.
        assert!(p.framebuffer[split - 1..WIDTH].iter().all(|&c| c == 2));
        assert_eq!(p.mode3_end, 84 + MODE3_BASE);
    }

    #[test]
    fn fetcher_samples_scroll_y_separately_for_each_bitplane() {
        let mut p = drawing_ppu();
        p.pixels.bg_len = 0;
        p.pixels.x = 0;
        p.vram[0] = 0xFF;
        p.vram[1] = 0;
        p.vram[2] = 0;
        p.vram[3] = 0xFF;
        for _ in 0..4 {
            p.fetch_dot();
        }
        p.write_register(0xFF42, 1);
        p.fetch_dot();
        p.fetch_dot();
        assert_eq!(p.pixels.bg, [3; 8]);
    }

    #[test]
    fn scy_write_on_a_bitplane_address_dot_changes_that_read() {
        let mut p = drawing_ppu();
        p.pixels.bg_len = 0;
        p.pixels.x = 0;
        p.vram[0] = 0xFF;
        p.vram[1] = 0;
        p.vram[2] = 0;
        p.vram[3] = 0xFF;
        for _ in 0..3 {
            p.fetch_dot(); // The low-bitplane address has just been sampled.
        }
        p.write_register(0xFF42, 1);
        for _ in 0..3 {
            p.fetch_dot();
        }
        assert_eq!(p.pixels.bg, [2; 8]);
    }

    #[test]
    fn lcdc_write_can_select_a_new_map_and_mix_tile_data_banks() {
        let mut p = drawing_ppu();
        p.pixels.bg_len = 0;
        p.pixels.x = 0;
        p.vram[0x1C01] = 1;
        p.vram[0x1010] = 0xFF; // Signed-address tile 1, low plane.
        p.vram[0x0011] = 0xFF; // Unsigned-address tile 1, high plane.
        p.fetch_dot();
        p.write_register(0xFF40, 0x99); // Map select arrives on the address dot.
        p.fetch_dot();
        p.fetch_dot();
        p.write_register(0xFF40, 0x89); // Low plane uses signed addressing.
        p.fetch_dot();
        p.write_register(0xFF40, 0x99); // Completed low read remains latched.
        p.fetch_dot();
        p.fetch_dot();
        assert_eq!(p.pixels.bg, [3; 8]);
    }

    #[test]
    fn an_odd_fetch_phase_held_during_an_object_stall_does_not_resample() {
        let mut p = drawing_ppu();
        p.lcdc |= 2;
        p.start_drawing(84);
        p.dot = 100;
        p.pixels.startup = 0;
        p.pixels.x = 0;
        p.pixels.phase = 2;
        p.pixels.tile = 0;
        p.fetch_dot(); // Address 0 is latched; phase 3 waits through OBJ fetch.
        p.pixels.objects[0] = LineObject { y: 16, x: 8, tile: 1, attr: 0, index: 0 };
        p.pixels.object_count = 1;
        p.pixels.next_object = 1;
        p.pixels.object_stall = 4;
        p.tick();
        p.write_register(0xFF42, 1);
        p.fetch_dot();
        assert_eq!(p.pixels.low, 0xFF); // Row 0 was already on the address bus.
    }

    #[test]
    fn scx_write_updates_a_map_read_within_its_two_dot_bus_window() {
        let mut p = drawing_ppu();
        p.pixels.bg_len = 0;
        p.pixels.x = 0;
        p.vram[0x1801] = 0;
        p.vram[0x1802] = 1;
        p.fetch_dot(); // Map address at the first dot.
        p.pixels.fetch_latch[1] = p.pixels.fetch_latch[0];
        p.pixels.fetch_latch_x[1] = p.pixels.fetch_latch_x[0];
        p.fetch_dot(); // Tile byte returned on the following dot.
        p.write_register(0xFF43, 8);
        for _ in 0..4 {
            p.fetch_dot();
        }
        assert_eq!(p.pixels.bg, [2; 8]);
    }

    #[test]
    fn object_size_is_sampled_after_waiting_and_for_each_bitplane() {
        let mut p = drawing_ppu();
        p.lcdc |= 2;
        p.pixels.x = 0;
        p.pixels.objects[0] = LineObject { y: 16, x: 8, tile: 1, attr: 0, index: 0 };
        p.pixels.object_count = 1;
        assert!(p.begin_object()); // Started as an 8x8 sprite using tile 1.
        p.pixels.object_waiting = false;
        p.pixels.object_stall = 3;
        p.write_register(0xFF40, 0x97);
        p.object_dot(); // 8x16 selects even tile 0 for the low plane.
        p.object_dot();
        p.write_register(0xFF40, 0x93);
        p.object_dot(); // 8x8 selects tile 1 again for the high plane.
        assert_eq!(p.pixels.obj, [3; 8]);
    }

    #[test]
    fn object_low_read_collision_expires_on_the_following_dot() {
        let mut p = drawing_ppu();
        p.lcdc |= 2;
        p.pixels.objects[0] = LineObject { y: 16, x: 8, tile: 1, attr: 0, index: 0 };
        p.pixels.object_count = 1;
        p.pixels.next_object = 1;
        p.pixels.object_stall = 3;
        p.object_dot();
        assert_eq!(p.pixels.object_low, 0);
        p.write_register(0xFF40, 0x97);
        assert_eq!(p.pixels.object_low, 0xFF);
        p.object_dot(); // The low read is now one dot older.
        p.write_register(0xFF40, 0x93);
        assert_eq!(p.pixels.object_low, 0xFF);
    }

    #[test]
    fn object_high_read_collision_restores_priority_and_survives_save() {
        let mut p = drawing_ppu();
        p.lcdc |= 2;
        p.start_drawing(84);
        p.dot = 100;
        p.pixels.objects[0] = LineObject { y: 16, x: 8, tile: 1, attr: 0, index: 0 };
        p.pixels.object_count = 1;
        p.pixels.next_object = 1;
        p.pixels.object_stall = 1;
        p.pixels.obj[0] = 1;
        p.pixels.obj_attr[0] = 0x80;
        p.object_dot();
        assert_eq!(p.pixels.obj, [1, 2, 2, 2, 2, 2, 2, 2]);
        let mut w = StateWriter::new();
        p.save(&mut w);
        let mut resumed = Ppu::new();
        resumed.load(&mut StateReader::new(&w.finish())).unwrap();
        for machine in [&mut p, &mut resumed] {
            machine.write_register(0xFF40, 0x97); // The new high plane is transparent.
            assert_eq!(machine.pixels.obj, [1, 0, 0, 0, 0, 0, 0, 0]);
            assert_eq!(machine.pixels.obj_attr, [0x80, 0, 0, 0, 0, 0, 0, 0]);
        }
    }

    #[test]
    fn window_restarts_fetcher_and_uses_its_own_tile_row() {
        let mut p = drawing_ppu();
        p.lcdc |= 0x60;
        p.wx = 47;
        p.wy_triggered = true;
        p.vram[0x1C00..0x2000].fill(1);
        finish_line(&mut p);
        assert!(p.framebuffer[..40].iter().all(|&c| c == 1));
        assert!(p.framebuffer[40..WIDTH].iter().all(|&c| c == 2));
        assert_eq!(p.window_line, 1);
        assert_eq!(p.mode3_end, 84 + MODE3_BASE + 6);
    }

    #[test]
    fn simultaneous_window_and_object_wait_for_the_background_refill() {
        let mut p = drawing_ppu();
        p.lcdc |= 0x62;
        p.wx = 47;
        p.wy_triggered = true;
        p.oam[..4].copy_from_slice(&[16, 48, 1, 0]);
        p.vram[0x1C00..0x2000].fill(1);
        finish_line(&mut p);
        assert_eq!(p.mode3_end, 84 + MODE3_BASE + 6 + 11);
        assert!(p.framebuffer[40..WIDTH].iter().all(|&c| c == 2));
    }

    #[test]
    fn disabling_objects_cancels_an_in_flight_fetch() {
        let mut p = drawing_ppu();
        p.lcdc |= 2;
        p.oam[..4].copy_from_slice(&[16, 48, 1, 0]);
        while p.pixels.object_stall == 0 {
            p.tick();
        }
        p.write_register(0xFF40, p.lcdc & !2);
        finish_line(&mut p);
        assert_eq!(p.pixels.object_stall, 0);
        assert!(p.framebuffer[..WIDTH].iter().all(|&c| c == 1));
    }

    #[test]
    fn object_priority_is_decided_before_background_priority() {
        let mut p = drawing_ppu();
        p.lcdc |= 2;
        p.oam[..8].copy_from_slice(&[16, 48, 1, 0x80, 16, 48, 1, 0]);
        finish_line(&mut p);
        // First object wins the OBJ queue, then loses to nonzero BG. The
        // second object cannot show through it just because it is in front.
        assert!(p.framebuffer[..WIDTH].iter().all(|&c| c == 1));
    }

    #[test]
    fn lcdc_write_on_the_window_start_dot_cancels_the_restart() {
        let mut p = drawing_ppu();
        p.lcdc |= 0x20;
        p.scx = 2;
        p.wx = 7;
        p.wy_triggered = true;
        while p.dot < 99 {
            p.tick();
        }
        assert!(p.window_restart.is_some());
        let mut w = StateWriter::new();
        p.save(&mut w);
        let mut resumed = Ppu::new();
        resumed.load(&mut StateReader::new(&w.finish())).unwrap();
        for machine in [&mut p, &mut resumed] {
            machine.write_register(0xFF40, 0x91);
            finish_line(machine);
            assert!(!machine.pixels.window_used);
            assert_eq!(machine.window_line, 0);
            assert_eq!(machine.mode3_end, 84 + MODE3_BASE + 2);
        }
        assert_eq!(p.framebuffer, resumed.framebuffer);
    }

    #[test]
    fn first_line_wy_comparison_waits_for_the_mode_2_clock() {
        for wy in [0, 0xFF] {
            let mut p = Ppu::new();
            p.lcdc |= 0x20;
            p.wy = 0;
            p.dot = 455;
            p.next_event = 456;
            p.tick();
            assert_eq!(p.position(), (0, 3));
            assert!(!p.wy_triggered);
            p.write_register(0xFF4A, wy);
            p.tick();
            assert_eq!(p.position(), (0, 7));
            assert_eq!(p.wy_triggered, wy == 0);
        }
    }

    #[test]
    fn a_late_wy_write_does_not_retroactively_start_the_window() {
        let mut p = drawing_ppu();
        p.lcdc |= 0x20;
        p.wy = 0xFF;
        p.wx = 15;
        while p.pixels.x < 7 {
            p.tick();
        }
        p.write_register(0xFF4A, 0);
        assert!(!p.wy_triggered);
        p.tick();
        assert!(p.wy_triggered);
        finish_line(&mut p);
        assert!(!p.pixels.window_used);
    }

    #[test]
    fn stat_write_conflict_lasts_one_dot_and_respects_the_oam_boundary() {
        let mut p = drawing_ppu();
        p.line = 1;
        p.ly = 0;
        p.lyc = 0xFF;
        p.dot = 3;
        p.stat_mode = Mode::HBlank;
        p.irq_mode = None;
        p.stat_select = 0x08;
        p.write_register(0xFF41, 0x20);
        assert_eq!(p.stat_select, 0x58);
        p.dot = 4;
        assert_eq!(p.dot_event() & IRQ_STAT, 0);
        assert_eq!(p.stat_select, 0x20);
        assert_eq!(p.stat_write, 0xFF);
    }

    #[test]
    fn impossible_drawing_positions_are_rejected_without_panicking() {
        let mut p = drawing_ppu();
        p.stat_mode = Mode::Drawing;
        p.dot = 0;
        p.pixels.startup = 0;
        p.pixels.x = 159;
        let mut w = StateWriter::new();
        p.save(&mut w);
        assert!(Ppu::new().load(&mut StateReader::new(&w.finish())).is_err());
        p.stat_mode = Mode::HBlank;
        p.mode3_end = 79;
        p.dot = 78;
        let mut w = StateWriter::new();
        p.save(&mut w);
        assert!(Ppu::new().load(&mut StateReader::new(&w.finish())).is_err());
    }

    #[test]
    fn pipeline_rejects_invalid_saved_indices() {
        let p = PixelPipeline::default();
        let mut w = StateWriter::new();
        p.save(&mut w);
        let original = w.finish();
        for (offset, bad) in [(24, 8), (25, 9), (26, 7), (33, 11), (34, 1), (38, 0xFF)] {
            let mut bytes = original.clone();
            bytes[offset] = bad;
            assert!(PixelPipeline::default().load(&mut StateReader::new(&bytes)).is_err(), "offset {offset}");
        }
    }

    #[test]
    fn pipeline_rejects_corrupt_fetch_history() {
        for (history, x, phase) in [([0, 4], 0, 0), ([1, 0], 0, 2), ([0, 1], i32::MIN, 2)] {
            let p = PixelPipeline { fetch_latch: history, fetch_latch_x: [x; 2], phase, ..PixelPipeline::default() };
            let mut w = StateWriter::new();
            p.save(&mut w);
            assert!(PixelPipeline::default().load(&mut StateReader::new(&w.finish())).is_err());
        }
    }

    #[test]
    fn pipeline_rejects_corrupt_object_read_history() {
        for bad in 0..5 {
            let mut p = PixelPipeline::default();
            match bad {
                0 => p.object_read = 3,
                1 => p.object_read = 2, // No active object to reread.
                2 => {
                    p.object_read = 1;
                    p.next_object = 1;
                    p.object_count = 1; // Inconsistent remaining fetch dots.
                }
                3 => p.object_prior[0] = 4,
                _ => p.object_prior_attr[0] = 1,
            }
            let mut w = StateWriter::new();
            p.save(&mut w);
            assert!(PixelPipeline::default().load(&mut StateReader::new(&w.finish())).is_err());
        }
    }
}
