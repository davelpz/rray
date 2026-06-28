const std = @import("std");
const zig_yaml = @import("yaml");
const Color = @import("../math/color.zig").Color;
const Matrix = @import("../math/matrix.zig").Matrix;
const Tuple = @import("../math/tuple.zig").Tuple;
const Material = @import("material.zig").Material;
const Pattern = @import("material.zig").Pattern;
const PatternKind = @import("material.zig").PatternKind;
const Texture = @import("texture.zig").Texture;
const objectMod = @import("object.zig");
const Object = objectMod.Object;
const ObjectDB = objectMod.ObjectDB;
const Light = @import("light.zig").Light;
const Scene = @import("scene.zig").Scene;
const Camera = @import("camera.zig").Camera;
const loadObj = @import("load_obj.zig").loadObj;

const Value = zig_yaml.Yaml.Value;
const Map = zig_yaml.Yaml.Map;

// Resolve a path from the YAML relative to the scene file's directory.
fn resolvePath(allocator: std.mem.Allocator, base_dir: []const u8, path: []const u8) ![]u8 {
    if (std.fs.path.isAbsolute(path)) return allocator.dupe(u8, path);
    return std.fs.path.join(allocator, &.{ base_dir, path });
}

pub fn renderSceneFromFile(
    path: []const u8,
    width: usize,
    height: usize,
    png_file: []const u8,
    aa: usize,
    io: std.Io,
    allocator: std.mem.Allocator,
) !void {
    const base_dir = std.fs.path.dirname(path) orelse ".";
    const raw = try std.Io.Dir.cwd().readFileAlloc(io, path, allocator, .unlimited);
    defer allocator.free(raw);
    // Normalize CR-only or CRLF line endings to LF for zig-yaml compatibility
    const contents = try normalizeCr(raw, allocator);
    defer allocator.free(contents);
    try renderSceneFromStr(contents, width, height, png_file, aa, io, allocator, base_dir);
}

fn normalizeCr(input: []const u8, allocator: std.mem.Allocator) ![]u8 {
    var out = try allocator.alloc(u8, input.len);
    var j: usize = 0;
    var i: usize = 0;
    while (i < input.len) : (i += 1) {
        if (input[i] == '\r') {
            out[j] = '\n';
            j += 1;
            if (i + 1 < input.len and input[i + 1] == '\n') {
                i += 1; // skip the LF in CRLF
            }
        } else {
            out[j] = input[i];
            j += 1;
        }
    }
    return out[0..j];
}

pub fn renderSceneFromStr(
    contents: []const u8,
    width: usize,
    height: usize,
    png_file: []const u8,
    aa: usize,
    io: std.Io,
    allocator: std.mem.Allocator,
    base_dir: []const u8,
) !void {
    var yaml: zig_yaml.Yaml = .{ .source = contents };
    try yaml.load(allocator);
    defer yaml.deinit(allocator);

    if (yaml.docs.items.len == 0) return error.EmptyYaml;
    const doc = yaml.docs.items[0];
    const doc_map = doc.asMap() orelse return error.InvalidYaml;

    var db = ObjectDB.init(allocator);
    defer db.deinit();

    // Pattern arena: lives as long as the scene
    var pattern_arena = std.heap.ArenaAllocator.init(allocator);
    defer pattern_arena.deinit();
    const pat_alloc = pattern_arena.allocator();

    var scene = Scene.init(allocator, &db);
    defer scene.deinit();

    const camera_val = doc_map.get("camera") orelse return error.NoCameraFound;
    const camera_map = camera_val.asMap() orelse return error.InvalidCamera;
    const camera = try buildCamera(camera_map, width * aa, height * aa);

    const lights_val = doc_map.get("lights") orelse return error.NoLightsFound;
    const lights_list = lights_val.asList() orelse return error.InvalidLights;
    for (lights_list) |light_val| {
        const light = try buildLight(light_val);
        try scene.addLight(light);
    }

    const scene_val = doc_map.get("scene") orelse return error.NoSceneFound;
    const scene_list = scene_val.asList() orelse return error.InvalidScene;
    for (scene_list) |obj_val| {
        const obj_map = obj_val.asMap() orelse continue;
        if (getBool(obj_map.get("hidden"), false)) continue;
        const id = try buildShape(obj_val, pat_alloc, io, allocator, &db, base_dir);
        try scene.addObjectId(id);
    }

    try camera.render(&scene, io, png_file, aa, allocator);
}

fn buildCamera(map: Map, hsize: usize, vsize: usize) !Camera {
    const fov = try getF64FromMap(map, "fov");
    const from_val = map.get("from") orelse return error.CameraFromMissing;
    const to_val = map.get("to") orelse return error.CameraToMissing;
    const up_val = map.get("up") orelse return error.CameraUpMissing;
    const from = try tupleFromList(from_val, true);
    const to = try tupleFromList(to_val, true);
    const up = try tupleFromList(up_val, false);

    var cam = Camera.new(hsize, vsize, degreesToRadians(fov));
    cam.setTransform(Matrix.viewTransform(from, to, up));
    return cam;
}

fn buildLight(val: Value) !Light {
    const map = val.asMap() orelse return error.InvalidLight;
    const type_str = (map.get("type") orelse return error.LightTypeMissing).asScalar() orelse return error.LightTypeMissing;
    const color_val = map.get("color") orelse return error.LightColorMissing;
    const intensity = try colorFromList(color_val);

    if (std.mem.eql(u8, type_str, "point")) {
        const pos_val = map.get("position") orelse return error.LightPosMissing;
        const position = try tupleFromList(pos_val, true);
        return Light.newPoint(position, intensity);
    } else if (std.mem.eql(u8, type_str, "area")) {
        const corner_val = map.get("corner") orelse return error.AreaLightCornerMissing;
        const uvec_val = map.get("uvec") orelse return error.AreaLightUMissing;
        const vvec_val = map.get("vvec") orelse return error.AreaLightVMissing;
        const corner = try tupleFromList(corner_val, true);
        const u = try tupleFromList(uvec_val, false);
        const v = try tupleFromList(vvec_val, false);
        const level: usize = blk: {
            const lv = map.get("level") orelse break :blk 5;
            break :blk @as(usize, @intFromFloat(getF64Default(lv, 5.0)));
        };
        return Light.newArea(corner, u, v, intensity, level);
    }
    return error.UnknownLightType;
}

fn buildShape(
    val: Value,
    pat_alloc: std.mem.Allocator,
    io: std.Io,
    allocator: std.mem.Allocator,
    db: *ObjectDB,
    base_dir: []const u8,
) anyerror!usize {
    const map = val.asMap() orelse return error.InvalidShape;
    const type_str = (map.get("type") orelse return error.ShapeTypeMissing).asScalar() orelse return error.ShapeTypeMissing;

    var base_obj: Object = undefined;
    var is_group_like = false;
    var group_id: usize = 0;

    if (std.mem.eql(u8, type_str, "group")) {
        group_id = try buildGroup(val, pat_alloc, io, allocator, db, base_dir);
        is_group_like = true;
    } else if (std.mem.eql(u8, type_str, "csg")) {
        group_id = try buildCsg(val, pat_alloc, io, allocator, db, base_dir);
        is_group_like = true;
    } else if (std.mem.eql(u8, type_str, "obj_file")) {
        const rel_path = (map.get("obj_file") orelse return error.ObjFileMissing).asScalar() orelse return error.ObjFileMissing;
        const file_path = try resolvePath(allocator, base_dir, rel_path);
        defer allocator.free(file_path);
        const mat = buildMaterial(map.get("material"), pat_alloc, io, allocator, base_dir);
        group_id = try loadObj(allocator, io, file_path, mat, db);
        is_group_like = true;
    } else {
        base_obj = try buildPrimitiveObject(map, type_str, pat_alloc, io, allocator);
    }

    if (is_group_like) {
        // Apply transform and material to group
        const transforms_val = map.get("transforms");
        if (transforms_val) |tv| {
            if (tv.asList()) |tlist| {
                const t = buildTransforms(tlist);
                db.getMut(group_id).transform = t;
                db.getMut(group_id).recomputeInverse();
            }
        }
        const mat = buildMaterial(map.get("material"), pat_alloc, io, allocator, base_dir);
        db.getMut(group_id).material = mat;
        return group_id;
    }

    // Apply transform
    const transforms_val = map.get("transforms");
    if (transforms_val) |tv| {
        if (tv.asList()) |tlist| {
            base_obj.transform = buildTransforms(tlist);
            base_obj.transform_inv = base_obj.transform.inverse();
        }
    }
    // Apply material
    base_obj.material = buildMaterial(map.get("material"), pat_alloc, io, allocator, base_dir);

    return db.add(base_obj);
}

fn buildPrimitiveObject(
    map: Map,
    type_str: []const u8,
    pat_alloc: std.mem.Allocator,
    io: std.Io,
    allocator: std.mem.Allocator,
) !Object {
    _ = pat_alloc;
    _ = io;
    _ = allocator;
    const default_mat = Material.default();
    var obj = Object{
        .transform = Matrix.identity(),
        .transform_inv = Matrix.identity(),
        .material = default_mat,
        .shape = undefined,
    };

    if (std.mem.eql(u8, type_str, "sphere")) {
        obj.shape = .{ .sphere = .{} };
    } else if (std.mem.eql(u8, type_str, "glass_sphere")) {
        obj.shape = .{ .sphere = .{} };
        obj.material.transparency = 1.0;
        obj.material.refractive_index = 1.5;
    } else if (std.mem.eql(u8, type_str, "plane")) {
        obj.shape = .{ .plane = .{} };
    } else if (std.mem.eql(u8, type_str, "cube")) {
        obj.shape = .{ .cube = .{} };
    } else if (std.mem.eql(u8, type_str, "cylinder")) {
        const min = getF64Default(map.get("minimum"), -std.math.inf(f64));
        const max = getF64Default(map.get("maximum"), std.math.inf(f64));
        const closed = getBool(map.get("closed"), false);
        obj.shape = .{ .cylinder = .{ .minimum = min, .maximum = max, .closed = closed } };
    } else if (std.mem.eql(u8, type_str, "cone")) {
        const min = getF64Default(map.get("minimum"), -std.math.inf(f64));
        const max = getF64Default(map.get("maximum"), std.math.inf(f64));
        const closed = getBool(map.get("closed"), false);
        obj.shape = .{ .cone = .{ .minimum = min, .maximum = max, .closed = closed } };
    } else if (std.mem.eql(u8, type_str, "triangle")) {
        const p1 = try tupleFromList(map.get("p1") orelse return error.MissingP1, true);
        const p2 = try tupleFromList(map.get("p2") orelse return error.MissingP2, true);
        const p3 = try tupleFromList(map.get("p3") orelse return error.MissingP3, true);
        obj.shape = .{ .triangle = objectMod.Triangle.init(p1, p2, p3) };
    } else if (std.mem.eql(u8, type_str, "torus")) {
        const minor_radius = getF64Default(map.get("minor_radius"), 0.25);
        obj.shape = .{ .torus = .{ .minor_radius = minor_radius } };
    } else {
        return error.UnknownShapeType;
    }

    return obj;
}

fn buildGroup(
    val: Value,
    pat_alloc: std.mem.Allocator,
    io: std.Io,
    allocator: std.mem.Allocator,
    db: *ObjectDB,
    base_dir: []const u8,
) anyerror!usize {
    const map = val.asMap() orelse return error.InvalidGroup;
    const children_val = map.get("children") orelse return error.MissingChildren;
    const children_list = children_val.asList() orelse return error.InvalidChildren;

    var child_ids: std.ArrayList(usize) = .empty;
    defer child_ids.deinit(allocator);

    for (children_list) |child_val| {
        const child_map = child_val.asMap() orelse continue;
        if (getBool(child_map.get("hidden"), false)) continue;
        const cid = try buildShape(child_val, pat_alloc, io, allocator, db, base_dir);
        try child_ids.append(allocator, cid);
    }

    const ids_slice = try allocator.dupe(usize, child_ids.items);
    const group_obj = Object{
        .transform = Matrix.identity(),
        .transform_inv = Matrix.identity(),
        .material = Material.default(),
        .shape = .{ .group = .{ .child_ids = ids_slice } },
    };
    const gid = try db.add(group_obj);
    for (ids_slice) |cid| {
        db.getMut(cid).parent_id = gid;
    }
    return gid;
}

fn buildCsg(
    val: Value,
    pat_alloc: std.mem.Allocator,
    io: std.Io,
    allocator: std.mem.Allocator,
    db: *ObjectDB,
    base_dir: []const u8,
) anyerror!usize {
    const map = val.asMap() orelse return error.InvalidCsg;
    const op_str = (map.get("operation") orelse return error.MissingCsgOp).asScalar() orelse return error.InvalidCsgOp;
    const op = parseCsgOp(op_str) orelse return error.UnknownCsgOp;

    const left_val = map.get("left") orelse return error.MissingCsgLeft;
    const right_val = map.get("right") orelse return error.MissingCsgRight;

    const left_id = try buildShape(left_val, pat_alloc, io, allocator, db, base_dir);
    const right_id = try buildShape(right_val, pat_alloc, io, allocator, db, base_dir);

    const csg_obj = Object{
        .transform = Matrix.identity(),
        .transform_inv = Matrix.identity(),
        .material = Material.default(),
        .shape = .{ .csg = .{ .left_id = left_id, .right_id = right_id, .operation = op } },
    };
    const cid = try db.add(csg_obj);
    db.getMut(left_id).parent_id = cid;
    db.getMut(right_id).parent_id = cid;
    return cid;
}

fn parseCsgOp(s: []const u8) ?objectMod.CsgOperation {
    if (std.mem.eql(u8, s, "union")) return .union_op;
    if (std.mem.eql(u8, s, "intersection")) return .intersection_op;
    if (std.mem.eql(u8, s, "difference")) return .difference_op;
    return null;
}

fn buildTransforms(list: []const Value) Matrix {
    var m = Matrix.identity();
    var i: usize = list.len;
    while (i > 0) {
        i -= 1;
        const t = buildSingleTransform(list[i]);
        m = m.mul(t);
    }
    return m;
}

fn buildSingleTransform(val: Value) Matrix {
    const map = val.asMap() orelse return Matrix.identity();
    const type_str = (map.get("type") orelse return Matrix.identity()).asScalar() orelse return Matrix.identity();

    if (std.mem.eql(u8, type_str, "translate")) {
        const amt = map.get("amount") orelse return Matrix.identity();
        const list = amt.asList() orelse return Matrix.identity();
        if (list.len < 3) return Matrix.identity();
        const x = getF64Default(list[0], 0);
        const y = getF64Default(list[1], 0);
        const z = getF64Default(list[2], 0);
        return Matrix.translate(x, y, z);
    } else if (std.mem.eql(u8, type_str, "scale")) {
        const amt = map.get("amount") orelse return Matrix.identity();
        const list = amt.asList() orelse return Matrix.identity();
        if (list.len < 3) return Matrix.identity();
        const x = getF64Default(list[0], 1);
        const y = getF64Default(list[1], 1);
        const z = getF64Default(list[2], 1);
        return Matrix.scale(x, y, z);
    } else if (std.mem.eql(u8, type_str, "rotate")) {
        const angle = degreesToRadians(getF64Default(map.get("angle"), 0));
        const axis_str = (map.get("axis") orelse return Matrix.identity()).asScalar() orelse return Matrix.identity();
        if (std.mem.eql(u8, axis_str, "x")) return Matrix.rotateX(angle);
        if (std.mem.eql(u8, axis_str, "y")) return Matrix.rotateY(angle);
        if (std.mem.eql(u8, axis_str, "z")) return Matrix.rotateZ(angle);
        return Matrix.identity();
    } else if (std.mem.eql(u8, type_str, "shear")) {
        const xy = getF64Default(map.get("xy"), 0);
        const xz = getF64Default(map.get("xz"), 0);
        const yx = getF64Default(map.get("yx"), 0);
        const yz = getF64Default(map.get("yz"), 0);
        const zx = getF64Default(map.get("zx"), 0);
        const zy = getF64Default(map.get("zy"), 0);
        return Matrix.shear(xy, xz, yx, yz, zx, zy);
    }
    return Matrix.identity();
}

fn buildMaterial(val: ?Value, pat_alloc: std.mem.Allocator, io: std.Io, allocator: std.mem.Allocator, base_dir: []const u8) Material {
    var m = Material.default();
    const v = val orelse return m;
    const map = v.asMap() orelse return m;

    m.ambient = getF64Default(map.get("ambient"), 0.1);
    m.diffuse = getF64Default(map.get("diffuse"), 0.9);
    m.specular = getF64Default(map.get("specular"), 0.9);
    m.shininess = getF64Default(map.get("shininess"), 200.0);
    m.reflective = getF64Default(map.get("reflective"), 0.0);
    m.transparency = getF64Default(map.get("transparency"), 0.0);
    m.refractive_index = getF64Default(map.get("refractive_index"), 1.0);

    if (map.get("pattern")) |pv| {
        m.pattern = buildPattern(pv, pat_alloc, io, allocator, base_dir);
    }
    return m;
}

fn buildPattern(val: Value, pat_alloc: std.mem.Allocator, io: std.Io, allocator: std.mem.Allocator, base_dir: []const u8) Pattern {
    const map = val.asMap() orelse return Pattern.solid(Color.white());
    const type_str = (map.get("type") orelse return Pattern.solid(Color.black())).asScalar() orelse return Pattern.solid(Color.black());

    const transforms_val = map.get("transforms");
    const transform = if (transforms_val) |tv|
        if (tv.asList()) |tlist| buildTransforms(tlist) else Matrix.identity()
    else
        Matrix.identity();

    if (std.mem.eql(u8, type_str, "solid")) {
        const color = buildPatternColor(map.get("color"), Color.black());
        return Pattern.solid(color).withTransform(transform);
    } else if (std.mem.eql(u8, type_str, "stripe")) {
        const a = allocSubPattern(map.get("color_a"), map.get("pattern_a"), transform, pat_alloc, io, allocator, base_dir);
        const b = allocSubPattern(map.get("color_b"), map.get("pattern_b"), transform, pat_alloc, io, allocator, base_dir);
        const pa = pat_alloc.create(Pattern) catch return Pattern.solid(Color.black());
        const pb = pat_alloc.create(Pattern) catch return Pattern.solid(Color.black());
        pa.* = a;
        pb.* = b;
        return Pattern{ .kind = .{ .stripe = .{ .a = pa, .b = pb } }, .transform = transform, .transform_inv = transform.inverse() };
    } else if (std.mem.eql(u8, type_str, "gradient")) {
        const a = allocSubPattern(map.get("color_a"), map.get("pattern_a"), transform, pat_alloc, io, allocator, base_dir);
        const b = allocSubPattern(map.get("color_b"), map.get("pattern_b"), transform, pat_alloc, io, allocator, base_dir);
        const pa = pat_alloc.create(Pattern) catch return Pattern.solid(Color.black());
        const pb = pat_alloc.create(Pattern) catch return Pattern.solid(Color.black());
        pa.* = a;
        pb.* = b;
        return Pattern{ .kind = .{ .gradient = .{ .a = pa, .b = pb } }, .transform = transform, .transform_inv = transform.inverse() };
    } else if (std.mem.eql(u8, type_str, "ring")) {
        const a = allocSubPattern(map.get("color_a"), map.get("pattern_a"), transform, pat_alloc, io, allocator, base_dir);
        const b = allocSubPattern(map.get("color_b"), map.get("pattern_b"), transform, pat_alloc, io, allocator, base_dir);
        const pa = pat_alloc.create(Pattern) catch return Pattern.solid(Color.black());
        const pb = pat_alloc.create(Pattern) catch return Pattern.solid(Color.black());
        pa.* = a;
        pb.* = b;
        return Pattern{ .kind = .{ .ring = .{ .a = pa, .b = pb } }, .transform = transform, .transform_inv = transform.inverse() };
    } else if (std.mem.eql(u8, type_str, "checker")) {
        const a = allocSubPattern(map.get("color_a"), map.get("pattern_a"), transform, pat_alloc, io, allocator, base_dir);
        const b = allocSubPattern(map.get("color_b"), map.get("pattern_b"), transform, pat_alloc, io, allocator, base_dir);
        const pa = pat_alloc.create(Pattern) catch return Pattern.solid(Color.black());
        const pb = pat_alloc.create(Pattern) catch return Pattern.solid(Color.black());
        pa.* = a;
        pb.* = b;
        return Pattern{ .kind = .{ .checker = .{ .a = pa, .b = pb } }, .transform = transform, .transform_inv = transform.inverse() };
    } else if (std.mem.eql(u8, type_str, "blend")) {
        const scale = getF64Default(map.get("scale"), 0.5);
        const a = allocSubPattern(map.get("color_a"), map.get("pattern_a"), transform, pat_alloc, io, allocator, base_dir);
        const b = allocSubPattern(map.get("color_b"), map.get("pattern_b"), transform, pat_alloc, io, allocator, base_dir);
        const pa = pat_alloc.create(Pattern) catch return Pattern.solid(Color.black());
        const pb = pat_alloc.create(Pattern) catch return Pattern.solid(Color.black());
        pa.* = a;
        pb.* = b;
        return Pattern{ .kind = .{ .blend = .{ .a = pa, .b = pb, .factor = scale } }, .transform = transform, .transform_inv = transform.inverse() };
    } else if (std.mem.eql(u8, type_str, "perturbed")) {
        const scale = getF64Default(map.get("scale"), 0.2);
        const octaves: usize = @intFromFloat(getF64Default(map.get("octaves"), 3.0));
        const persistence = getF64Default(map.get("persistence"), 0.5);
        const a = allocSubPattern(map.get("color_a"), map.get("pattern_a"), transform, pat_alloc, io, allocator, base_dir);
        const pa = pat_alloc.create(Pattern) catch return Pattern.solid(Color.black());
        pa.* = a;
        return Pattern{ .kind = .{ .perturbed = .{ .inner = pa, .scale = scale, .octaves = octaves, .persistence = persistence } }, .transform = transform, .transform_inv = transform.inverse() };
    } else if (std.mem.eql(u8, type_str, "noise")) {
        const scale = getF64Default(map.get("scale"), 1.0);
        const octaves: usize = @intFromFloat(getF64Default(map.get("octaves"), 1.0));
        const persistence = getF64Default(map.get("persistence"), 1.0);
        const a = allocSubPattern(map.get("color_a"), map.get("pattern_a"), transform, pat_alloc, io, allocator, base_dir);
        const b = allocSubPattern(map.get("color_b"), map.get("pattern_b"), transform, pat_alloc, io, allocator, base_dir);
        const pa = pat_alloc.create(Pattern) catch return Pattern.solid(Color.black());
        const pb = pat_alloc.create(Pattern) catch return Pattern.solid(Color.black());
        pa.* = a;
        pb.* = b;
        return Pattern{ .kind = .{ .noise_pat = .{ .a = pa, .b = pb, .scale = scale, .octaves = octaves, .persistence = persistence } }, .transform = transform, .transform_inv = transform.inverse() };
    } else if (std.mem.eql(u8, type_str, "image")) {
        const file_val = map.get("file") orelse return Pattern.solid(Color.black());
        const rel_path = file_val.asScalar() orelse return Pattern.solid(Color.black());
        const file_path = resolvePath(allocator, base_dir, rel_path) catch return Pattern.solid(Color.black());
        defer allocator.free(file_path);
        const texture = Texture.load(allocator, io, file_path) catch return Pattern.solid(Color.black());
        return Pattern{ .kind = .{ .texture = texture }, .transform = transform, .transform_inv = transform.inverse() };
    }

    return Pattern.solid(Color.black());
}

fn allocSubPattern(
    color_val: ?Value,
    pattern_val: ?Value,
    parent_transform: Matrix,
    pat_alloc: std.mem.Allocator,
    io: std.Io,
    allocator: std.mem.Allocator,
    base_dir: []const u8,
) Pattern {
    if (color_val) |cv| {
        if (cv.asList()) |_| {
            const color = buildPatternColor(color_val, Color.black());
            return Pattern.solid(color).withTransform(parent_transform);
        }
    }
    if (pattern_val) |pv| {
        if (pv.asMap()) |_| {
            return buildPattern(pv, pat_alloc, io, allocator, base_dir);
        }
    }
    return Pattern.solid(Color.black());
}

fn buildPatternColor(val: ?Value, default: Color) Color {
    const v = val orelse return default;
    return colorFromList(v) catch default;
}

fn colorFromList(val: Value) !Color {
    const list = val.asList() orelse return error.NotAList;
    if (list.len < 3) return error.TooShort;
    const r = getF64Default(list[0], 0);
    const g = getF64Default(list[1], 0);
    const b = getF64Default(list[2], 0);
    return Color.init(r, g, b);
}

fn tupleFromList(val: Value, is_point: bool) !Tuple {
    const list = val.asList() orelse return error.NotAList;
    if (list.len < 3) return error.TooShort;
    const x = getF64Default(list[0], 0);
    const y = getF64Default(list[1], 0);
    const z = getF64Default(list[2], 0);
    return if (is_point) Tuple.point(x, y, z) else Tuple.vector(x, y, z);
}

fn getF64(val: Value) !f64 {
    return switch (val) {
        .scalar => |s| std.fmt.parseFloat(f64, s) catch error.InvalidNumber,
        else => error.InvalidNumber,
    };
}

fn getF64Default(val: ?Value, default: f64) f64 {
    const v = val orelse return default;
    return switch (v) {
        .scalar => |s| std.fmt.parseFloat(f64, s) catch default,
        else => default,
    };
}

fn getF64FromMap(map: Map, key: []const u8) !f64 {
    const v = map.get(key) orelse return error.KeyNotFound;
    return getF64(v);
}

fn getBool(val: ?Value, default: bool) bool {
    const v = val orelse return default;
    return switch (v) {
        .boolean => |b| b,
        .scalar => |s| std.mem.eql(u8, s, "true") or std.mem.eql(u8, s, "yes"),
        else => default,
    };
}

fn degreesToRadians(degrees: f64) f64 {
    return degrees * std.math.pi / 180.0;
}
