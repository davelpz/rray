const std = @import("std");
const EPSILON = @import("../main.zig").EPSILON;

pub const Tuple = struct {
    x: f64,
    y: f64,
    z: f64,
    w: f64,

    pub fn point(x: f64, y: f64, z: f64) Tuple {
        return .{ .x = x, .y = y, .z = z, .w = 1.0 };
    }

    pub fn vector(x: f64, y: f64, z: f64) Tuple {
        return .{ .x = x, .y = y, .z = z, .w = 0.0 };
    }

    pub fn new(x: f64, y: f64, z: f64, w: f64) Tuple {
        return .{ .x = x, .y = y, .z = z, .w = w };
    }

    pub fn isPoint(self: Tuple) bool {
        return self.w == 1.0;
    }

    pub fn isVector(self: Tuple) bool {
        return self.w == 0.0;
    }

    pub fn add(self: Tuple, other: Tuple) Tuple {
        return .{ .x = self.x + other.x, .y = self.y + other.y, .z = self.z + other.z, .w = self.w + other.w };
    }

    pub fn sub(self: Tuple, other: Tuple) Tuple {
        return .{ .x = self.x - other.x, .y = self.y - other.y, .z = self.z - other.z, .w = self.w - other.w };
    }

    pub fn negate(self: Tuple) Tuple {
        return .{ .x = -self.x, .y = -self.y, .z = -self.z, .w = -self.w };
    }

    pub fn scale(self: Tuple, s: f64) Tuple {
        return .{ .x = self.x * s, .y = self.y * s, .z = self.z * s, .w = self.w * s };
    }

    pub fn divScalar(self: Tuple, s: f64) Tuple {
        return self.scale(1.0 / s);
    }

    pub fn magnitude(self: Tuple) f64 {
        return @sqrt(self.x * self.x + self.y * self.y + self.z * self.z + self.w * self.w);
    }

    pub fn lengthSq(self: Tuple) f64 {
        return self.x * self.x + self.y * self.y + self.z * self.z;
    }

    pub fn normalize(self: Tuple) Tuple {
        const m = self.magnitude();
        return self.scale(1.0 / m);
    }

    pub fn dot(self: Tuple, other: Tuple) f64 {
        return self.x * other.x + self.y * other.y + self.z * other.z + self.w * other.w;
    }

    pub fn cross(self: Tuple, other: Tuple) Tuple {
        return vector(
            self.y * other.z - self.z * other.y,
            self.z * other.x - self.x * other.z,
            self.x * other.y - self.y * other.x,
        );
    }

    pub fn reflect(self: Tuple, normal: Tuple) Tuple {
        return self.sub(normal.scale(2.0 * self.dot(normal)));
    }

    pub fn eql(self: Tuple, other: Tuple) bool {
        return @abs(self.x - other.x) < EPSILON and
            @abs(self.y - other.y) < EPSILON and
            @abs(self.z - other.z) < EPSILON and
            @abs(self.w - other.w) < EPSILON;
    }
};

test "tuple basics" {
    const t = testing;
    const p = Tuple.point(4.3, -4.2, 3.1);
    try t.expect(p.isPoint());
    try t.expect(!p.isVector());

    const v = Tuple.vector(4.3, -4.2, 3.1);
    try t.expect(v.isVector());
    try t.expect(!v.isPoint());

    const a = Tuple.point(3, -2, 5);
    const b = Tuple.vector(-2, 3, 1);
    const sum = a.add(b);
    try t.expect(sum.eql(Tuple.point(1, 1, 6)));

    const v2 = Tuple.vector(1, -1, 0);
    const n = Tuple.vector(0, 1, 0);
    const r = v2.reflect(n);
    try t.expect(r.eql(Tuple.vector(1, 1, 0)));
}

const testing = std.testing;
