const std = @import("std");
const Color = @import("../math/color.zig").Color;
const Tuple = @import("../math/tuple.zig").Tuple;
const objectMod = @import("object.zig");
const ObjectDB = objectMod.ObjectDB;

pub const LightKind = union(enum) {
    point,
    area: struct {
        corner: Tuple,
        u: Tuple,
        v: Tuple,
        level: usize,
    },
};

pub const Light = struct {
    kind: LightKind,
    intensity: Color,
    position: Tuple,

    pub fn newPoint(position: Tuple, intensity: Color) Light {
        return .{
            .kind = .point,
            .intensity = intensity,
            .position = position,
        };
    }

    pub fn newArea(corner: Tuple, u: Tuple, v: Tuple, intensity: Color, level: usize) Light {
        const center = corner.add(u.scale(0.5)).add(v.scale(0.5));
        return .{
            .kind = .{ .area = .{ .corner = corner, .u = u, .v = v, .level = level } },
            .intensity = intensity,
            .position = center,
        };
    }

    pub fn samplePoint(self: Light, sample: usize, amount: usize) Tuple {
        return switch (self.kind) {
            .point => self.position,
            .area => |a| {
                const row = sample / amount;
                const col = sample % amount;
                const af = @as(f64, @floatFromInt(amount));
                const u_rand = (@as(f64, @floatFromInt(col)) + randomF64()) / af;
                const v_rand = (@as(f64, @floatFromInt(row)) + randomF64()) / af;
                return a.corner.add(a.u.scale(u_rand)).add(a.v.scale(v_rand));
            },
        };
    }
};

threadlocal var tl_prng: std.Random.DefaultPrng = std.Random.DefaultPrng.init(0);
threadlocal var tl_prng_seeded: bool = false;

fn randomF64() f64 {
    if (!tl_prng_seeded) {
        var seed: u64 = undefined;
        _ = std.os.linux.getrandom(@ptrCast(&seed), @sizeOf(u64), 0);
        tl_prng = std.Random.DefaultPrng.init(seed);
        tl_prng_seeded = true;
    }
    return tl_prng.random().float(f64);
}

pub fn patternAtObject(object_id: usize, world_pt: Tuple, db: *const ObjectDB) Color {
    const object_pt = objectMod.worldToObject(object_id, world_pt, db);
    const obj = db.get(object_id);
    return obj.material.pattern.patternAt(object_pt, object_id, db);
}

pub fn lighting(
    object_id: usize,
    light: Light,
    point: Tuple,
    eyev: Tuple,
    normalv: Tuple,
    in_shadow: f64,
    db: *const ObjectDB,
) Color {
    const obj = db.get(object_id);
    const material = obj.material;
    const color = patternAtObject(object_id, point, db);
    const effective_color = color.hadamard(light.intensity);
    const lightv = light.position.sub(point).normalize();
    const ambient = effective_color.scale(material.ambient);
    const light_dot_normal = lightv.dot(normalv);

    var diffuse = Color.black();
    var specular = Color.black();
    if (light_dot_normal >= 0.0) {
        diffuse = effective_color.scale(material.diffuse * light_dot_normal);
        const reflectv = lightv.negate().reflect(normalv);
        const reflect_dot_eye = reflectv.dot(eyev);
        if (reflect_dot_eye > 0.0) {
            const factor = std.math.pow(f64, reflect_dot_eye, material.shininess);
            specular = light.intensity.scale(material.specular * factor);
        }
    }

    const diffuse_specular = diffuse.add(specular).scale(1.0 - in_shadow);
    return ambient.add(diffuse_specular);
}
