const std = @import("std");
const Color = @import("../math/color.zig").Color;
const Tuple = @import("../math/tuple.zig").Tuple;
const Ray = @import("ray.zig").Ray;
const Intersection = @import("intersection.zig").Intersection;
const intersectionMod = @import("intersection.zig");
const Computations = @import("computations.zig").Computations;
const prepareComputations = @import("computations.zig").prepareComputations;
const ObjectDB = @import("object.zig").ObjectDB;
const Light = @import("light.zig").Light;
const lighting = @import("light.zig").lighting;

pub const Scene = struct {
    lights: std.ArrayList(Light),
    object_ids: std.ArrayList(usize),
    db: *ObjectDB,
    allocator: std.mem.Allocator,

    pub fn init(allocator: std.mem.Allocator, db: *ObjectDB) Scene {
        return .{
            .lights = .empty,
            .object_ids = .empty,
            .db = db,
            .allocator = allocator,
        };
    }

    pub fn deinit(self: *Scene) void {
        self.lights.deinit(self.allocator);
        self.object_ids.deinit(self.allocator);
    }

    pub fn addLight(self: *Scene, light: Light) !void {
        try self.lights.append(self.allocator, light);
    }

    pub fn addObjectId(self: *Scene, id: usize) !void {
        try self.object_ids.append(self.allocator, id);
    }

    pub fn intersect(self: *const Scene, ray: Ray, alloc: std.mem.Allocator) ![]Intersection {
        var xs: std.ArrayList(Intersection) = .empty;
        for (self.object_ids.items) |id| {
            const obj = self.db.get(id);
            const obj_xs = try obj.intersect(ray, self.db, alloc);
            try xs.appendSlice(alloc, obj_xs);
        }
        intersectionMod.sortIntersections(xs.items);
        return xs.toOwnedSlice(alloc);
    }

    pub fn colorAt(self: *const Scene, ray: Ray, remaining: usize, alloc: std.mem.Allocator) !Color {
        const xs = try self.intersect(ray, alloc);
        const h = intersectionMod.hit(xs) orelse return Color.black();
        const comps = prepareComputations(h, ray, xs, self.db);
        return self.shadeHit(comps, remaining, alloc);
    }

    pub fn shadeHit(self: *const Scene, comps: Computations, remaining: usize, alloc: std.mem.Allocator) !Color {
        var surface = Color.black();
        for (self.lights.items) |light| {
            const light_color = try self.shadeHitLight(comps, light, alloc);
            surface = surface.add(light_color);
        }

        const reflected = try self.reflectedColor(comps, remaining, alloc);
        const refracted = try self.refractedColor(comps, remaining, alloc);

        const obj = self.db.get(comps.object_id);
        const material = obj.material;

        if (material.reflective > 0.0 and material.transparency > 0.0) {
            const reflectance = comps.schlick();
            return surface.add(reflected.scale(reflectance)).add(refracted.scale(1.0 - reflectance));
        } else {
            return surface.add(reflected).add(refracted);
        }
    }

    fn shadeHitLight(self: *const Scene, comps: Computations, light: Light, alloc: std.mem.Allocator) !Color {
        return switch (light.kind) {
            .point => {
                const shadowed = try self.isShadowed(comps.over_point, light.position, alloc);
                return lighting(
                    comps.object_id,
                    light,
                    comps.over_point,
                    comps.eyev,
                    comps.normalv,
                    if (shadowed) 1.0 else 0.0,
                    self.db,
                );
            },
            .area => |a| {
                const amount = a.level * a.level;
                var total_shadowed: usize = 0;
                for (0..amount) |sample| {
                    const light_pos = light.samplePoint(sample, a.level);
                    if (try self.isShadowed(comps.over_point, light_pos, alloc)) {
                        total_shadowed += 1;
                    }
                }
                const shadow_ratio = @as(f64, @floatFromInt(total_shadowed)) /
                    @as(f64, @floatFromInt(amount));
                return lighting(
                    comps.object_id,
                    light,
                    comps.over_point,
                    comps.eyev,
                    comps.normalv,
                    shadow_ratio,
                    self.db,
                );
            },
        };
    }

    pub fn isShadowed(self: *const Scene, point: Tuple, light_position: Tuple, alloc: std.mem.Allocator) !bool {
        const v = light_position.sub(point);
        const distance = v.magnitude();
        const direction = v.normalize();
        const shadow_ray = Ray.init(point, direction);
        const xs = try self.intersect(shadow_ray, alloc);
        const h = intersectionMod.hit(xs) orelse return false;
        return h.t < distance;
    }

    pub fn reflectedColor(self: *const Scene, comps: Computations, remaining: usize, alloc: std.mem.Allocator) !Color {
        const obj = self.db.get(comps.object_id);
        if (remaining == 0 or obj.material.reflective == 0.0) return Color.black();
        const reflect_ray = Ray.init(comps.over_point, comps.reflectv);
        const color = try self.colorAt(reflect_ray, remaining - 1, alloc);
        return color.scale(obj.material.reflective);
    }

    pub fn refractedColor(self: *const Scene, comps: Computations, remaining: usize, alloc: std.mem.Allocator) !Color {
        const obj = self.db.get(comps.object_id);
        if (remaining == 0 or obj.material.transparency == 0.0) return Color.black();

        const n_ratio = comps.n1 / comps.n2;
        const cos_i = comps.eyev.dot(comps.normalv);
        const sin2_t = n_ratio * n_ratio * (1.0 - cos_i * cos_i);
        if (sin2_t > 1.0) return Color.black(); // total internal reflection

        const cos_t = @sqrt(1.0 - sin2_t);
        const direction = comps.normalv.scale(n_ratio * cos_i - cos_t).sub(comps.eyev.scale(n_ratio));
        const refract_ray = Ray.init(comps.under_point, direction);
        const color = try self.colorAt(refract_ray, remaining - 1, alloc);
        return color.scale(obj.material.transparency);
    }
};
