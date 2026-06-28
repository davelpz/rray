const std = @import("std");
const noise = @import("raytracer/noise.zig");
const scene_builder_yaml = @import("raytracer/scene_builder_yaml.zig");

pub const EPSILON: f64 = 0.00001;

fn printHelp() void {
    std.debug.print(
        \\rray - A simple raytracer
        \\
        \\Usage: rray [OPTIONS] -s <SCENE>
        \\
        \\Options:
        \\  -W <WIDTH>   Width of the generated image (default: 800)
        \\  -H <HEIGHT>  Height of the generated image (default: 600)
        \\  -s <SCENE>   Scene file in YAML format (required)
        \\  -o <OUTPUT>  Output PNG file (default: output.png)
        \\  -a <AA>      Anti-aliasing level 1-5 (default: 1)
        \\  -h, --help   Print this help message
        \\
    , .{});
}

pub fn main(proc: std.process.Init) !void {
    var width: usize = 800;
    var height: usize = 600;
    var scene: ?[]const u8 = null;
    var output: []const u8 = "output.png";
    var aa: usize = 1;

    var args = std.process.Args.Iterator.init(proc.minimal.args);
    _ = args.skip(); // skip program name

    while (args.next()) |arg| {
        if (std.mem.eql(u8, arg, "-h") or std.mem.eql(u8, arg, "--help")) {
            printHelp();
            return;
        } else if (std.mem.eql(u8, arg, "-W")) {
            const val = args.next() orelse {
                std.debug.print("Error: -W requires a value\n", .{});
                return error.InvalidArgs;
            };
            width = std.fmt.parseUnsigned(usize, val, 10) catch {
                std.debug.print("Error: invalid width '{s}'\n", .{val});
                return error.InvalidArgs;
            };
        } else if (std.mem.eql(u8, arg, "-H")) {
            const val = args.next() orelse {
                std.debug.print("Error: -H requires a value\n", .{});
                return error.InvalidArgs;
            };
            height = std.fmt.parseUnsigned(usize, val, 10) catch {
                std.debug.print("Error: invalid height '{s}'\n", .{val});
                return error.InvalidArgs;
            };
        } else if (std.mem.eql(u8, arg, "-s")) {
            const val = args.next() orelse {
                std.debug.print("Error: -s requires a value\n", .{});
                return error.InvalidArgs;
            };
            scene = val;
        } else if (std.mem.eql(u8, arg, "-o")) {
            const val = args.next() orelse {
                std.debug.print("Error: -o requires a value\n", .{});
                return error.InvalidArgs;
            };
            output = val;
        } else if (std.mem.eql(u8, arg, "-a")) {
            const val = args.next() orelse {
                std.debug.print("Error: -a requires a value\n", .{});
                return error.InvalidArgs;
            };
            aa = std.fmt.parseUnsigned(usize, val, 10) catch {
                std.debug.print("Error: invalid aa level '{s}'\n", .{val});
                return error.InvalidArgs;
            };
            if (aa == 0 or aa > 5) {
                std.debug.print("Error: aa must be between 1 and 5\n", .{});
                return error.InvalidArgs;
            }
        } else {
            std.debug.print("Error: unknown argument '{s}'\n", .{arg});
            return error.InvalidArgs;
        }
    }

    const scene_file = scene orelse {
        std.debug.print("Error: -s <scene.yaml> is required\n", .{});
        return error.InvalidArgs;
    };

    noise.initNoise();

    try scene_builder_yaml.renderSceneFromFile(
        scene_file,
        width,
        height,
        output,
        aa,
        proc.io,
        proc.gpa,
    );
}
