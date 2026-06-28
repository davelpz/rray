const std = @import("std");
const Color = @import("../math/color.zig").Color;
const Matrix = @import("../math/matrix.zig").Matrix;
const Tuple = @import("../math/tuple.zig").Tuple;
const noise = @import("noise.zig");
const Texture = @import("texture.zig").Texture;

pub const PatternKind = union(enum) {
    solid: Color,
    stripe: struct { a: *Pattern, b: *Pattern },
    gradient: struct { a: *Pattern, b: *Pattern },
    ring: struct { a: *Pattern, b: *Pattern },
    checker: struct { a: *Pattern, b: *Pattern },
    blend: struct { a: *Pattern, b: *Pattern, factor: f64 },
    perturbed: struct { inner: *Pattern, scale: f64, octaves: usize, persistence: f64 },
    noise_pat: struct { a: *Pattern, b: *Pattern, scale: f64, octaves: usize, persistence: f64 },
    texture: Texture,
    test_pattern,
};

pub const Pattern = struct {
    kind: PatternKind,
    transform: Matrix,
    transform_inv: Matrix,

    pub fn solid(c: Color) Pattern {
        return .{
            .kind = .{ .solid = c },
            .transform = Matrix.identity(),
            .transform_inv = Matrix.identity(),
        };
    }

    pub fn solidWithTransform(c: Color, t: Matrix) Pattern {
        return .{
            .kind = .{ .solid = c },
            .transform = t,
            .transform_inv = t.inverse(),
        };
    }

    pub fn withTransform(self: Pattern, t: Matrix) Pattern {
        var p = self;
        p.transform = t;
        p.transform_inv = t.inverse();
        return p;
    }

    pub fn patternAt(self: *const Pattern, object_pt: Tuple, object_id: usize, db: anytype) Color {
        const pp = self.transform_inv.mulTuple(object_pt);
        return switch (self.kind) {
            .solid => |c| c,
            .test_pattern => Color.init(pp.x, pp.y, pp.z),
            .stripe => |s| {
                const idx: i64 = @intFromFloat(@floor(pp.x));
                if (@mod(idx, 2) == 0) return s.a.patternAt(pp, object_id, db);
                return s.b.patternAt(pp, object_id, db);
            },
            .gradient => |g| {
                const ac = g.a.patternAt(pp, object_id, db);
                const bc = g.b.patternAt(pp, object_id, db);
                const frac = pp.x - @floor(pp.x);
                return ac.add(bc.sub(ac).scale(frac));
            },
            .ring => |r| {
                const dist = @sqrt(pp.x * pp.x + pp.z * pp.z);
                const idx: i64 = @intFromFloat(@floor(dist));
                if (@mod(idx, 2) == 0) return r.a.patternAt(pp, object_id, db);
                return r.b.patternAt(pp, object_id, db);
            },
            .checker => |ck| {
                const sx: i64 = @intFromFloat(@floor(pp.x));
                const sy: i64 = @intFromFloat(@floor(pp.y));
                const sz: i64 = @intFromFloat(@floor(pp.z));
                if (@mod(sx + sy + sz, 2) == 0) return ck.a.patternAt(pp, object_id, db);
                return ck.b.patternAt(pp, object_id, db);
            },
            .blend => |bl| {
                const ac = bl.a.patternAt(pp, object_id, db);
                const bc = bl.b.patternAt(pp, object_id, db);
                return ac.scale(1.0 - bl.factor).add(bc.scale(bl.factor));
            },
            .perturbed => |pt| {
                const nx = noise.octavePerlin(pp.x, pp.y, pp.z, pt.octaves, pt.persistence) * pt.scale;
                const ny = noise.octavePerlin(pp.x, pp.y, pp.z + 1.0, pt.octaves, pt.persistence) * pt.scale;
                const nz = noise.octavePerlin(pp.x, pp.y, pp.z + 2.0, pt.octaves, pt.persistence) * pt.scale;
                const perturbed = Tuple.new(pp.x + nx, pp.y + ny, pp.z + nz, pp.w);
                return pt.inner.patternAt(perturbed, object_id, db);
            },
            .noise_pat => |np| {
                const n = noise.octavePerlin(pp.x, pp.y, pp.z, np.octaves, np.persistence) * np.scale;
                if (n <= 0.0) return np.a.patternAt(pp, object_id, db).scale(-n);
                return np.b.patternAt(pp, object_id, db).scale(n);
            },
            .texture => |*tex| {
                const obj = db.get(object_id);
                const uv = obj.uvMapping(pp);
                return tex.sampleTexture(uv.u, uv.v);
            },
        };
    }
};

pub const Material = struct {
    pattern: Pattern,
    ambient: f64,
    diffuse: f64,
    specular: f64,
    shininess: f64,
    reflective: f64,
    transparency: f64,
    refractive_index: f64,

    pub fn default() Material {
        return .{
            .pattern = Pattern.solid(Color.white()),
            .ambient = 0.1,
            .diffuse = 0.9,
            .specular = 0.9,
            .shininess = 200.0,
            .reflective = 0.0,
            .transparency = 0.0,
            .refractive_index = 1.0,
        };
    }
};
