pub const Intersection = struct {
    t: f64,
    object_id: usize,
    u: f64 = 0,
    v: f64 = 0,
};

pub fn hit(xs: []const Intersection) ?Intersection {
    var best: ?Intersection = null;
    for (xs) |i| {
        if (i.t >= 0) {
            if (best == null or i.t < best.?.t) {
                best = i;
            }
        }
    }
    return best;
}

pub fn sortIntersections(xs: []Intersection) void {
    std.mem.sort(Intersection, xs, {}, struct {
        fn lessThan(_: void, a: Intersection, b: Intersection) bool {
            return a.t < b.t;
        }
    }.lessThan);
}

const std = @import("std");
