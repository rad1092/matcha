/*
 * Reference runs for the corpus analysis: plays ROMs in SameBoy (DMG-B model,
 * SameBoy's open-source DMG boot ROM) with exactly the input schedule of
 * `matcha profile --input monkey`, and prints one JSON object per ROM with the
 * same screen metrics matcha records. analysis/report.py compares the two.
 *
 * Frame 0 is the moment the boot ROM hands over to the cartridge, which is
 * where matcha (which skips the boot ROM) starts.
 *
 * Power-on RAM: SameBoy normally fills WRAM, HRAM, OAM and wave RAM with
 * time-seeded noise. "zero" starts them at 0 like matcha, isolating
 * emulation differences; "random" uses SameBoy's DMG noise patterns with a
 * fixed seed, to see which ROMs depend on uninitialised memory.
 *
 * Build: analysis/reference/build.sh <sameboy-checkout>
 * Usage: sameboy_profile <dmg_boot.bin> <seconds> zero|random <rom>...
 *        (one JSON object per line on stdout)
 */
#define GB_INTERNAL /* read boot_rom_finished; same layout as the library build */
#include "Core/gb.h"

#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#define W 160
#define H 144
#define SAMPLE_EVERY 15   /* frames between distinct-frame samples */
#define STATIC_RUN 30     /* identical frames in a row that make a "static screen" */
#define MAX_STATIC 64
#define MAX_SAMPLES 4096

static uint32_t pixels[W * H];

static struct {
    uint64_t rng;
    uint8_t held;
    uint64_t frame; /* frames completed since the boot ROM finished */
    uint64_t target;
    bool booted, locked;
    uint64_t lcd_on_frames, nonblank_frames;
    uint64_t samples[MAX_SAMPLES];
    size_t n_samples;
    uint64_t last_hash;
    unsigned run;
    uint64_t statics[MAX_STATIC];
    size_t n_statics;
} S;

static uint64_t fnv1a(const uint8_t *p, size_t n) {
    uint64_t h = 0xcbf29ce484222325ull;
    for (size_t i = 0; i < n; i++) h = (h ^ p[i]) * 0x100000001b3ull;
    return h;
}

static uint64_t xorshift(void) {
    uint64_t x = S.rng;
    x ^= x << 13;
    x ^= x >> 7;
    x ^= x << 17;
    return S.rng = x;
}

/* Mirror of input_for() in crates/matcha-cli/src/profile.rs (monkey mode).
 * Returns -1 to keep the previous buttons. Bit order matches both emulators:
 * right, left, up, down, A, B, select, start. */
static int input_for(uint64_t frame) {
    if (frame < 600) {
        unsigned f = frame % 120;
        if (f >= 60 && f <= 65) return 0x80;
        if (f >= 90 && f <= 95) return 0x10;
        return 0;
    }
    if (frame % 8) return -1;
    uint64_t r = xorshift();
    static const uint8_t dirs[5] = {0x00, 0x01, 0x02, 0x04, 0x08};
    uint8_t b = dirs[r % 5];
    if (r & 0x100) b |= 0x10;
    if (r & 0x200) b |= 0x20;
    if (((r >> 16) & 0x1F) == 0) b |= 0x80;
    return b;
}

static void apply_input(GB_gameboy_t *gb) {
    int b = input_for(S.frame);
    if (b >= 0 && b != S.held) {
        S.held = (uint8_t)b;
        GB_set_key_mask(gb, (GB_key_mask_t)b);
    }
}

/* GB_PALETTE_GREY encodes shade s as red = 0xFF - 0x55 * s. */
static uint32_t rgb_encode(GB_gameboy_t *gb, uint8_t r, uint8_t g, uint8_t b) {
    (void)gb; (void)g; (void)b;
    return r;
}

static void vblank(GB_gameboy_t *gb, GB_vblank_type_t type) {
    (void)type;
    if (!S.booted) return;
    S.frame++;
    uint8_t shades[W * H];
    bool nonblank = false;
    for (size_t i = 0; i < W * H; i++) {
        shades[i] = (uint8_t)(3 - pixels[i] / 0x55);
        nonblank |= shades[i] != shades[0];
    }
    /* Read LCDC directly: memory reads from a vblank callback re-enter the PPU. */
    if (gb->io_registers[GB_IO_LCDC] & 0x80) S.lcd_on_frames++;
    if (nonblank) S.nonblank_frames++;
    uint64_t h = fnv1a(shades, sizeof shades);
    if (S.frame % SAMPLE_EVERY == 0 && S.n_samples < MAX_SAMPLES) S.samples[S.n_samples++] = h;
    if (h == S.last_hash) {
        if (++S.run == STATIC_RUN && S.n_statics < MAX_STATIC) {
            bool seen = false;
            for (size_t i = 0; i < S.n_statics; i++) seen |= S.statics[i] == h;
            if (!seen) S.statics[S.n_statics++] = h;
        }
    } else {
        S.last_hash = h;
        S.run = 1;
    }
    apply_input(gb);
}

static void log_cb(GB_gameboy_t *gb, const char *s, GB_log_attributes_t a) {
    (void)gb; (void)a;
    if (strstr(s, "Illegal Opcode")) S.locked = true;
}

static int cmp_u64(const void *a, const void *b) {
    uint64_t x = *(const uint64_t *)a, y = *(const uint64_t *)b;
    return (x > y) - (x < y);
}

static void json_string(const char *s) {
    putchar('"');
    for (; *s; s++) {
        if (*s == '"' || *s == '\\') putchar('\\');
        putchar(*s);
    }
    putchar('"');
}

static int profile(const char *boot, const char *path, double seconds, bool random_ram) {
    FILE *f = fopen(path, "rb");
    if (!f) return -1;
    static uint8_t rom[8 << 20];
    size_t n = fread(rom, 1, sizeof rom, f);
    fclose(f);

    memset(&S, 0, sizeof S);
    S.rng = fnv1a(rom, n) | 1; /* same seed as matcha */
    S.target = (uint64_t)(seconds * 4194304.0 / 70224.0 + 0.5);

    GB_random_set_enabled(random_ram);
    GB_random_seed(0x6D61746368610000ull); /* fixed: runs are reproducible */
    GB_gameboy_t *gb = GB_alloc();
    GB_init(gb, GB_MODEL_DMG_B);
    if (GB_load_boot_rom(gb, boot)) { fprintf(stderr, "cannot load %s\n", boot); exit(2); }
    GB_set_log_callback(gb, log_cb);
    GB_set_pixels_output(gb, pixels);
    GB_set_rgb_encode_callback(gb, rgb_encode);
    GB_set_palette(gb, &GB_PALETTE_GREY);
    GB_set_vblank_callback(gb, vblank);
    GB_set_emulate_joypad_bouncing(gb, false);
    GB_set_turbo_mode(gb, true, true); /* no real-time throttling, render every frame */
    GB_set_rtc_mode(gb, GB_RTC_MODE_ACCURATE); /* RTC follows emulated time, as in matcha */
    GB_load_rom_from_buffer(gb, rom, n);

    uint64_t boot_cycles = 0, cycles = 0;
    const uint64_t limit = (S.target + 600) * 70224ull * 2; /* 8 MHz units */
    while (S.frame < S.target && cycles < limit) {
        if (!S.booted && gb->boot_rom_finished) {
            S.booted = true;
            boot_cycles = cycles;
            apply_input(gb);
        }
        cycles += GB_run(gb);
    }

    qsort(S.samples, S.n_samples, sizeof S.samples[0], cmp_u64);
    size_t distinct = 0;
    for (size_t i = 0; i < S.n_samples; i++) distinct += i == 0 || S.samples[i] != S.samples[i - 1];

    printf("{\"rom\":");
    json_string(path);
    printf(",\"booted\":%s,\"boot_ms\":%.0f,\"frames\":%llu,\"locked\":%s,\"lcd_on_frames\":%llu,"
           "\"nonblank_frames\":%llu,\"distinct_frames_sampled\":%zu,\"static_screens\":[",
           S.booted ? "true" : "false", boot_cycles / 8388.608, (unsigned long long)S.frame,
           S.locked ? "true" : "false", (unsigned long long)S.lcd_on_frames,
           (unsigned long long)S.nonblank_frames, distinct);
    for (size_t i = 0; i < S.n_statics; i++) printf("%s\"%016llx\"", i ? "," : "", (unsigned long long)S.statics[i]);
    printf("]}\n");
    fflush(stdout);
    GB_free(gb);
    GB_dealloc(gb);
    return 0;
}

int main(int argc, char **argv) {
    if (argc < 5 || (strcmp(argv[3], "zero") && strcmp(argv[3], "random"))) {
        fprintf(stderr, "usage: %s <dmg_boot.bin> <seconds> zero|random <rom>...\n", argv[0]);
        return 2;
    }
    double seconds = atof(argv[2]);
    bool random_ram = !strcmp(argv[3], "random");
    for (int i = 4; i < argc; i++) {
        if (profile(argv[1], argv[i], seconds, random_ram)) fprintf(stderr, "%s: cannot read\n", argv[i]);
    }
    return 0;
}
