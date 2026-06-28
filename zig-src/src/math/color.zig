const std = @import("std");
const EPSILON = @import("../main.zig").EPSILON;

pub const Color = struct {
    r: f64,
    g: f64,
    b: f64,

    pub fn init(r: f64, g: f64, b: f64) Color {
        return .{ .r = r, .g = g, .b = b };
    }

    pub fn white() Color {
        return init(1, 1, 1);
    }

    pub fn black() Color {
        return init(0, 0, 0);
    }

    pub fn add(self: Color, other: Color) Color {
        return init(self.r + other.r, self.g + other.g, self.b + other.b);
    }

    pub fn sub(self: Color, other: Color) Color {
        return init(self.r - other.r, self.g - other.g, self.b - other.b);
    }

    pub fn scale(self: Color, s: f64) Color {
        return init(self.r * s, self.g * s, self.b * s);
    }

    pub fn hadamard(self: Color, other: Color) Color {
        return init(self.r * other.r, self.g * other.g, self.b * other.b);
    }

    pub fn eql(self: Color, other: Color) bool {
        return @abs(self.r - other.r) < EPSILON and
            @abs(self.g - other.g) < EPSILON and
            @abs(self.b - other.b) < EPSILON;
    }
};
