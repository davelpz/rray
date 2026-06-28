const std = @import("std");
const Tuple = @import("../math/tuple.zig").Tuple;
const Matrix = @import("../math/matrix.zig").Matrix;
const Ray = @import("ray.zig").Ray;
const Intersection = @import("intersection.zig").Intersection;
const Material = @import("material.zig").Material;

const EPSILON: f64 = 0.00001;

// ─── Quartic/Cubic/Quadratic Solvers ──────────────────────────────────────────

fn cbrt(x: f64) f64 {
    if (x >= 0) return std.math.pow(f64, x, 1.0 / 3.0);
    return -std.math.pow(f64, -x, 1.0 / 3.0);
}

// Solve t^2 + b*t + c = 0; returns count, roots in out[0..count]
fn solveQuadratic(b: f64, c: f64, out: *[2]f64) usize {
    const disc = b * b - 4.0 * c;
    if (disc < 0) return 0;
    const sq = @sqrt(disc);
    out[0] = (-b - sq) / 2.0;
    out[1] = (-b + sq) / 2.0;
    return if (disc < 1e-12) 1 else 2;
}

// Solve t^3 + a*t^2 + b*t + c = 0; returns count, roots in out[0..count]
fn solveCubic(a: f64, b: f64, c: f64, out: *[3]f64) usize {
    const p = b - a * a / 3.0;
    const q = 2.0 * a * a * a / 27.0 - a * b / 3.0 + c;
    const shift = -a / 3.0;

    const D = q * q / 4.0 + p * p * p / 27.0;

    if (D > 1e-10) {
        const sqD = @sqrt(D);
        out[0] = cbrt(-q / 2.0 + sqD) + cbrt(-q / 2.0 - sqD) + shift;
        return 1;
    } else if (D > -1e-10) {
        const m = cbrt(-q / 2.0);
        out[0] = 2.0 * m + shift;
        out[1] = -m + shift;
        return if (@abs(out[0] - out[1]) < 1e-10) 1 else 2;
    } else {
        // Three real roots via trigonometric method
        const r = @sqrt(-p * p * p / 27.0);
        const phi = std.math.acos(std.math.clamp(-q / (2.0 * r), -1.0, 1.0));
        const m = 2.0 * cbrt(r);
        out[0] = m * @cos(phi / 3.0) + shift;
        out[1] = m * @cos((phi + 2.0 * std.math.pi) / 3.0) + shift;
        out[2] = m * @cos((phi + 4.0 * std.math.pi) / 3.0) + shift;
        return 3;
    }
}

// Solve a4*t^4 + a3*t^3 + a2*t^2 + a1*t + a0 = 0; returns count, roots in out[0..count]
fn solveQuartic(a4: f64, a3: f64, a2: f64, a1: f64, a0: f64, out: *[4]f64) usize {
    const b = a3 / a4;
    const c = a2 / a4;
    const d = a1 / a4;
    const e = a0 / a4;

    // Depress via t = u - b/4
    const p = c - 3.0 * b * b / 8.0;
    const q = b * b * b / 8.0 - b * c / 2.0 + d;
    const r = -3.0 * b * b * b * b / 256.0 + b * b * c / 16.0 - b * d / 4.0 + e;
    const shift = -b / 4.0;

    var count: usize = 0;

    if (@abs(q) < 1e-10) {
        // Biquadratic: u^4 + p*u^2 + r = 0 => solve as quadratic in u^2
        var quad: [2]f64 = undefined;
        const nq = solveQuadratic(p, r, &quad);
        for (quad[0..nq]) |val| {
            if (val >= 0) {
                const sq = @sqrt(val);
                out[count] = sq + shift;
                count += 1;
                if (sq > 1e-10) {
                    out[count] = -sq + shift;
                    count += 1;
                }
            }
        }
        return count;
    }

    // Resolvent cubic: m^3 - (p/2)*m^2 - r*m + (p*r/2 - q^2/8) = 0
    var cub: [3]f64 = undefined;
    const nc = solveCubic(-p / 2.0, -r, p * r / 2.0 - q * q / 8.0, &cub);

    // Pick largest root (maximizes chance of 2m-p >= 0)
    var m = cub[0];
    for (cub[1..nc]) |root| {
        if (root > m) m = root;
    }

    const two_m_minus_p = 2.0 * m - p;
    if (two_m_minus_p < 0) return 0;

    const s = @sqrt(two_m_minus_p);
    const q_over_2s = q / (2.0 * s);

    var q1: [2]f64 = undefined;
    const n1 = solveQuadratic(s, m - q_over_2s, &q1);
    for (q1[0..n1]) |u| {
        out[count] = u + shift;
        count += 1;
    }

    var q2: [2]f64 = undefined;
    const n2 = solveQuadratic(-s, m + q_over_2s, &q2);
    for (q2[0..n2]) |u| {
        out[count] = u + shift;
        count += 1;
    }

    return count;
}

// ─── AABB ─────────────────────────────────────────────────────────────────────

pub const Aabb = struct {
    min: Tuple,
    max: Tuple,

    pub fn init(mn: Tuple, mx: Tuple) Aabb {
        return .{ .min = mn, .max = mx };
    }

    fn checkAxis(origin: f64, direction: f64, mn: f64, mx: f64) struct { tmin: f64, tmax: f64 } {
        const tmin_num = mn - origin;
        const tmax_num = mx - origin;
        var tmin: f64 = undefined;
        var tmax: f64 = undefined;
        if (@abs(direction) >= EPSILON) {
            tmin = tmin_num / direction;
            tmax = tmax_num / direction;
        } else {
            tmin = tmin_num * std.math.inf(f64);
            tmax = tmax_num * std.math.inf(f64);
        }
        if (tmin > tmax) {
            const tmp = tmin;
            tmin = tmax;
            tmax = tmp;
        }
        return .{ .tmin = tmin, .tmax = tmax };
    }

    pub fn intersect(self: Aabb, ray: Ray) bool {
        const x = checkAxis(ray.origin.x, ray.direction.x, self.min.x, self.max.x);
        const y = checkAxis(ray.origin.y, ray.direction.y, self.min.y, self.max.y);
        const z = checkAxis(ray.origin.z, ray.direction.z, self.min.z, self.max.z);
        const tmin = @max(x.tmin, @max(y.tmin, z.tmin));
        const tmax = @min(x.tmax, @min(y.tmax, z.tmax));
        return tmin <= tmax;
    }

    pub fn adjustPoint(self: *Aabb, x: f64, y: f64, z: f64) void {
        self.min = Tuple.point(@min(self.min.x, x), @min(self.min.y, y), @min(self.min.z, z));
        self.max = Tuple.point(@max(self.max.x, x), @max(self.max.y, y), @max(self.max.z, z));
    }

    pub fn adjustAabb(self: *Aabb, other: Aabb) void {
        self.adjustPoint(other.min.x, other.min.y, other.min.z);
        self.adjustPoint(other.max.x, other.max.y, other.max.z);
    }

    pub fn applyTransform(self: Aabb, t: Matrix) Aabb {
        const corners = [8]Tuple{
            Tuple.point(self.min.x, self.min.y, self.min.z),
            Tuple.point(self.min.x, self.min.y, self.max.z),
            Tuple.point(self.min.x, self.max.y, self.min.z),
            Tuple.point(self.min.x, self.max.y, self.max.z),
            Tuple.point(self.max.x, self.min.y, self.min.z),
            Tuple.point(self.max.x, self.min.y, self.max.z),
            Tuple.point(self.max.x, self.max.y, self.min.z),
            Tuple.point(self.max.x, self.max.y, self.max.z),
        };
        const inf = std.math.inf(f64);
        var result = Aabb.init(
            Tuple.point(inf, inf, inf),
            Tuple.point(-inf, -inf, -inf),
        );
        for (corners) |corner| {
            const tc = t.mulTuple(corner);
            result.adjustPoint(tc.x, tc.y, tc.z);
        }
        return result;
    }
};

// ─── Shape-specific data types ────────────────────────────────────────────────

pub const CsgOperation = enum { union_op, intersection_op, difference_op };

pub const Sphere = struct {};
pub const Plane = struct {};
pub const Cube = struct {};
pub const Cylinder = struct {
    minimum: f64,
    maximum: f64,
    closed: bool,
};
pub const Cone = struct {
    minimum: f64,
    maximum: f64,
    closed: bool,
};
pub const Torus = struct {
    minor_radius: f64,
};
pub const Triangle = struct {
    p1: Tuple,
    p2: Tuple,
    p3: Tuple,
    e1: Tuple,
    e2: Tuple,
    normal: Tuple,

    pub fn init(p1: Tuple, p2: Tuple, p3: Tuple) Triangle {
        const e1 = p2.sub(p1);
        const e2 = p3.sub(p1);
        return .{
            .p1 = p1,
            .p2 = p2,
            .p3 = p3,
            .e1 = e1,
            .e2 = e2,
            .normal = e2.cross(e1).normalize(),
        };
    }
};
pub const SmoothTriangle = struct {
    p1: Tuple,
    p2: Tuple,
    p3: Tuple,
    n1: Tuple,
    n2: Tuple,
    n3: Tuple,
    e1: Tuple,
    e2: Tuple,
    normal: Tuple,

    pub fn init(p1: Tuple, p2: Tuple, p3: Tuple, n1: Tuple, n2: Tuple, n3: Tuple) SmoothTriangle {
        const e1 = p2.sub(p1);
        const e2 = p3.sub(p1);
        return .{
            .p1 = p1,
            .p2 = p2,
            .p3 = p3,
            .n1 = n1,
            .n2 = n2,
            .n3 = n3,
            .e1 = e1,
            .e2 = e2,
            .normal = e2.cross(e1).normalize(),
        };
    }
};
pub const Group = struct {
    child_ids: []const usize,
};
pub const Csg = struct {
    left_id: usize,
    right_id: usize,
    operation: CsgOperation,
};

pub const ShapeData = union(enum) {
    sphere: Sphere,
    plane: Plane,
    cube: Cube,
    cylinder: Cylinder,
    cone: Cone,
    torus: Torus,
    triangle: Triangle,
    smooth_triangle: SmoothTriangle,
    group: Group,
    csg: Csg,
};

// ─── Object ───────────────────────────────────────────────────────────────────

pub const Object = struct {
    id: usize = 0,
    parent_id: ?usize = null,
    transform: Matrix,
    transform_inv: Matrix,
    material: Material,
    shape: ShapeData,

    pub fn recomputeInverse(self: *Object) void {
        self.transform_inv = self.transform.inverse();
    }

    pub fn intersect(self: *const Object, ray: Ray, db: *const ObjectDB, alloc: std.mem.Allocator) anyerror![]Intersection {
        const local_ray = ray.transform(self.transform_inv);
        return self.localIntersect(local_ray, db, alloc);
    }

    pub fn normalAt(self: *const Object, world_pt: Tuple, hit: Intersection, db: *const ObjectDB) Tuple {
        const local_pt = worldToObject(self.id, world_pt, db);
        const local_normal = self.localNormalAt(local_pt, hit);
        return normalToWorld(self.id, local_normal, db);
    }

    pub fn getAabb(self: *const Object, db: *const ObjectDB) Aabb {
        const inf = std.math.inf(f64);
        return switch (self.shape) {
            .sphere => Aabb.init(Tuple.point(-1, -1, -1), Tuple.point(1, 1, 1)),
            .plane => Aabb.init(
                Tuple.point(-inf, 0, -inf),
                Tuple.point(inf, 0, inf),
            ),
            .cube => Aabb.init(Tuple.point(-1, -1, -1), Tuple.point(1, 1, 1)),
            .cylinder => |cy| Aabb.init(
                Tuple.point(-1, cy.minimum, -1),
                Tuple.point(1, cy.maximum, 1),
            ),
            .cone => |co| {
                const limit = @max(@abs(co.minimum), @abs(co.maximum));
                return Aabb.init(
                    Tuple.point(-limit, co.minimum, -limit),
                    Tuple.point(limit, co.maximum, limit),
                );
            },
            .torus => |to| {
                const r = to.minor_radius;
                return Aabb.init(
                    Tuple.point(-1.0 - r, -1.0 - r, -r),
                    Tuple.point(1.0 + r, 1.0 + r, r),
                );
            },
            .triangle => |tr| {
                return Aabb.init(
                    Tuple.point(
                        @min(tr.p1.x, @min(tr.p2.x, tr.p3.x)),
                        @min(tr.p1.y, @min(tr.p2.y, tr.p3.y)),
                        @min(tr.p1.z, @min(tr.p2.z, tr.p3.z)),
                    ),
                    Tuple.point(
                        @max(tr.p1.x, @max(tr.p2.x, tr.p3.x)),
                        @max(tr.p1.y, @max(tr.p2.y, tr.p3.y)),
                        @max(tr.p1.z, @max(tr.p2.z, tr.p3.z)),
                    ),
                );
            },
            .smooth_triangle => |st| {
                return Aabb.init(
                    Tuple.point(
                        @min(st.p1.x, @min(st.p2.x, st.p3.x)),
                        @min(st.p1.y, @min(st.p2.y, st.p3.y)),
                        @min(st.p1.z, @min(st.p2.z, st.p3.z)),
                    ),
                    Tuple.point(
                        @max(st.p1.x, @max(st.p2.x, st.p3.x)),
                        @max(st.p1.y, @max(st.p2.y, st.p3.y)),
                        @max(st.p1.z, @max(st.p2.z, st.p3.z)),
                    ),
                );
            },
            .group => |g| {
                var aabb = Aabb.init(
                    Tuple.point(inf, inf, inf),
                    Tuple.point(-inf, -inf, -inf),
                );
                for (g.child_ids) |cid| {
                    const child = db.get(cid);
                    const child_aabb = child.getAabb(db).applyTransform(child.transform);
                    aabb.adjustAabb(child_aabb);
                }
                return aabb;
            },
            .csg => |cs| {
                var aabb = Aabb.init(
                    Tuple.point(inf, inf, inf),
                    Tuple.point(-inf, -inf, -inf),
                );
                const left = db.get(cs.left_id);
                const right = db.get(cs.right_id);
                const la = left.getAabb(db).applyTransform(left.transform);
                const ra = right.getAabb(db).applyTransform(right.transform);
                aabb.adjustAabb(la);
                aabb.adjustAabb(ra);
                return aabb;
            },
        };
    }

    pub fn includes(self: *const Object, id: usize, db: *const ObjectDB) bool {
        return switch (self.shape) {
            .group => |g| {
                for (g.child_ids) |cid| {
                    if (db.get(cid).includes(id, db)) return true;
                }
                return false;
            },
            .csg => |cs| id == cs.left_id or id == cs.right_id,
            else => self.id == id,
        };
    }

    pub fn uvMapping(self: *const Object, pt: Tuple) struct { u: f64, v: f64 } {
        switch (self.shape) {
            .sphere => {
                const theta = std.math.atan2(pt.z, pt.x);
                const phi = std.math.acos(pt.y / @sqrt(pt.lengthSq()));
                const u = (theta + std.math.pi) / (2.0 * std.math.pi);
                const v = 1.0 - phi / std.math.pi;
                return .{ .u = u, .v = v };
            },
            .plane => {
                var u = @mod(pt.x, 1.0);
                var v = @mod(pt.z, 1.0);
                if (u < 0) u = 1.0 + u;
                if (v < 0) v = 1.0 + v;
                return .{ .u = u, .v = v };
            },
            .cube => {
                const ax = @abs(pt.x);
                const ay = @abs(pt.y);
                const az = @abs(pt.z);
                if (ax >= ay and ax >= az) {
                    if (pt.x > 0) return .{ .u = (pt.z + 1.0) * 0.5, .v = (pt.y + 1.0) * 0.5 };
                    return .{ .u = (1.0 - pt.z) * 0.5, .v = (pt.y + 1.0) * 0.5 };
                } else if (ay >= ax and ay >= az) {
                    if (pt.y > 0) return .{ .u = (pt.x + 1.0) * 0.5, .v = (1.0 - pt.z) * 0.5 };
                    return .{ .u = (pt.x + 1.0) * 0.5, .v = (pt.z + 1.0) * 0.5 };
                } else {
                    if (pt.z > 0) return .{ .u = (pt.x + 1.0) * 0.5, .v = (pt.y + 1.0) * 0.5 };
                    return .{ .u = (1.0 - pt.x) * 0.5, .v = (pt.y + 1.0) * 0.5 };
                }
            },
            .cylinder => |cy| {
                if (cy.closed and (pt.y <= cy.minimum or pt.y >= cy.maximum)) {
                    return .{ .u = (pt.x + 1.0) / 2.0, .v = (pt.z + 1.0) / 2.0 };
                }
                const theta = std.math.atan2(pt.z, pt.x);
                const u = (theta + std.math.pi) / (2.0 * std.math.pi);
                var v = @mod(pt.y, 1.0);
                if (v < 0) v = 1.0 + v;
                return .{ .u = u, .v = v };
            },
            .cone => |co| {
                const y_min_dist = @abs(pt.y - co.minimum);
                const y_max_dist = @abs(pt.y - co.maximum);
                if (co.closed and (y_min_dist <= EPSILON or y_max_dist <= EPSILON)) {
                    const radius = @abs(pt.y);
                    return .{
                        .u = (pt.x / radius + 1.0) / 2.0,
                        .v = (pt.z / radius + 1.0) / 2.0,
                    };
                }
                const theta = (std.math.atan2(pt.z, pt.x) + std.math.pi) / (2.0 * std.math.pi);
                const height_range = co.maximum - co.minimum;
                const normalized_y = (pt.y - co.minimum) / height_range;
                return .{ .u = normalized_y, .v = theta };
            },
            .torus => {
                const u = (std.math.atan2(pt.y, pt.x) + std.math.pi) / (2.0 * std.math.pi);
                const dist_to_center = @sqrt(pt.x * pt.x + pt.y * pt.y) - 1.0;
                const v = (std.math.atan2(pt.z, dist_to_center) + std.math.pi) / (2.0 * std.math.pi);
                return .{ .u = u, .v = v };
            },
            .triangle, .smooth_triangle => {
                const p1 = switch (self.shape) {
                    .triangle => |tr| tr.p1,
                    .smooth_triangle => |st| st.p1,
                    else => unreachable,
                };
                const p2 = switch (self.shape) {
                    .triangle => |tr| tr.p2,
                    .smooth_triangle => |st| st.p2,
                    else => unreachable,
                };
                const p3 = switch (self.shape) {
                    .triangle => |tr| tr.p3,
                    .smooth_triangle => |st| st.p3,
                    else => unreachable,
                };
                const v0 = p2.sub(p1);
                const v1 = p3.sub(p1);
                const v2 = pt.sub(p1);
                const d00 = v0.dot(v0);
                const d01 = v0.dot(v1);
                const d11 = v1.dot(v1);
                const d20 = v2.dot(v0);
                const d21 = v2.dot(v1);
                const denom = d00 * d11 - d01 * d01;
                const lambda1 = (d11 * d20 - d01 * d21) / denom;
                const lambda2 = (d00 * d21 - d01 * d20) / denom;
                return .{ .u = lambda1, .v = lambda2 };
            },
            .group, .csg => return .{ .u = 0, .v = 0 },
        }
    }

    fn localIntersect(self: *const Object, ray: Ray, db: *const ObjectDB, alloc: std.mem.Allocator) ![]Intersection {
        var xs: std.ArrayList(Intersection) = .empty;
        switch (self.shape) {
            .sphere => {
                const sphere_to_ray = ray.origin.sub(Tuple.point(0, 0, 0));
                const a = ray.direction.dot(ray.direction);
                const b = 2.0 * ray.direction.dot(sphere_to_ray);
                const c = sphere_to_ray.dot(sphere_to_ray) - 1.0;
                const disc = b * b - 4.0 * a * c;
                if (disc >= 0) {
                    const sq = @sqrt(disc);
                    try xs.append(alloc, .{ .t = (-b - sq) / (2.0 * a), .object_id = self.id, .u = 0, .v = 0 });
                    try xs.append(alloc, .{ .t = (-b + sq) / (2.0 * a), .object_id = self.id, .u = 0, .v = 0 });
                }
            },
            .plane => {
                if (@abs(ray.direction.y) >= EPSILON) {
                    const t = -ray.origin.y / ray.direction.y;
                    try xs.append(alloc, .{ .t = t, .object_id = self.id, .u = 0, .v = 0 });
                }
            },
            .cube => {
                const xt = cubeCheckAxis(ray.origin.x, ray.direction.x);
                const yt = cubeCheckAxis(ray.origin.y, ray.direction.y);
                const zt = cubeCheckAxis(ray.origin.z, ray.direction.z);
                const tmin = @max(xt.tmin, @max(yt.tmin, zt.tmin));
                const tmax = @min(xt.tmax, @min(yt.tmax, zt.tmax));
                if (tmin <= tmax) {
                    try xs.append(alloc, .{ .t = tmin, .object_id = self.id, .u = 0, .v = 0 });
                    try xs.append(alloc, .{ .t = tmax, .object_id = self.id, .u = 0, .v = 0 });
                }
            },
            .cylinder => |cy| {
                try cylinderIntersect(ray, cy, self.id, alloc, &xs);
            },
            .cone => |co| {
                try coneIntersect(ray, co, self.id, alloc, &xs);
            },
            .torus => |to| {
                const ox = ray.origin.x;
                const oy = ray.origin.y;
                const oz = ray.origin.z;
                const dx = ray.direction.x;
                const dy = ray.direction.y;
                const dz = ray.direction.z;
                const r = to.minor_radius;
                const r_sq = r * r;
                const sum_d_sq = dx * dx + dy * dy + dz * dz;
                const e = ox * ox + oy * oy + oz * oz - r_sq + 1.0;
                const f = ray.origin.dot(ray.direction);
                const a4 = sum_d_sq * sum_d_sq;
                const a3 = 4.0 * sum_d_sq * f;
                const a2 = 2.0 * sum_d_sq * e + 4.0 * f * f - 4.0 * (dx * dx + dy * dy);
                const a1 = 4.0 * e * f - 8.0 * (ox * dx + oy * dy);
                const a0 = e * e - 4.0 * (ox * ox + oy * oy);
                var roots: [4]f64 = undefined;
                const n = solveQuartic(a4, a3, a2, a1, a0, &roots);
                for (roots[0..n]) |t| {
                    if (t > 0) {
                        try xs.append(alloc, .{ .t = t, .object_id = self.id, .u = 0, .v = 0 });
                    }
                }
            },
            .triangle => |tr| {
                const dir_cross_e2 = ray.direction.cross(tr.e2);
                const det = tr.e1.dot(dir_cross_e2);
                if (@abs(det) >= EPSILON) {
                    const f = 1.0 / det;
                    const p1_to_origin = ray.origin.sub(tr.p1);
                    const u = f * p1_to_origin.dot(dir_cross_e2);
                    if (u >= 0.0 and u <= 1.0) {
                        const origin_cross_e1 = p1_to_origin.cross(tr.e1);
                        const v = f * ray.direction.dot(origin_cross_e1);
                        if (v >= 0.0 and (u + v) <= 1.0) {
                            const t = f * tr.e2.dot(origin_cross_e1);
                            try xs.append(alloc, .{ .t = t, .object_id = self.id, .u = u, .v = v });
                        }
                    }
                }
            },
            .smooth_triangle => |st| {
                const dir_cross_e2 = ray.direction.cross(st.e2);
                const det = st.e1.dot(dir_cross_e2);
                if (@abs(det) >= EPSILON) {
                    const f = 1.0 / det;
                    const p1_to_origin = ray.origin.sub(st.p1);
                    const u = f * p1_to_origin.dot(dir_cross_e2);
                    if (u >= 0.0 and u <= 1.0) {
                        const origin_cross_e1 = p1_to_origin.cross(st.e1);
                        const v = f * ray.direction.dot(origin_cross_e1);
                        if (v >= 0.0 and (u + v) <= 1.0) {
                            const t = f * st.e2.dot(origin_cross_e1);
                            try xs.append(alloc, .{ .t = t, .object_id = self.id, .u = u, .v = v });
                        }
                    }
                }
            },
            .group => |g| {
                if (self.getAabb(db).intersect(ray)) {
                    for (g.child_ids) |cid| {
                        const child = db.get(cid);
                        const child_xs = try child.intersect(ray, db, alloc);
                        try xs.appendSlice(alloc, child_xs);
                    }
                    std.mem.sort(Intersection, xs.items, {}, struct {
                        fn lt(_: void, a: Intersection, b: Intersection) bool {
                            return a.t < b.t;
                        }
                    }.lt);
                }
            },
            .csg => |cs| {
                const left = db.get(cs.left_id);
                const right = db.get(cs.right_id);
                const left_xs = try left.intersect(ray, db, alloc);
                const right_xs = try right.intersect(ray, db, alloc);
                try xs.appendSlice(alloc, left_xs);
                try xs.appendSlice(alloc, right_xs);
                std.mem.sort(Intersection, xs.items, {}, struct {
                    fn lt(_: void, a: Intersection, b: Intersection) bool {
                        return a.t < b.t;
                    }
                }.lt);
                const filtered = try csgFilterIntersections(cs, xs.items, db, alloc);
                return filtered;
            },
        }
        return try xs.toOwnedSlice(alloc);
    }

    fn localNormalAt(self: *const Object, pt: Tuple, hit: Intersection) Tuple {
        return switch (self.shape) {
            .sphere => pt.sub(Tuple.point(0, 0, 0)),
            .plane => Tuple.vector(0, 1, 0),
            .cube => {
                const maxc = @max(@abs(pt.x), @max(@abs(pt.y), @abs(pt.z)));
                if (maxc == @abs(pt.x)) return Tuple.vector(pt.x, 0, 0);
                if (maxc == @abs(pt.y)) return Tuple.vector(0, pt.y, 0);
                return Tuple.vector(0, 0, pt.z);
            },
            .cylinder => |cy| {
                const dist = pt.x * pt.x + pt.z * pt.z;
                if (dist < 1.0 and pt.y >= cy.maximum - EPSILON) return Tuple.vector(0, 1, 0);
                if (dist < 1.0 and pt.y <= cy.minimum + EPSILON) return Tuple.vector(0, -1, 0);
                return Tuple.vector(pt.x, 0, pt.z);
            },
            .cone => |co| {
                const dist = pt.x * pt.x + pt.z * pt.z;
                if (dist < 1.0 and pt.y >= co.maximum - EPSILON) return Tuple.vector(0, 1, 0);
                if (dist < 1.0 and pt.y <= co.minimum + EPSILON) return Tuple.vector(0, -1, 0);
                var y = @sqrt(dist);
                if (pt.y > 0) y = -y;
                return Tuple.vector(pt.x, y, pt.z);
            },
            .torus => |to| {
                const sum_sq = pt.x * pt.x + pt.y * pt.y + pt.z * pt.z;
                const param_sq = 1.0 + to.minor_radius * to.minor_radius;
                return Tuple.vector(
                    4.0 * pt.x * (sum_sq - param_sq),
                    4.0 * pt.y * (sum_sq - param_sq),
                    4.0 * pt.z * (sum_sq - param_sq + 2.0),
                ).normalize();
            },
            .triangle => |tr| tr.normal,
            .smooth_triangle => |st| {
                return st.n2.scale(hit.u)
                    .add(st.n3.scale(hit.v))
                    .add(st.n1.scale(1.0 - hit.u - hit.v));
            },
            .group => Tuple.vector(0, 1, 0), // should not be called
            .csg => Tuple.vector(0, 1, 0), // should not be called
        };
    }
};

// ─── Object WorldSpace helpers ────────────────────────────────────────────────

pub fn worldToObject(object_id: usize, world_pt: Tuple, db: *const ObjectDB) Tuple {
    const obj = db.get(object_id);
    var pt = world_pt;
    if (obj.parent_id) |pid| {
        pt = worldToObject(pid, pt, db);
    }
    return obj.transform_inv.mulTuple(pt);
}

pub fn normalToWorld(object_id: usize, local_normal: Tuple, db: *const ObjectDB) Tuple {
    const obj = db.get(object_id);
    var n = obj.transform_inv.transpose().mulTuple(local_normal);
    n.w = 0.0;
    n = n.normalize();
    if (obj.parent_id) |pid| {
        n = normalToWorld(pid, n, db);
    }
    return n;
}

// ─── ObjectDB ─────────────────────────────────────────────────────────────────

pub const ObjectDB = struct {
    objects: std.ArrayList(Object),
    allocator: std.mem.Allocator,

    pub fn init(allocator: std.mem.Allocator) ObjectDB {
        return .{ .objects = .empty, .allocator = allocator };
    }

    pub fn deinit(self: *ObjectDB) void {
        self.objects.deinit(self.allocator);
    }

    pub fn add(self: *ObjectDB, obj: Object) !usize {
        const id = self.objects.items.len;
        try self.objects.append(self.allocator, obj);
        self.objects.items[id].id = id;
        self.objects.items[id].transform_inv = self.objects.items[id].transform.inverse();
        return id;
    }

    pub fn get(self: *const ObjectDB, id: usize) *const Object {
        return &self.objects.items[id];
    }

    pub fn getMut(self: *ObjectDB, id: usize) *Object {
        return &self.objects.items[id];
    }
};

// ─── Private helpers ──────────────────────────────────────────────────────────

fn cubeCheckAxis(origin: f64, direction: f64) struct { tmin: f64, tmax: f64 } {
    const tmin_num = -1.0 - origin;
    const tmax_num = 1.0 - origin;
    var tmin: f64 = undefined;
    var tmax: f64 = undefined;
    if (@abs(direction) >= EPSILON) {
        tmin = tmin_num / direction;
        tmax = tmax_num / direction;
    } else {
        tmin = tmin_num * std.math.inf(f64);
        tmax = tmax_num * std.math.inf(f64);
    }
    if (tmin > tmax) {
        const tmp = tmin;
        tmin = tmax;
        tmax = tmp;
    }
    return .{ .tmin = tmin, .tmax = tmax };
}

fn cylinderCapHit(ray: Ray, t: f64) bool {
    const x = ray.origin.x + t * ray.direction.x;
    const z = ray.origin.z + t * ray.direction.z;
    return (x * x + z * z) <= 1.0;
}

fn cylinderIntersect(ray: Ray, cy: Cylinder, id: usize, alloc: std.mem.Allocator, xs: *std.ArrayList(Intersection)) !void {
    const a = ray.direction.x * ray.direction.x + ray.direction.z * ray.direction.z;
    if (@abs(a) > EPSILON) {
        const b = 2.0 * ray.origin.x * ray.direction.x + 2.0 * ray.origin.z * ray.direction.z;
        const c = ray.origin.x * ray.origin.x + ray.origin.z * ray.origin.z - 1.0;
        const disc = b * b - 4.0 * a * c;
        if (disc >= 0) {
            var t0 = (-b - @sqrt(disc)) / (2.0 * a);
            var t1 = (-b + @sqrt(disc)) / (2.0 * a);
            if (t0 > t1) {
                const tmp = t0;
                t0 = t1;
                t1 = tmp;
            }
            const y0 = ray.origin.y + t0 * ray.direction.y;
            if (cy.minimum < y0 and y0 < cy.maximum) {
                try xs.append(alloc, .{ .t = t0, .object_id = id, .u = 0, .v = 0 });
            }
            const y1 = ray.origin.y + t1 * ray.direction.y;
            if (cy.minimum < y1 and y1 < cy.maximum) {
                try xs.append(alloc, .{ .t = t1, .object_id = id, .u = 0, .v = 0 });
            }
        }
    }
    if (cy.closed and @abs(ray.direction.y) >= EPSILON) {
        const t_min = (cy.minimum - ray.origin.y) / ray.direction.y;
        if (cylinderCapHit(ray, t_min)) {
            try xs.append(alloc, .{ .t = t_min, .object_id = id, .u = 0, .v = 0 });
        }
        const t_max = (cy.maximum - ray.origin.y) / ray.direction.y;
        if (cylinderCapHit(ray, t_max)) {
            try xs.append(alloc, .{ .t = t_max, .object_id = id, .u = 0, .v = 0 });
        }
    }
}

fn coneCapHit(ray: Ray, t: f64) bool {
    const x = ray.origin.x + t * ray.direction.x;
    const y = ray.origin.y + t * ray.direction.y;
    const z = ray.origin.z + t * ray.direction.z;
    return (x * x + z * z) <= y * y;
}

fn coneIntersect(ray: Ray, co: Cone, id: usize, alloc: std.mem.Allocator, xs: *std.ArrayList(Intersection)) !void {
    const a = ray.direction.x * ray.direction.x - ray.direction.y * ray.direction.y + ray.direction.z * ray.direction.z;
    const b = 2.0 * ray.origin.x * ray.direction.x - 2.0 * ray.origin.y * ray.direction.y + 2.0 * ray.origin.z * ray.direction.z;

    if (@abs(a) < EPSILON and @abs(b) < EPSILON) {
        // Caps only
    } else if (@abs(a) < EPSILON) {
        const t = -(ray.origin.x * ray.origin.x - ray.origin.y * ray.origin.y + ray.origin.z * ray.origin.z) / (2.0 * b);
        const y = ray.origin.y + t * ray.direction.y;
        if (co.minimum < y and y < co.maximum) {
            try xs.append(alloc, .{ .t = t, .object_id = id, .u = 0, .v = 0 });
        }
    } else {
        const c = ray.origin.x * ray.origin.x - ray.origin.y * ray.origin.y + ray.origin.z * ray.origin.z;
        const disc = b * b - 4.0 * a * c;
        if (disc >= 0) {
            var t0 = (-b - @sqrt(disc)) / (2.0 * a);
            var t1 = (-b + @sqrt(disc)) / (2.0 * a);
            if (t0 > t1) {
                const tmp = t0;
                t0 = t1;
                t1 = tmp;
            }
            const y0 = ray.origin.y + t0 * ray.direction.y;
            if (co.minimum < y0 and y0 < co.maximum) {
                try xs.append(alloc, .{ .t = t0, .object_id = id, .u = 0, .v = 0 });
            }
            const y1 = ray.origin.y + t1 * ray.direction.y;
            if (co.minimum < y1 and y1 < co.maximum) {
                try xs.append(alloc, .{ .t = t1, .object_id = id, .u = 0, .v = 0 });
            }
        }
    }

    if (co.closed and @abs(ray.direction.y) >= EPSILON) {
        const t_min = (co.minimum - ray.origin.y) / ray.direction.y;
        if (coneCapHit(ray, t_min)) {
            try xs.append(alloc, .{ .t = t_min, .object_id = id, .u = 0, .v = 0 });
        }
        const t_max = (co.maximum - ray.origin.y) / ray.direction.y;
        if (coneCapHit(ray, t_max)) {
            try xs.append(alloc, .{ .t = t_max, .object_id = id, .u = 0, .v = 0 });
        }
    }
}

fn csgIntersectionAllowed(op: CsgOperation, lhit: bool, inl: bool, inr: bool) bool {
    return switch (op) {
        .union_op => (lhit and !inr) or (!lhit and !inl),
        .intersection_op => (lhit and inr) or (!lhit and inl),
        .difference_op => (lhit and !inr) or (!lhit and inl),
    };
}

fn csgFilterIntersections(cs: Csg, xs: []const Intersection, db: *const ObjectDB, alloc: std.mem.Allocator) ![]Intersection {
    var inl = false;
    var inr = false;
    var result: std.ArrayList(Intersection) = .empty;
    const left = db.get(cs.left_id);
    for (xs) |i| {
        const lhit = left.includes(i.object_id, db);
        if (csgIntersectionAllowed(cs.operation, lhit, inl, inr)) {
            try result.append(alloc, i);
        }
        if (lhit) inl = !inl else inr = !inr;
    }
    return try result.toOwnedSlice(alloc);
}
