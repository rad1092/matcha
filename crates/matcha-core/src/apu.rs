//! The APU: two pulse channels (one with sweep), a wave channel and a noise
//! channel, mixed to stereo and resampled for the host (ADR-0005).
//!
//! Channel timers run at their hardware rates (the wave channel at 2 T-cycle
//! resolution); the frame sequencer is driven by the DIV-APU event from the
//! timer. Output is produced once per M-cycle, box-filtered down to the host
//! sample rate, then passed through the DMG's DC-blocking high-pass filter.

use crate::state::{StateError, StateReader, StateWriter};
use alloc::vec::Vec;

/// M-cycles per second.
const MCYCLE_HZ: u32 = 1_048_576;
/// Largest amount of audio kept if the host never drains (≈1 s at 48 kHz).
const MAX_BUFFERED_FRAMES: usize = 48_000;

const DUTY: [u8; 4] = [0b0000_0001, 0b1000_0001, 0b1000_0111, 0b0111_1110];

/// Value OR'd into reads of FF10..FF2F (unused/write-only bits read as 1).
const READ_MASK: [u8; 0x20] = [
    0x80, 0x3F, 0x00, 0xFF, 0xBF, // NR10-NR14
    0xFF, 0x3F, 0x00, 0xFF, 0xBF, // --, NR21-NR24
    0x7F, 0xFF, 0x9F, 0xFF, 0xBF, // NR30-NR34
    0xFF, 0xFF, 0x00, 0x00, 0xBF, // --, NR41-NR44
    0x00, 0x00, 0x70, // NR50-NR52
    0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, // FF27-FF2F
];

#[derive(Clone, Debug, Default)]
struct Length {
    enabled: bool,
    counter: u16,
}

impl Length {
    /// Returns true if the channel should be disabled.
    fn clock(&mut self) -> bool {
        if self.enabled && self.counter > 0 {
            self.counter -= 1;
            return self.counter == 0;
        }
        false
    }
}

#[derive(Clone, Debug, Default)]
struct Envelope {
    initial: u8,
    increase: bool,
    period: u8,
    volume: u8,
    timer: u8,
}

impl Envelope {
    fn write(&mut self, v: u8) {
        self.initial = v >> 4;
        self.increase = v & 0x08 != 0;
        self.period = v & 0x07;
    }
    fn read(&self) -> u8 {
        self.initial << 4 | u8::from(self.increase) << 3 | self.period
    }
    fn dac_on(&self) -> bool {
        self.initial != 0 || self.increase
    }
    fn trigger(&mut self, envelope_next: bool) {
        self.volume = self.initial;
        self.timer = if self.period == 0 { 8 } else { self.period };
        if envelope_next {
            self.timer += 1;
        }
    }
    fn clock(&mut self) {
        if self.period == 0 {
            return;
        }
        self.timer = self.timer.saturating_sub(1);
        if self.timer == 0 {
            self.timer = self.period;
            if self.increase && self.volume < 15 {
                self.volume += 1;
            } else if !self.increase && self.volume > 0 {
                self.volume -= 1;
            }
        }
    }
}

#[derive(Clone, Debug, Default)]
struct Pulse {
    enabled: bool,
    duty: u8,
    duty_step: u8,
    freq: u16,
    /// M-cycles until the next duty step.
    timer: u16,
    length: Length,
    env: Envelope,
    // Sweep (channel 1 only).
    sweep_period: u8,
    sweep_negate: bool,
    sweep_shift: u8,
    sweep_timer: u8,
    sweep_enabled: bool,
    sweep_shadow: u16,
    /// A subtraction was computed since the last trigger (NR10 quirk).
    sweep_negated_once: bool,
}

impl Pulse {
    #[inline]
    fn tick(&mut self) {
        if self.timer <= 1 {
            self.timer = 2048 - self.freq;
            self.duty_step = (self.duty_step + 1) & 7;
        } else {
            self.timer -= 1;
        }
    }

    #[inline]
    fn output(&self) -> u8 {
        if self.enabled && DUTY[usize::from(self.duty)] >> (7 - self.duty_step) & 1 != 0 {
            self.env.volume
        } else {
            0
        }
    }

    fn sweep_calc(&mut self) -> u16 {
        let delta = self.sweep_shadow >> self.sweep_shift;
        if self.sweep_negate {
            self.sweep_negated_once = true;
            self.sweep_shadow.wrapping_sub(delta)
        } else {
            self.sweep_shadow + delta
        }
    }

    fn clock_sweep(&mut self) {
        self.sweep_timer = self.sweep_timer.saturating_sub(1);
        if self.sweep_timer != 0 {
            return;
        }
        self.sweep_timer = if self.sweep_period == 0 { 8 } else { self.sweep_period };
        if !self.sweep_enabled || self.sweep_period == 0 {
            return;
        }
        let new = self.sweep_calc();
        if new > 2047 {
            self.enabled = false;
        } else if self.sweep_shift != 0 {
            self.sweep_shadow = new;
            self.freq = new;
            if self.sweep_calc() > 2047 {
                self.enabled = false;
            }
        }
    }
}

#[derive(Clone, Debug, Default)]
struct Wave {
    enabled: bool,
    dac: bool,
    level: u8,
    freq: u16,
    /// Timer in units of 2 T-cycles.
    timer: u16,
    position: u8,
    sample: u8,
    length: Length,
    /// The channel read wave RAM during the last M-cycle (DMG access window).
    just_read: bool,
    ram: [u8; 16],
}

impl Wave {
    #[inline]
    fn tick(&mut self) {
        self.just_read = false;
        if !self.enabled {
            return;
        }
        for _ in 0..2 {
            if self.timer <= 1 {
                self.timer = 2048 - self.freq;
                self.position = (self.position + 1) & 31;
                self.sample = self.ram[usize::from(self.position >> 1)];
                self.just_read = true;
            } else {
                self.timer -= 1;
            }
        }
    }

    #[inline]
    fn output(&self) -> u8 {
        if !self.enabled {
            return 0;
        }
        let nibble = if self.position & 1 == 0 { self.sample >> 4 } else { self.sample & 0x0F };
        match self.level {
            0 => 0,
            n => nibble >> (n - 1),
        }
    }
}

#[derive(Clone, Debug, Default)]
struct Noise {
    enabled: bool,
    nr43: u8,
    lfsr: u16,
    timer: u32,
    length: Length,
    env: Envelope,
}

impl Noise {
    fn period(&self) -> u32 {
        let divisor: u32 = match self.nr43 & 7 {
            0 => 2,
            n => u32::from(n) * 4,
        };
        divisor << (self.nr43 >> 4)
    }

    #[inline]
    fn tick(&mut self) {
        if self.timer <= 1 {
            self.timer = self.period();
            if self.nr43 >> 4 >= 14 {
                return; // shifts 14/15 starve the LFSR of clocks
            }
            let bit = (self.lfsr ^ (self.lfsr >> 1)) & 1;
            self.lfsr = (self.lfsr >> 1) | (bit << 14);
            if self.nr43 & 0x08 != 0 {
                self.lfsr = (self.lfsr & !(1 << 6)) | (bit << 6);
            }
        } else {
            self.timer -= 1;
        }
    }

    #[inline]
    fn output(&self) -> u8 {
        if self.enabled && self.lfsr & 1 == 0 { self.env.volume } else { 0 }
    }
}

pub struct Apu {
    powered: bool,
    nr50: u8,
    nr51: u8,
    ch1: Pulse,
    ch2: Pulse,
    ch3: Wave,
    ch4: Noise,
    /// Index of the next frame-sequencer step (0..8).
    frame_step: u8,

    // Output stage.
    sample_rate: u32,
    phase: u32,
    acc_l: f32,
    acc_r: f32,
    acc_n: u32,
    hpf_l: f32,
    hpf_r: f32,
    hpf_factor: f32,
    buffer: Vec<f32>,
    /// Per-channel mute mask for debugging/visualisation (bit n = channel n+1).
    pub(crate) channel_mask: u8,
}

impl core::fmt::Debug for Apu {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Apu")
            .field("powered", &self.powered)
            .field("channels", &self.channel_levels())
            .field("frame_step", &self.frame_step)
            .finish_non_exhaustive()
    }
}

impl Apu {
    /// State after the DMG boot ROM ("ba-ding" has just played on channel 1).
    pub fn new() -> Self {
        let mut apu = Self::power_on();
        for (i, v) in [0x80u8, 0xBF, 0xF3, 0xFF, 0xBF, 0xFF, 0x3F, 0x00, 0xFF, 0xBF, 0x7F, 0xFF, 0x9F, 0xFF, 0xBF, 0xFF, 0xFF, 0x00, 0x00, 0xBF, 0x77, 0xF3]
            .into_iter()
            .enumerate()
        {
            let addr = 0xFF10 + i as u16;
            if addr != 0xFF14 && addr != 0xFF19 && addr != 0xFF1E && addr != 0xFF23 {
                apu.write(addr, v);
            }
        }
        // Channel 1 is still enabled (the boot chime), with its envelope decayed.
        apu.ch1.enabled = true;
        apu.ch1.env.volume = 0;
        apu
    }

    pub fn power_on() -> Self {
        Self {
            powered: true,
            nr50: 0,
            nr51: 0,
            ch1: Pulse::default(),
            ch2: Pulse::default(),
            ch3: Wave::default(),
            ch4: Noise::default(),
            frame_step: 0,
            sample_rate: 48_000,
            phase: 0,
            acc_l: 0.0,
            acc_r: 0.0,
            acc_n: 0,
            hpf_l: 0.0,
            hpf_r: 0.0,
            hpf_factor: hpf_factor(48_000),
            buffer: Vec::new(),
            channel_mask: 0x0F,
        }
    }

    pub fn set_sample_rate(&mut self, rate: u32) {
        let rate = rate.clamp(8_000, 192_000);
        self.sample_rate = rate;
        self.hpf_factor = hpf_factor(rate);
        self.phase = 0;
    }

    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    /// Interleaved stereo samples produced since the last drain.
    pub fn samples(&self) -> &[f32] {
        &self.buffer
    }

    pub fn clear_samples(&mut self) {
        self.buffer.clear();
    }

    /// Whether the next frame-sequencer step clocks length counters.
    #[inline]
    fn next_step_clocks_length(&self) -> bool {
        self.frame_step & 1 == 0
    }

    /// DIV-APU event: advance the frame sequencer.
    pub fn frame_sequencer(&mut self) {
        if !self.powered {
            return;
        }
        let step = self.frame_step;
        self.frame_step = (step + 1) & 7;
        if step & 1 == 0 {
            if self.ch1.length.clock() {
                self.ch1.enabled = false;
            }
            if self.ch2.length.clock() {
                self.ch2.enabled = false;
            }
            if self.ch3.length.clock() {
                self.ch3.enabled = false;
            }
            if self.ch4.length.clock() {
                self.ch4.enabled = false;
            }
        }
        if step == 2 || step == 6 {
            self.ch1.clock_sweep();
        }
        if step == 7 {
            self.ch1.env.clock();
            self.ch2.env.clock();
            self.ch4.env.clock();
        }
    }

    /// Advances one M-cycle and produces output.
    #[inline]
    pub fn tick(&mut self) {
        if self.powered {
            self.ch1.tick();
            self.ch2.tick();
            self.ch3.tick();
            self.ch4.tick();
        }
        self.mix();
    }

    #[inline]
    fn mix(&mut self) {
        let mut l = 0.0f32;
        let mut r = 0.0f32;
        let mut any_dac = false;
        if self.powered {
            let outs = [
                (self.ch1.env.dac_on(), self.ch1.output()),
                (self.ch2.env.dac_on(), self.ch2.output()),
                (self.ch3.dac, self.ch3.output()),
                (self.ch4.env.dac_on(), self.ch4.output()),
            ];
            for (i, &(dac, digital)) in outs.iter().enumerate() {
                if !dac {
                    continue;
                }
                any_dac = true;
                if self.channel_mask & (1 << i) == 0 {
                    continue;
                }
                // DAC: digital 0..15 -> analog +1..-1 (the slope is negative on hardware).
                let analog = 1.0 - f32::from(digital) / 7.5;
                if self.nr51 & (0x10 << i) != 0 {
                    l += analog;
                }
                if self.nr51 & (0x01 << i) != 0 {
                    r += analog;
                }
            }
            l *= f32::from((self.nr50 >> 4) & 7) + 1.0;
            r *= f32::from(self.nr50 & 7) + 1.0;
        }
        // Normalise: 4 channels * volume 8 -> +-32; keep headroom.
        const SCALE: f32 = 1.0 / 32.0;
        if any_dac {
            self.acc_l += l * SCALE;
            self.acc_r += r * SCALE;
        }
        self.acc_n += 1;
        self.phase += self.sample_rate;
        if self.phase >= MCYCLE_HZ {
            self.phase -= MCYCLE_HZ;
            let n = self.acc_n as f32;
            let (in_l, in_r) = (self.acc_l / n, self.acc_r / n);
            self.acc_l = 0.0;
            self.acc_r = 0.0;
            self.acc_n = 0;
            let out_l = in_l - self.hpf_l;
            self.hpf_l = in_l - out_l * self.hpf_factor;
            let out_r = in_r - self.hpf_r;
            self.hpf_r = in_r - out_r * self.hpf_factor;
            if self.buffer.len() < MAX_BUFFERED_FRAMES * 2 {
                self.buffer.push(out_l);
                self.buffer.push(out_r);
            }
        }
    }

    pub fn read(&self, addr: u16) -> u8 {
        match addr {
            0xFF30..=0xFF3F => {
                if self.ch3.enabled {
                    // DMG: only readable in the cycle the channel itself reads.
                    if self.ch3.just_read { self.ch3.ram[usize::from(self.ch3.position >> 1)] } else { 0xFF }
                } else {
                    self.ch3.ram[usize::from(addr - 0xFF30)]
                }
            }
            0xFF10..=0xFF2F => {
                let mask = READ_MASK[usize::from(addr - 0xFF10)];
                let v = match addr {
                    0xFF10 => self.ch1.sweep_period << 4 | u8::from(self.ch1.sweep_negate) << 3 | self.ch1.sweep_shift,
                    0xFF11 => self.ch1.duty << 6,
                    0xFF12 => self.ch1.env.read(),
                    0xFF14 => u8::from(self.ch1.length.enabled) << 6,
                    0xFF16 => self.ch2.duty << 6,
                    0xFF17 => self.ch2.env.read(),
                    0xFF19 => u8::from(self.ch2.length.enabled) << 6,
                    0xFF1A => u8::from(self.ch3.dac) << 7,
                    0xFF1C => self.ch3.level << 5,
                    0xFF1E => u8::from(self.ch3.length.enabled) << 6,
                    0xFF21 => self.ch4.env.read(),
                    0xFF22 => self.ch4.nr43,
                    0xFF23 => u8::from(self.ch4.length.enabled) << 6,
                    0xFF24 => self.nr50,
                    0xFF25 => self.nr51,
                    0xFF26 => {
                        u8::from(self.powered) << 7
                            | u8::from(self.ch4.enabled) << 3
                            | u8::from(self.ch3.enabled) << 2
                            | u8::from(self.ch2.enabled) << 1
                            | u8::from(self.ch1.enabled)
                    }
                    _ => 0,
                };
                v | mask
            }
            _ => 0xFF,
        }
    }

    pub fn write(&mut self, addr: u16, value: u8) {
        if let 0xFF30..=0xFF3F = addr {
            if self.ch3.enabled {
                if self.ch3.just_read {
                    self.ch3.ram[usize::from(self.ch3.position >> 1)] = value;
                }
            } else {
                self.ch3.ram[usize::from(addr - 0xFF30)] = value;
            }
            return;
        }
        if addr == 0xFF26 {
            let on = value & 0x80 != 0;
            if self.powered && !on {
                self.power_off();
            } else if !self.powered && on {
                self.powered = true;
                self.frame_step = 0;
            }
            return;
        }
        if !self.powered {
            // DMG: length counters stay writable while powered off.
            match addr {
                0xFF11 => self.ch1.length.counter = 64 - u16::from(value & 0x3F),
                0xFF16 => self.ch2.length.counter = 64 - u16::from(value & 0x3F),
                0xFF1B => self.ch3.length.counter = 256 - u16::from(value),
                0xFF20 => self.ch4.length.counter = 64 - u16::from(value & 0x3F),
                _ => {}
            }
            return;
        }
        let length_next = self.next_step_clocks_length();
        let envelope_next = self.frame_step == 7;
        match addr {
            0xFF10 => {
                let ch = &mut self.ch1;
                let was_negate = ch.sweep_negate;
                ch.sweep_period = (value >> 4) & 7;
                ch.sweep_negate = value & 0x08 != 0;
                ch.sweep_shift = value & 7;
                if was_negate && !ch.sweep_negate && ch.sweep_negated_once {
                    ch.enabled = false;
                }
            }
            0xFF11 => {
                self.ch1.duty = value >> 6;
                self.ch1.length.counter = 64 - u16::from(value & 0x3F);
            }
            0xFF12 => {
                self.ch1.env.write(value);
                if !self.ch1.env.dac_on() {
                    self.ch1.enabled = false;
                }
            }
            0xFF13 => self.ch1.freq = (self.ch1.freq & 0x700) | u16::from(value),
            0xFF14 => {
                self.ch1.freq = (self.ch1.freq & 0xFF) | (u16::from(value & 7) << 8);
                let ch = &mut self.ch1;
                write_length_enable(&mut ch.length, &mut ch.enabled, value, length_next);
                if value & 0x80 != 0 {
                    trigger_pulse(ch, length_next, envelope_next);
                    // Sweep.
                    ch.sweep_shadow = ch.freq;
                    ch.sweep_timer = if ch.sweep_period == 0 { 8 } else { ch.sweep_period };
                    ch.sweep_enabled = ch.sweep_period != 0 || ch.sweep_shift != 0;
                    ch.sweep_negated_once = false;
                    if ch.sweep_shift != 0 && ch.sweep_calc() > 2047 {
                        ch.enabled = false;
                    }
                }
            }
            0xFF16 => {
                self.ch2.duty = value >> 6;
                self.ch2.length.counter = 64 - u16::from(value & 0x3F);
            }
            0xFF17 => {
                self.ch2.env.write(value);
                if !self.ch2.env.dac_on() {
                    self.ch2.enabled = false;
                }
            }
            0xFF18 => self.ch2.freq = (self.ch2.freq & 0x700) | u16::from(value),
            0xFF19 => {
                self.ch2.freq = (self.ch2.freq & 0xFF) | (u16::from(value & 7) << 8);
                let ch = &mut self.ch2;
                write_length_enable(&mut ch.length, &mut ch.enabled, value, length_next);
                if value & 0x80 != 0 {
                    trigger_pulse(ch, length_next, envelope_next);
                }
            }
            0xFF1A => {
                self.ch3.dac = value & 0x80 != 0;
                if !self.ch3.dac {
                    self.ch3.enabled = false;
                }
            }
            0xFF1B => self.ch3.length.counter = 256 - u16::from(value),
            0xFF1C => self.ch3.level = (value >> 5) & 3,
            0xFF1D => self.ch3.freq = (self.ch3.freq & 0x700) | u16::from(value),
            0xFF1E => {
                self.ch3.freq = (self.ch3.freq & 0xFF) | (u16::from(value & 7) << 8);
                let ch = &mut self.ch3;
                write_length_enable(&mut ch.length, &mut ch.enabled, value, length_next);
                if value & 0x80 != 0 {
                    // DMG: retriggering while the channel is about to read
                    // corrupts the first bytes of wave RAM.
                    if ch.enabled && ch.timer == 1 {
                        let pos = usize::from(((ch.position + 1) & 31) >> 1);
                        if pos < 4 {
                            ch.ram[0] = ch.ram[pos];
                        } else {
                            let base = pos & !3;
                            ch.ram.copy_within(base..base + 4, 0);
                        }
                    }
                    ch.enabled = ch.dac;
                    if ch.length.counter == 0 {
                        ch.length.counter = if ch.length.enabled && !length_next { 255 } else { 256 };
                    }
                    ch.timer = 2048 - ch.freq + 3;
                    ch.position = 0;
                }
            }
            0xFF20 => self.ch4.length.counter = 64 - u16::from(value & 0x3F),
            0xFF21 => {
                self.ch4.env.write(value);
                if !self.ch4.env.dac_on() {
                    self.ch4.enabled = false;
                }
            }
            0xFF22 => self.ch4.nr43 = value,
            0xFF23 => {
                let ch = &mut self.ch4;
                write_length_enable(&mut ch.length, &mut ch.enabled, value, length_next);
                if value & 0x80 != 0 {
                    ch.enabled = ch.env.dac_on();
                    if ch.length.counter == 0 {
                        ch.length.counter = if ch.length.enabled && !length_next { 63 } else { 64 };
                    }
                    ch.env.trigger(envelope_next);
                    ch.timer = ch.period();
                    ch.lfsr = 0x7FFF;
                }
            }
            0xFF24 => self.nr50 = value,
            0xFF25 => self.nr51 = value,
            _ => {}
        }
    }

    fn power_off(&mut self) {
        // Registers clear; DMG keeps length counters and wave RAM.
        let lengths = [self.ch1.length.counter, self.ch2.length.counter, self.ch3.length.counter, self.ch4.length.counter];
        let ram = self.ch3.ram;
        self.ch1 = Pulse::default();
        self.ch2 = Pulse::default();
        self.ch3 = Wave { ram, ..Wave::default() };
        self.ch4 = Noise::default();
        self.ch1.length.counter = lengths[0];
        self.ch2.length.counter = lengths[1];
        self.ch3.length.counter = lengths[2];
        self.ch4.length.counter = lengths[3];
        self.nr50 = 0;
        self.nr51 = 0;
        self.powered = false;
    }

    /// Channel status for visualisers: (enabled, digital output 0..15) x4.
    pub fn channel_levels(&self) -> [(bool, u8); 4] {
        [
            (self.ch1.enabled, self.ch1.output()),
            (self.ch2.enabled, self.ch2.output()),
            (self.ch3.enabled, self.ch3.output()),
            (self.ch4.enabled, self.ch4.output()),
        ]
    }

    pub(crate) fn save(&self, w: &mut StateWriter) {
        w.bool(self.powered);
        w.u8s(&[self.nr50, self.nr51, self.frame_step]);
        for ch in [&self.ch1, &self.ch2] {
            save_pulse(ch, w);
        }
        let c3 = &self.ch3;
        w.bool(c3.enabled);
        w.bool(c3.dac);
        w.u8s(&[c3.level, c3.position, c3.sample]);
        w.u16(c3.freq);
        w.u16(c3.timer);
        save_length(&c3.length, w);
        w.bool(c3.just_read);
        w.u8s(&c3.ram);
        let c4 = &self.ch4;
        w.bool(c4.enabled);
        w.u8(c4.nr43);
        w.u16(c4.lfsr);
        w.u32(c4.timer);
        save_length(&c4.length, w);
        save_envelope(&c4.env, w);
        w.u32(self.phase);
        w.f32(self.acc_l);
        w.f32(self.acc_r);
        w.u32(self.acc_n);
        w.f32(self.hpf_l);
        w.f32(self.hpf_r);
    }

    pub(crate) fn load(&mut self, r: &mut StateReader) -> Result<(), StateError> {
        self.powered = r.bool()?;
        let mut b = [0u8; 3];
        r.u8s(&mut b)?;
        [self.nr50, self.nr51, self.frame_step] = [b[0], b[1], b[2] & 7];
        load_pulse(&mut self.ch1, r)?;
        load_pulse(&mut self.ch2, r)?;
        let c3 = &mut self.ch3;
        c3.enabled = r.bool()?;
        c3.dac = r.bool()?;
        let mut b = [0u8; 3];
        r.u8s(&mut b)?;
        [c3.level, c3.position, c3.sample] = [b[0] & 3, b[1] & 31, b[2]];
        c3.freq = r.u16()? & 0x7FF;
        c3.timer = r.u16()?;
        load_length(&mut c3.length, r)?;
        c3.just_read = r.bool()?;
        r.u8s(&mut c3.ram)?;
        let c4 = &mut self.ch4;
        c4.enabled = r.bool()?;
        c4.nr43 = r.u8()?;
        c4.lfsr = r.u16()? & 0x7FFF;
        c4.timer = r.u32()?;
        load_length(&mut c4.length, r)?;
        load_envelope(&mut c4.env, r)?;
        self.phase = r.u32()? % MCYCLE_HZ;
        self.acc_l = r.f32()?;
        self.acc_r = r.f32()?;
        self.acc_n = r.u32()?;
        self.hpf_l = r.f32()?;
        self.hpf_r = r.f32()?;
        self.buffer.clear();
        Ok(())
    }
}

impl Default for Apu {
    fn default() -> Self {
        Self::new()
    }
}

fn hpf_factor(rate: u32) -> f32 {
    // 0.999958^(4194304 / rate), via exp/ln-free repeated squaring.
    powf(0.999_958, 4_194_304.0 / rate as f32)
}

/// `base^exp` for 0 < base < 1 without libm (core has no powf).
fn powf(base: f32, exp: f32) -> f32 {
    let whole = exp as u32;
    let frac = exp - whole as f32;
    let mut result = 1.0f32;
    let mut b = base;
    let mut e = whole;
    while e > 0 {
        if e & 1 == 1 {
            result *= b;
        }
        b *= b;
        e >>= 1;
    }
    // Linear interpolation for the fractional part is plenty for a filter
    // coefficient this close to 1.
    result * (1.0 - frac * (1.0 - base))
}

fn write_length_enable(length: &mut Length, enabled: &mut bool, value: u8, length_next: bool) {
    let was = length.enabled;
    length.enabled = value & 0x40 != 0;
    // Extra length clock when enabling length in the first half of a period.
    if !was && length.enabled && !length_next && length.counter > 0 {
        length.counter -= 1;
        if length.counter == 0 && value & 0x80 == 0 {
            *enabled = false;
        }
    }
}

fn trigger_pulse(ch: &mut Pulse, length_next: bool, envelope_next: bool) {
    ch.enabled = ch.env.dac_on();
    if ch.length.counter == 0 {
        ch.length.counter = if ch.length.enabled && !length_next { 63 } else { 64 };
    }
    ch.timer = 2048 - ch.freq;
    ch.env.trigger(envelope_next);
}

fn save_length(l: &Length, w: &mut StateWriter) {
    w.bool(l.enabled);
    w.u16(l.counter);
}

fn load_length(l: &mut Length, r: &mut StateReader) -> Result<(), StateError> {
    l.enabled = r.bool()?;
    l.counter = r.u16()?.min(256);
    Ok(())
}

fn save_envelope(e: &Envelope, w: &mut StateWriter) {
    w.u8s(&[e.initial, u8::from(e.increase), e.period, e.volume, e.timer]);
}

fn load_envelope(e: &mut Envelope, r: &mut StateReader) -> Result<(), StateError> {
    let mut b = [0u8; 5];
    r.u8s(&mut b)?;
    e.initial = b[0] & 0x0F;
    e.increase = b[1] != 0;
    e.period = b[2] & 7;
    e.volume = b[3] & 0x0F;
    e.timer = b[4];
    Ok(())
}

fn save_pulse(ch: &Pulse, w: &mut StateWriter) {
    w.bool(ch.enabled);
    w.u8s(&[ch.duty, ch.duty_step]);
    w.u16(ch.freq);
    w.u16(ch.timer);
    save_length(&ch.length, w);
    save_envelope(&ch.env, w);
    w.u8s(&[ch.sweep_period, u8::from(ch.sweep_negate), ch.sweep_shift, ch.sweep_timer]);
    w.bool(ch.sweep_enabled);
    w.u16(ch.sweep_shadow);
    w.bool(ch.sweep_negated_once);
}

fn load_pulse(ch: &mut Pulse, r: &mut StateReader) -> Result<(), StateError> {
    ch.enabled = r.bool()?;
    let mut b = [0u8; 2];
    r.u8s(&mut b)?;
    ch.duty = b[0] & 3;
    ch.duty_step = b[1] & 7;
    ch.freq = r.u16()? & 0x7FF;
    ch.timer = r.u16()?;
    load_length(&mut ch.length, r)?;
    load_envelope(&mut ch.env, r)?;
    let mut s = [0u8; 4];
    r.u8s(&mut s)?;
    ch.sweep_period = s[0] & 7;
    ch.sweep_negate = s[1] != 0;
    ch.sweep_shift = s[2] & 7;
    ch.sweep_timer = s[3];
    ch.sweep_enabled = r.bool()?;
    ch.sweep_shadow = r.u16()?;
    ch.sweep_negated_once = r.bool()?;
    Ok(())
}
