const std = @import("std");
const zigimg = @import("zigimg");
const Color = @import("../math/color.zig").Color;

pub const Texture = struct {
    width: u32,
    height: u32,
    pixels: []u8, // RGBA8, length = width * height * 4
    allocator: std.mem.Allocator,

    pub fn load(allocator: std.mem.Allocator, io: std.Io, path: []const u8) !Texture {
        var read_buf: [65536]u8 = undefined;
        var image = try zigimg.Image.fromFilePath(allocator, io, path, &read_buf);
        defer image.deinit(allocator);

        const w: u32 = @intCast(image.width);
        const h: u32 = @intCast(image.height);
        const pixels = try allocator.alloc(u8, w * h * 4);

        var i: usize = 0;
        var it = image.iterator();
        while (it.next()) |pixel| {
            // iterator yields Colorf32 with .r/.g/.b/.a as f32 in [0,1]
            pixels[i] = @intFromFloat(@min(@max(pixel.r * 255.0, 0.0), 255.0));
            pixels[i + 1] = @intFromFloat(@min(@max(pixel.g * 255.0, 0.0), 255.0));
            pixels[i + 2] = @intFromFloat(@min(@max(pixel.b * 255.0, 0.0), 255.0));
            pixels[i + 3] = @intFromFloat(@min(@max(pixel.a * 255.0, 0.0), 255.0));
            i += 4;
        }

        return .{ .width = w, .height = h, .pixels = pixels, .allocator = allocator };
    }

    pub fn deinit(self: *Texture) void {
        self.allocator.free(self.pixels);
    }

    pub fn getColor(self: Texture, u: f64, v: f64) [4]u8 {
        const uc = std.math.clamp(u, 0.0, 1.0);
        const vc = std.math.clamp(v, 0.0, 1.0);
        const x: u32 = @intCast(@min(@as(u32, @intFromFloat(uc * @as(f64, @floatFromInt(self.width)))), self.width - 1));
        const y_raw: u32 = @intCast(@min(@as(u32, @intFromFloat(vc * @as(f64, @floatFromInt(self.height)))), self.height - 1));
        const y = self.height - y_raw - 1;
        const base = (y * self.width + x) * 4;
        return .{ self.pixels[base], self.pixels[base + 1], self.pixels[base + 2], self.pixels[base + 3] };
    }

    pub fn sampleTexture(self: Texture, u: f64, v: f64) Color {
        const rgba = self.getColor(u, v);
        return Color.init(
            @as(f64, @floatFromInt(rgba[0])) / 255.0,
            @as(f64, @floatFromInt(rgba[1])) / 255.0,
            @as(f64, @floatFromInt(rgba[2])) / 255.0,
        );
    }
};
