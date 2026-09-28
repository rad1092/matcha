//! C-ABI WebAssembly bindings for matcha-core (ADR-0004).
//!
//! One `.wasm` file serves both the browser player and the Node MCP server.
//! No wasm-bindgen: every export takes/returns plain numbers, and bulk data
//! moves through buffers owned by the emulator handle. `web/matcha.js` wraps
//! this into a friendly class.
//!
//! Safety contract for all `unsafe` exports: `emu` must be a live handle from
//! [`matcha_new`], and `(ptr, len)` pairs must describe memory previously
//! obtained from [`matcha_alloc`] (or otherwise valid for `len` bytes).

use matcha_core::cpu::StepKind;
use matcha_core::{Buttons, GameBoy, HEIGHT, RunEvent, WIDTH, palettes};
use std::fmt::Write as _;
use std::sync::Mutex;

static LAST_ERROR: Mutex<String> = Mutex::new(String::new());

fn set_error(msg: impl Into<String>) {
    if let Ok(mut e) = LAST_ERROR.lock() {
        *e = msg.into();
    }
}

/// Emulator handle plus the output buffers JS reads from.
pub struct Emu {
    gb: GameBoy,
    palette: [u32; 4],
    rgba: Vec<u8>,
    scratch_rgba: Vec<u8>,
    text: String,
    state: Vec<u8>,
    words: [u32; 32],
}

impl Emu {
    fn text(&mut self, s: String) -> u32 {
        self.text = s;
        self.text.len() as u32
    }
}

/// # Safety
/// `ptr` must be valid for reads of `len` bytes.
unsafe fn bytes<'a>(ptr: *const u8, len: u32) -> &'a [u8] {
    if ptr.is_null() || len == 0 {
        return &[];
    }
    // SAFETY: guaranteed by the caller (see module docs).
    unsafe { std::slice::from_raw_parts(ptr, len as usize) }
}

/// # Safety
/// `emu` must be a live handle from `matcha_new`.
unsafe fn emu<'a>(emu: *mut Emu) -> &'a mut Emu {
    // SAFETY: guaranteed by the caller (see module docs).
    unsafe { &mut *emu }
}

// --- memory -------------------------------------------------------------------

/// Allocates `len` bytes for JS to fill (ROMs, states, saves).
#[unsafe(no_mangle)]
pub extern "C" fn matcha_alloc(len: u32) -> *mut u8 {
    let mut buf = vec![0u8; len as usize].into_boxed_slice();
    let ptr = buf.as_mut_ptr();
    std::mem::forget(buf);
    ptr
}

/// Frees memory from [`matcha_alloc`].
///
/// # Safety
/// `(ptr, len)` must come from a single `matcha_alloc(len)` call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn matcha_free(ptr: *mut u8, len: u32) {
    if !ptr.is_null() {
        // SAFETY: reconstructs exactly the boxed slice leaked by matcha_alloc.
        drop(unsafe { Box::from_raw(std::ptr::slice_from_raw_parts_mut(ptr, len as usize)) });
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn matcha_last_error_ptr() -> *const u8 {
    LAST_ERROR.lock().map_or(std::ptr::null(), |e| e.as_ptr())
}

#[unsafe(no_mangle)]
pub extern "C" fn matcha_last_error_len() -> u32 {
    LAST_ERROR.lock().map_or(0, |e| e.len() as u32)
}

// --- lifecycle -------------------------------------------------------------------

/// Creates an emulator from a ROM image. Returns null on error.
///
/// # Safety
/// `(rom, len)` must be readable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn matcha_new(rom: *const u8, len: u32) -> *mut Emu {
    // SAFETY: forwarded caller guarantee.
    let rom = unsafe { bytes(rom, len) }.to_vec();
    match GameBoy::new(rom) {
        Ok(gb) => Box::into_raw(Box::new(Emu {
            gb,
            palette: palettes::MATCHA,
            rgba: vec![0; WIDTH * HEIGHT * 4],
            scratch_rgba: Vec::new(),
            text: String::new(),
            state: Vec::new(),
            words: [0; 32],
        })),
        Err(e) => {
            set_error(e.to_string());
            std::ptr::null_mut()
        }
    }
}

/// # Safety
/// `emu` must be a live handle; it is invalid afterwards.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn matcha_destroy(emu: *mut Emu) {
    if !emu.is_null() {
        // SAFETY: the handle came from Box::into_raw in matcha_new.
        drop(unsafe { Box::from_raw(emu) });
    }
}

/// # Safety
/// See module docs.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn matcha_reset(e: *mut Emu) {
    unsafe { emu(e) }.gb.reset();
}

// --- running ---------------------------------------------------------------------

fn event_code(ev: RunEvent, words: &mut [u32; 32]) -> u32 {
    match ev {
        RunEvent::FrameComplete => 0,
        RunEvent::Breakpoint { pc } => {
            words[0] = u32::from(pc);
            1
        }
        RunEvent::Watchpoint { pc, hit } => {
            words[0] = u32::from(pc);
            words[1] = u32::from(hit.addr);
            words[2] = u32::from(hit.value);
            words[3] = u32::from(hit.write);
            2
        }
        RunEvent::CycleBudget => 3,
    }
}

/// Runs to the end of the frame. Returns 0 = frame, 1 = breakpoint (PC in
/// word 0), 2 = watchpoint (pc, addr, value, write in words 0..4), 3 = budget.
///
/// # Safety
/// See module docs.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn matcha_run_frame(e: *mut Emu) -> u32 {
    let e = unsafe { emu(e) };
    let ev = e.gb.run_frame();
    event_code(ev, &mut e.words)
}

/// Executes one instruction. Returns 0 = instruction, 1 = interrupt,
/// 2 = halted, 3 = stopped, 4 = locked.
///
/// # Safety
/// See module docs.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn matcha_step(e: *mut Emu) -> u32 {
    match unsafe { emu(e) }.gb.step().kind {
        StepKind::Instruction { .. } => 0,
        StepKind::Interrupt { .. } => 1,
        StepKind::Halted => 2,
        StepKind::Stopped => 3,
        StepKind::Locked { .. } => 4,
    }
}

/// Button mask: bit0 right, 1 left, 2 up, 3 down, 4 A, 5 B, 6 select, 7 start.
///
/// # Safety
/// See module docs.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn matcha_set_buttons(e: *mut Emu, mask: u32) {
    unsafe { emu(e) }.gb.set_buttons(Buttons(mask as u8));
}

// --- video -----------------------------------------------------------------------

/// Sets the 4-shade palette (0xRRGGBB, lightest first).
///
/// # Safety
/// See module docs.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn matcha_set_palette(e: *mut Emu, c0: u32, c1: u32, c2: u32, c3: u32) {
    unsafe { emu(e) }.palette = [c0, c1, c2, c3];
}

/// Renders the current frame to RGBA; returns a pointer to 160*144*4 bytes.
///
/// # Safety
/// See module docs.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn matcha_render(e: *mut Emu) -> *const u8 {
    let e = unsafe { emu(e) };
    e.gb.render_rgba(&e.palette, &mut e.rgba);
    e.rgba.as_ptr()
}

/// Renders all 384 VRAM tiles as a 128x192 RGBA sheet (16 tiles per row),
/// shaded with the current palette through BGP. Returns the pixel pointer.
///
/// # Safety
/// See module docs.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn matcha_render_tiles(e: *mut Emu) -> *const u8 {
    let e = unsafe { emu(e) };
    let (w, h) = (128usize, 192usize);
    e.scratch_rgba.resize(w * h * 4, 0);
    let bgp = e.gb.ppu_registers()[6];
    let mut tile = [0u8; 64];
    for t in 0..384 {
        e.gb.decode_tile(t, &mut tile);
        let (tx, ty) = ((t % 16) * 8, (t / 16) * 8);
        for (i, &c) in tile.iter().enumerate() {
            let shade = (bgp >> (c * 2)) & 3;
            let rgb = e.palette[usize::from(shade)];
            let o = ((ty + i / 8) * w + tx + i % 8) * 4;
            e.scratch_rgba[o..o + 4].copy_from_slice(&[(rgb >> 16) as u8, (rgb >> 8) as u8, rgb as u8, 255]);
        }
    }
    e.scratch_rgba.as_ptr()
}

// --- audio -----------------------------------------------------------------------

/// # Safety
/// See module docs.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn matcha_set_sample_rate(e: *mut Emu, hz: u32) {
    unsafe { emu(e) }.gb.set_sample_rate(hz);
}

/// Interleaved stereo f32 samples since the last clear.
///
/// # Safety
/// See module docs.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn matcha_audio_ptr(e: *mut Emu) -> *const f32 {
    unsafe { emu(e) }.gb.audio_samples().as_ptr()
}

/// # Safety
/// See module docs.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn matcha_audio_len(e: *mut Emu) -> u32 {
    unsafe { emu(e) }.gb.audio_samples().len() as u32
}

/// # Safety
/// See module docs.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn matcha_audio_clear(e: *mut Emu) {
    unsafe { emu(e) }.gb.clear_audio();
}

/// Mutes channels: bit n = channel n+1 audible.
///
/// # Safety
/// See module docs.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn matcha_set_channel_mask(e: *mut Emu, mask: u32) {
    unsafe { emu(e) }.gb.set_audio_channel_mask(mask as u8);
}

// --- save states & battery ----------------------------------------------------------

/// Serializes the machine; returns the length (read via `matcha_state_ptr`).
///
/// # Safety
/// See module docs.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn matcha_save_state(e: *mut Emu) -> u32 {
    let e = unsafe { emu(e) };
    e.state = e.gb.save_state();
    e.state.len() as u32
}

/// # Safety
/// See module docs.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn matcha_state_ptr(e: *mut Emu) -> *const u8 {
    unsafe { emu(e) }.state.as_ptr()
}

/// Returns 0 on success; on failure the machine is unchanged and the reason
/// is available through `matcha_last_error_*`.
///
/// # Safety
/// See module docs.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn matcha_load_state(e: *mut Emu, ptr: *const u8, len: u32) -> i32 {
    let e = unsafe { emu(e) };
    match e.gb.load_state(unsafe { bytes(ptr, len) }) {
        Ok(()) => 0,
        Err(err) => {
            set_error(err.to_string());
            -1
        }
    }
}

/// Battery RAM length (0 = no battery).
///
/// # Safety
/// See module docs.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn matcha_battery_len(e: *mut Emu) -> u32 {
    unsafe { emu(e) }.gb.battery_ram().map_or(0, |r| r.len() as u32)
}

/// # Safety
/// See module docs.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn matcha_battery_ptr(e: *mut Emu) -> *const u8 {
    unsafe { emu(e) }.gb.battery_ram().map_or(std::ptr::null(), <[u8]>::as_ptr)
}

/// 1 if battery RAM changed since the last call.
///
/// # Safety
/// See module docs.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn matcha_battery_dirty(e: *mut Emu) -> u32 {
    u32::from(unsafe { emu(e) }.gb.take_battery_dirty())
}

/// # Safety
/// See module docs.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn matcha_load_battery(e: *mut Emu, ptr: *const u8, len: u32) {
    let e = unsafe { emu(e) };
    e.gb.load_battery_ram(unsafe { bytes(ptr, len) });
}

/// # Safety
/// See module docs.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn matcha_rtc_advance(e: *mut Emu, seconds: f64) {
    if seconds.is_finite() && seconds > 0.0 {
        unsafe { emu(e) }.gb.rtc_advance_seconds(seconds as u64);
    }
}

// --- debugging ----------------------------------------------------------------------

/// # Safety
/// See module docs.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn matcha_peek(e: *mut Emu, addr: u32) -> u32 {
    u32::from(unsafe { emu(e) }.gb.peek(addr as u16))
}

/// # Safety
/// See module docs.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn matcha_poke(e: *mut Emu, addr: u32, value: u32) {
    unsafe { emu(e) }.gb.poke(addr as u16, value as u8);
}

/// Copies `len` bytes starting at `addr` (side-effect free) into `out`.
///
/// # Safety
/// `out` must be writable for `len` bytes; see module docs.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn matcha_peek_range(e: *mut Emu, addr: u32, out: *mut u8, len: u32) {
    let e = unsafe { emu(e) };
    if out.is_null() {
        return;
    }
    // SAFETY: caller guarantees `out` is writable for `len` bytes.
    let dst = unsafe { std::slice::from_raw_parts_mut(out, len as usize) };
    for (i, b) in dst.iter_mut().enumerate() {
        *b = e.gb.peek((addr as usize + i) as u16);
    }
}

/// Machine state as 32 words (read via `matcha_words_ptr`):
/// 0 AF, 1 BC, 2 DE, 3 HL, 4 SP, 5 PC, 6 IME, 7 power (0 run,1 halt,2 stop,3 locked),
/// 8 IE, 9 IF, 10-11 cycles lo/hi, 12-13 frames lo/hi, 14 ROM bank,
/// 15-25 LCDC STAT SCY SCX LY LYC BGP OBP0 OBP1 WY WX, 26 PPU line, 27 PPU dot,
/// 28 APU levels (4x (enabled<<4 | level) bytes), 29 buttons.
///
/// # Safety
/// See module docs.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn matcha_snapshot(e: *mut Emu) -> *const u32 {
    let e = unsafe { emu(e) };
    let gb = &e.gb;
    let r = gb.registers();
    let (ie, if_) = gb.interrupt_registers();
    let w = &mut e.words;
    w[0] = u32::from(r.af());
    w[1] = u32::from(r.bc());
    w[2] = u32::from(r.de());
    w[3] = u32::from(r.hl());
    w[4] = u32::from(r.sp);
    w[5] = u32::from(r.pc);
    w[6] = u32::from(gb.ime());
    w[7] = gb.power_state() as u32;
    w[8] = u32::from(ie);
    w[9] = u32::from(if_);
    w[10] = gb.cycles() as u32;
    w[11] = (gb.cycles() >> 32) as u32;
    w[12] = gb.frame_count() as u32;
    w[13] = (gb.frame_count() >> 32) as u32;
    w[14] = gb.cartridge().current_rom_bank() as u32;
    for (i, v) in gb.ppu_registers().iter().enumerate() {
        w[15 + i] = u32::from(*v);
    }
    let (line, dot) = gb.ppu_position();
    w[26] = u32::from(line);
    w[27] = u32::from(dot);
    w[28] = gb
        .audio_channel_levels()
        .iter()
        .enumerate()
        .fold(0, |acc, (i, &(on, lvl))| acc | (u32::from(on) << 4 | u32::from(lvl)) << (i * 8));
    w[29] = u32::from(gb.buttons().0);
    w.as_ptr()
}

/// Pointer to the 32-word scratch area used by `matcha_snapshot` / run events.
///
/// # Safety
/// See module docs.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn matcha_words_ptr(e: *mut Emu) -> *const u32 {
    unsafe { emu(e) }.words.as_ptr()
}

/// Disassembles `count` instructions from `addr` as lines of
/// `addr bytes<TAB>text`; returns the text length (see `matcha_text_ptr`).
///
/// # Safety
/// See module docs.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn matcha_disassemble(e: *mut Emu, addr: u32, count: u32) -> u32 {
    let e = unsafe { emu(e) };
    let mut out = String::new();
    let mut a = addr as u16;
    for _ in 0..count.min(256) {
        let ins = e.gb.disassemble(a);
        let bytes: Vec<String> = ins.bytes[..usize::from(ins.len)].iter().map(|b| format!("{b:02x}")).collect();
        let _ = writeln!(out, "{a:04x} {}\t{}", bytes.join(" "), ins.text);
        a = a.wrapping_add(u16::from(ins.len));
    }
    e.text(out)
}

/// Cartridge header as JSON; returns the text length.
///
/// # Safety
/// See module docs.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn matcha_header_json(e: *mut Emu) -> u32 {
    let e = unsafe { emu(e) };
    let h = e.gb.header().clone();
    let title: String = h.title.chars().filter(|c| *c != '"' && *c != '\\').collect();
    let json = format!(
        concat!(
            "{{\"title\":\"{}\",\"cartType\":{},\"cartTypeName\":\"{}\",\"romSize\":{},",
            "\"ramSize\":{},\"cgbFlag\":{},\"sgbFlag\":{},\"headerChecksumOk\":{},",
            "\"battery\":{},\"rtc\":{}}}"
        ),
        title,
        h.cart_type,
        h.cart_type_name(),
        h.rom_size,
        h.ram_size,
        h.cgb_flag,
        h.sgb_flag,
        h.header_checksum_ok,
        e.gb.cartridge().has_battery(),
        e.gb.cartridge().has_rtc(),
    );
    e.text(json)
}

/// # Safety
/// See module docs.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn matcha_text_ptr(e: *mut Emu) -> *const u8 {
    unsafe { emu(e) }.text.as_ptr()
}

/// # Safety
/// See module docs.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn matcha_breakpoint(e: *mut Emu, addr: u32, add: u32) -> u32 {
    let e = unsafe { emu(e) };
    u32::from(if add != 0 { e.gb.add_breakpoint(addr as u16) } else { e.gb.remove_breakpoint(addr as u16) })
}

/// # Safety
/// See module docs.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn matcha_clear_breakpoints(e: *mut Emu) {
    unsafe { emu(e) }.gb.clear_breakpoints();
}

/// # Safety
/// See module docs.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn matcha_watchpoint(e: *mut Emu, addr: u32, write: u32) -> u32 {
    u32::from(unsafe { emu(e) }.gb.add_watchpoint(addr as u16, write != 0))
}

/// # Safety
/// See module docs.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn matcha_clear_watchpoints(e: *mut Emu) {
    unsafe { emu(e) }.gb.clear_watchpoints();
}

/// Serial port output (what the ROM "printed"); returns the text length.
///
/// # Safety
/// See module docs.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn matcha_serial_text(e: *mut Emu) -> u32 {
    let e = unsafe { emu(e) };
    let s = String::from_utf8_lossy(e.gb.serial_output()).into_owned();
    e.text(s)
}

/// Starts (on != 0) or stops profiling.
///
/// # Safety
/// See module docs.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn matcha_profile(e: *mut Emu, on: u32) {
    let e = unsafe { emu(e) };
    if on != 0 {
        e.gb.enable_profiling();
    } else {
        e.gb.disable_profiling();
    }
}

/// Profile as JSON (utilization, per-opcode counts, interrupts); returns length.
///
/// # Safety
/// See module docs.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn matcha_profile_json(e: *mut Emu) -> u32 {
    let e = unsafe { emu(e) };
    let Some(p) = e.gb.profile() else {
        return e.text("null".into());
    };
    let join = |v: &[u64]| v.iter().map(u64::to_string).collect::<Vec<_>>().join(",");
    let json = format!(
        concat!(
            "{{\"instructions\":{},\"busy\":{},\"halted\":{},\"stopped\":{},\"interrupt\":{},",
            "\"utilization\":{:.6},\"interrupts\":[{}],\"opcodes\":[{}],\"cb\":[{}],\"covered\":{}}}"
        ),
        p.instructions,
        p.busy_cycles,
        p.halted_cycles,
        p.stopped_cycles,
        p.interrupt_cycles,
        p.cpu_utilization(),
        join(&p.interrupts),
        join(&p.opcodes[..]),
        join(&p.cb_opcodes[..]),
        p.covered_bytes(),
    );
    e.text(json)
}

/// Mnemonic for an opcode (prefixed with CB when `cb != 0`), operands shown
/// as zero; returns the text length (see `matcha_text_ptr`).
///
/// # Safety
/// See module docs.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn matcha_opcode_text(e: *mut Emu, op: u32, cb: u32) -> u32 {
    let e = unsafe { emu(e) };
    let bytes = if cb != 0 { [0xCB, op as u8, 0] } else { [op as u8, 0, 0] };
    let text = matcha_core::disasm::decode(0, bytes).text;
    e.text(text)
}

/// 144 u16 values describing the last frame's lines: mode 3 length in dots
/// (bits 0–8), object count (bits 9–12), window active (bit 13).
///
/// # Safety
/// See module docs.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn matcha_line_timing(e: *mut Emu) -> *const u16 {
    unsafe { emu(e) }.gb.ppu_line_timing().as_ptr()
}

/// Removes a read (`write == 0`) or write watchpoint.
///
/// # Safety
/// See module docs.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn matcha_remove_watchpoint(e: *mut Emu, addr: u32, write: u32) -> u32 {
    u32::from(unsafe { emu(e) }.gb.remove_watchpoint(addr as u16, write != 0))
}
