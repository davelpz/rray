const std = @import("std");
const zigimg = @import("zigimg");
const Color = @import("../math/color.zig").Color;

pub const Canvas = struct {
    width: usize,
    height: usize,
    pixels: []Color,
    allocator: std.mem.Allocator,

    pub fn init(width: usize, height: usize, allocator: std.mem.Allocator) !Canvas {
        const pixels = try allocator.alloc(Color, width * height);
        @memset(pixels, Color.black());
        return .{ .width = width, .height = height, .pixels = pixels, .allocator = allocator };
    }

    pub fn deinit(self: *Canvas) void {
        self.allocator.free(self.pixels);
    }

    pub fn writePixel(self: *Canvas, x: usize, y: usize, color: Color) void {
        self.pixels[y * self.width + x] = color;
    }

    pub fn pixelAt(self: Canvas, x: usize, y: usize) Color {
        return self.pixels[y * self.width + x];
    }

    pub fn writeToPng(self: Canvas, io: std.Io, path: []const u8, aa: usize, allocator: std.mem.Allocator) !void {
        const out_w = self.width / aa;
        const out_h = self.height / aa;
        var image = try zigimg.Image.create(allocator, out_w, out_h, .rgba32);
        defer image.deinit(allocator);

        const total: f64 = @floatFromInt(aa * aa);
        for (0..out_h) |oy| {
            for (0..out_w) |ox| {
                var r: f64 = 0;
                var g: f64 = 0;
                var b: f64 = 0;
                for (0..aa) |dy| {
                    for (0..aa) |dx| {
                        const c = self.pixelAt(ox * aa + dx, oy * aa + dy);
                        r += c.r;
                        g += c.g;
                        b += c.b;
                    }
                }
                r = std.math.clamp(r / total, 0.0, 1.0);
                g = std.math.clamp(g / total, 0.0, 1.0);
                b = std.math.clamp(b / total, 0.0, 1.0);
                image.pixels.rgba32[oy * out_w + ox] = .{
                    .r = @intFromFloat(r * 255.0),
                    .g = @intFromFloat(g * 255.0),
                    .b = @intFromFloat(b * 255.0),
                    .a = 255,
                };
            }
        }

        var write_buf: [65536]u8 = undefined;
        try image.writeToFilePath(allocator, io, path, &write_buf, .{ .png = .{} });
    }
};
