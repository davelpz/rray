const std = @import("std");
const Tuple = @import("../math/tuple.zig").Tuple;
const Ray = @import("ray.zig").Ray;
const Intersection = @import("intersection.zig").Intersection;
const ObjectDB = @import("object.zig").ObjectDB;

const EPSILON: f64 = 0.00001;

pub const Computations = struct {
    t: f64,
    object_id: usize,
    point: Tuple,
    eyev: Tuple,
    normalv: Tuple,
    inside: bool,
    over_point: Tuple,
    under_point: Tuple,
    reflectv: Tuple,
    n1: f64,
    n2: f64,

    pub fn schlick(self: Computations) f64 {
        var cos = self.eyev.dot(self.normalv);
        if (self.n1 > self.n2) {
            const n = self.n1 / self.n2;
            const sin2_t = n * n * (1.0 - cos * cos);
            if (sin2_t > 1.0) return 1.0;
            cos = @sqrt(1.0 - sin2_t);
        }
        const r0 = ((self.n1 - self.n2) / (self.n1 + self.n2)) *
            ((self.n1 - self.n2) / (self.n1 + self.n2));
        return r0 + (1.0 - r0) * std.math.pow(f64, 1.0 - cos, 5.0);
    }
};

pub fn prepareComputations(
    hit_ix: Intersection,
    ray: Ray,
    xs: []const Intersection,
    db: *const ObjectDB,
) Computations {
    const point = ray.position(hit_ix.t);
    const eyev = ray.direction.negate();
    const obj = db.get(hit_ix.object_id);
    const normalv_raw = obj.normalAt(point, hit_ix, db);
    const inside = normalv_raw.dot(eyev) < 0.0;
    const normalv = if (inside) normalv_raw.negate() else normalv_raw;
    const over_point = point.add(normalv.scale(EPSILON));
    const under_point = point.sub(normalv.scale(EPSILON));
    const reflectv = ray.direction.reflect(normalv);

    var n1: f64 = 1.0;
    var n2: f64 = 1.0;
    var containers_buf: [64]usize = undefined;
    var containers_len: usize = 0;

    for (xs) |i| {
        const is_hit = i.t == hit_ix.t and i.object_id == hit_ix.object_id and
            i.u == hit_ix.u and i.v == hit_ix.v;

        if (is_hit) {
            n1 = if (containers_len == 0)
                1.0
            else
                db.get(containers_buf[containers_len - 1]).material.refractive_index;
        }

        var found_idx: ?usize = null;
        for (containers_buf[0..containers_len], 0..) |cid, idx| {
            if (cid == i.object_id) {
                found_idx = idx;
                break;
            }
        }
        if (found_idx) |fi| {
            // Remove by shifting
            var k = fi;
            while (k + 1 < containers_len) : (k += 1) {
                containers_buf[k] = containers_buf[k + 1];
            }
            containers_len -= 1;
        } else {
            if (containers_len < containers_buf.len) {
                containers_buf[containers_len] = i.object_id;
                containers_len += 1;
            }
        }

        if (is_hit) {
            n2 = if (containers_len == 0)
                1.0
            else
                db.get(containers_buf[containers_len - 1]).material.refractive_index;
            break;
        }
    }

    return .{
        .t = hit_ix.t,
        .object_id = hit_ix.object_id,
        .point = point,
        .eyev = eyev,
        .normalv = normalv,
        .inside = inside,
        .over_point = over_point,
        .under_point = under_point,
        .reflectv = reflectv,
        .n1 = n1,
        .n2 = n2,
    };
}
