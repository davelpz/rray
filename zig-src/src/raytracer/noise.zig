// Classic Perlin noise, compatible with FastNoiseLite Perlin (seed=1337).
// Ported from the FastNoiseLite reference implementation.

const std = @import("std");

var perm: [512]usize = undefined;
var initialized = false;

pub fn initNoise() void {
    if (initialized) return;
    // Build permutation table with seed 1337 (same as Rust fastnoise-lite default)
    var seed: u64 = 1337;
    var p: [256]usize = undefined;
    for (0..256) |i| {
        p[i] = i;
    }
    // Fisher-Yates shuffle seeded
    var i: usize = 255;
    while (i > 0) : (i -= 1) {
        seed = seed *% 6364136223846793005 +% 1442695040888963407;
        const j: usize = @intCast((seed >> 33) % @as(u64, @intCast(i + 1)));
        const tmp = p[i];
        p[i] = p[j];
        p[j] = tmp;
    }
    for (0..256) |idx| {
        perm[idx] = p[idx];
        perm[idx + 256] = p[idx];
    }
    initialized = true;
}

fn fade(t: f64) f64 {
    return t * t * t * (t * (t * 6.0 - 15.0) + 10.0);
}

fn lerp(a: f64, b: f64, t: f64) f64 {
    return a + t * (b - a);
}

fn grad(hash: usize, x: f64, y: f64, z: f64) f64 {
    const h = hash & 15;
    const u: f64 = if (h < 8) x else y;
    const v: f64 = if (h < 4) y else if (h == 12 or h == 14) x else z;
    const sign_u: f64 = if ((h & 1) == 0) u else -u;
    const sign_v: f64 = if ((h & 2) == 0) v else -v;
    return sign_u + sign_v;
}

pub fn getNoise3d(x: f64, y: f64, z: f64) f64 {
    const xi: i32 = @intFromFloat(@floor(x));
    const yi: i32 = @intFromFloat(@floor(y));
    const zi: i32 = @intFromFloat(@floor(z));

    const xf = x - @floor(x);
    const yf = y - @floor(y);
    const zf = z - @floor(z);

    const u = fade(xf);
    const v = fade(yf);
    const w = fade(zf);

    const X: usize = @intCast(xi & 255);
    const Y: usize = @intCast(yi & 255);
    const Z: usize = @intCast(zi & 255);

    const aaa = perm[perm[perm[X] + Y] + Z];
    const aba = perm[perm[perm[X] + Y + 1] + Z];
    const aab = perm[perm[perm[X] + Y] + Z + 1];
    const abb = perm[perm[perm[X] + Y + 1] + Z + 1];
    const baa = perm[perm[perm[X + 1] + Y] + Z];
    const bba = perm[perm[perm[X + 1] + Y + 1] + Z];
    const bab = perm[perm[perm[X + 1] + Y] + Z + 1];
    const bbb = perm[perm[perm[X + 1] + Y + 1] + Z + 1];

    const x1 = lerp(grad(aaa, xf, yf, zf), grad(baa, xf - 1, yf, zf), u);
    const x2 = lerp(grad(aba, xf, yf - 1, zf), grad(bba, xf - 1, yf - 1, zf), u);
    const y1 = lerp(x1, x2, v);

    const x3 = lerp(grad(aab, xf, yf, zf - 1), grad(bab, xf - 1, yf, zf - 1), u);
    const x4 = lerp(grad(abb, xf, yf - 1, zf - 1), grad(bbb, xf - 1, yf - 1, zf - 1), u);
    const y2 = lerp(x3, x4, v);

    return lerp(y1, y2, w);
}

pub fn octavePerlin(x: f64, y: f64, z: f64, octaves: usize, persistence: f64) f64 {
    var total: f64 = 0;
    var freq: f64 = 1;
    var amplitude: f64 = 1;
    var max_val: f64 = 0;
    for (0..octaves) |_| {
        total += getNoise3d(x * freq, y * freq, z * freq) * amplitude;
        max_val += amplitude;
        amplitude *= persistence;
        freq *= 2;
    }
    return total / max_val;
}
