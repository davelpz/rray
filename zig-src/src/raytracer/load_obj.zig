const std = @import("std");
const Tuple = @import("../math/tuple.zig").Tuple;
const Matrix = @import("../math/matrix.zig").Matrix;
const Material = @import("material.zig").Material;
const objectMod = @import("object.zig");
const Object = objectMod.Object;
const ObjectDB = objectMod.ObjectDB;

pub fn loadObj(
    allocator: std.mem.Allocator,
    io: std.Io,
    path: []const u8,
    material: Material,
    db: *ObjectDB,
) !usize {
    const contents = try std.Io.Dir.cwd().readFileAlloc(io, path, allocator, .unlimited);
    defer allocator.free(contents);
    return parseObj(allocator, contents, material, db);
}

fn parseObj(
    allocator: std.mem.Allocator,
    contents: []const u8,
    material: Material,
    db: *ObjectDB,
) !usize {
    var vertices: std.ArrayList(Tuple) = .empty;
    defer vertices.deinit(allocator);
    var normals: std.ArrayList(Tuple) = .empty;
    defer normals.deinit(allocator);

    // Collect triangles per current group
    var all_tri_ids: std.ArrayList(usize) = .empty;
    defer all_tri_ids.deinit(allocator);

    var lines = std.mem.splitScalar(u8, contents, '\n');
    while (lines.next()) |raw_line| {
        const line = std.mem.trim(u8, raw_line, " \r\t");
        if (line.len == 0 or line[0] == '#') continue;

        var tokens = std.mem.tokenizeScalar(u8, line, ' ');
        const cmd = tokens.next() orelse continue;

        if (std.mem.eql(u8, cmd, "v")) {
            const x = parseF64(tokens.next() orelse continue) orelse continue;
            const y = parseF64(tokens.next() orelse continue) orelse continue;
            const z = parseF64(tokens.next() orelse continue) orelse continue;
            try vertices.append(allocator, Tuple.point(x, y, z));
        } else if (std.mem.eql(u8, cmd, "vn")) {
            const x = parseF64(tokens.next() orelse continue) orelse continue;
            const y = parseF64(tokens.next() orelse continue) orelse continue;
            const z = parseF64(tokens.next() orelse continue) orelse continue;
            try normals.append(allocator, Tuple.vector(x, y, z));
        } else if (std.mem.eql(u8, cmd, "f")) {
            const tri_ids = try parseFace(allocator, &tokens, &vertices, &normals, material, db);
            defer allocator.free(tri_ids);
            try all_tri_ids.appendSlice(allocator, tri_ids);
        }
    }

    const child_ids = try allocator.dupe(usize, all_tri_ids.items);
    const group_obj = Object{
        .transform = Matrix.identity(),
        .transform_inv = Matrix.identity(),
        .material = material,
        .shape = .{ .group = .{ .child_ids = child_ids } },
    };
    const group_id = try db.add(group_obj);

    // Set parent_id for each child
    for (child_ids) |cid| {
        db.getMut(cid).parent_id = group_id;
    }

    return group_id;
}

fn parseFace(
    allocator: std.mem.Allocator,
    tokens: *std.mem.TokenIterator(u8, .scalar),
    vertices: *const std.ArrayList(Tuple),
    normals: *const std.ArrayList(Tuple),
    material: Material,
    db: *ObjectDB,
) ![]usize {
    var vert_ids: std.ArrayList(usize) = .empty;
    defer vert_ids.deinit(allocator);
    var norm_ids: std.ArrayList(?usize) = .empty;
    defer norm_ids.deinit(allocator);

    while (tokens.next()) |tok| {
        const parsed = parseFaceVertex(tok);
        try vert_ids.append(allocator, parsed.v);
        try norm_ids.append(allocator, parsed.vn);
    }

    const has_normals = blk: {
        for (norm_ids.items) |n| {
            if (n == null) break :blk false;
        }
        break :blk norm_ids.items.len > 0;
    };

    var tri_ids: std.ArrayList(usize) = .empty;
    errdefer tri_ids.deinit(allocator);

    // Fan triangulation: v[0], v[i], v[i+1]
    for (1..vert_ids.items.len - 1) |i| {
        const vi0 = vert_ids.items[0];
        const vi1 = vert_ids.items[i];
        const vi2 = vert_ids.items[i + 1];

        if (vi0 >= vertices.items.len or vi1 >= vertices.items.len or vi2 >= vertices.items.len) continue;

        const p1 = vertices.items[vi0];
        const p2 = vertices.items[vi1];
        const p3 = vertices.items[vi2];

        const tri_id = if (has_normals) blk: {
            const ni0 = norm_ids.items[0].?;
            const ni1 = norm_ids.items[i].?;
            const ni2 = norm_ids.items[i + 1].?;
            if (ni0 >= normals.items.len or ni1 >= normals.items.len or ni2 >= normals.items.len) {
                break :blk try addTriangle(p1, p2, p3, material, db);
            }
            const n1 = normals.items[ni0];
            const n2 = normals.items[ni1];
            const n3 = normals.items[ni2];
            break :blk try addSmoothTriangle(p1, p2, p3, n1, n2, n3, material, db);
        } else try addTriangle(p1, p2, p3, material, db);

        try tri_ids.append(allocator, tri_id);
    }

    return tri_ids.toOwnedSlice(allocator);
}

const FaceVertex = struct { v: usize, vt: ?usize, vn: ?usize };

fn parseFaceVertex(tok: []const u8) FaceVertex {
    var parts = std.mem.splitScalar(u8, tok, '/');
    const v_str = parts.next() orelse "0";
    const vt_str = parts.next();
    const vn_str = parts.next();

    // OBJ is 1-indexed; convert to 0-indexed
    const v = (std.fmt.parseInt(usize, v_str, 10) catch 1) -| 1;
    const vt = if (vt_str) |s| blk: {
        if (s.len == 0) break :blk null;
        break :blk (std.fmt.parseInt(usize, s, 10) catch 1) -| 1;
    } else null;
    const vn = if (vn_str) |s| blk: {
        if (s.len == 0) break :blk null;
        break :blk (std.fmt.parseInt(usize, s, 10) catch 1) -| 1;
    } else null;

    return .{ .v = v, .vt = vt, .vn = vn };
}

fn addTriangle(p1: Tuple, p2: Tuple, p3: Tuple, material: Material, db: *ObjectDB) !usize {
    const obj = Object{
        .transform = Matrix.identity(),
        .transform_inv = Matrix.identity(),
        .material = material,
        .shape = .{ .triangle = objectMod.Triangle.init(p1, p2, p3) },
    };
    return db.add(obj);
}

fn addSmoothTriangle(p1: Tuple, p2: Tuple, p3: Tuple, n1: Tuple, n2: Tuple, n3: Tuple, material: Material, db: *ObjectDB) !usize {
    const obj = Object{
        .transform = Matrix.identity(),
        .transform_inv = Matrix.identity(),
        .material = material,
        .shape = .{ .smooth_triangle = objectMod.SmoothTriangle.init(p1, p2, p3, n1, n2, n3) },
    };
    return db.add(obj);
}

fn parseF64(s: []const u8) ?f64 {
    return std.fmt.parseFloat(f64, s) catch null;
}
