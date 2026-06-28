const Tuple = @import("../math/tuple.zig").Tuple;
const Matrix = @import("../math/matrix.zig").Matrix;

pub const Ray = struct {
    origin: Tuple,
    direction: Tuple,

    pub fn init(origin: Tuple, direction: Tuple) Ray {
        return .{ .origin = origin, .direction = direction };
    }

    pub fn position(self: Ray, t: f64) Tuple {
        return self.origin.add(self.direction.scale(t));
    }

    pub fn transform(self: Ray, m: Matrix) Ray {
        return .{
            .origin = m.mulTuple(self.origin),
            .direction = m.mulTuple(self.direction),
        };
    }
};
