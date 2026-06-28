const std = @import("std");
const Matrix = @import("../math/matrix.zig").Matrix;
const Tuple = @import("../math/tuple.zig").Tuple;
const Ray = @import("ray.zig").Ray;
const Scene = @import("scene.zig").Scene;
const Canvas = @import("canvas.zig").Canvas;

pub const Camera = struct {
    hsize: usize,
    vsize: usize,
    field_of_view: f64,
    transform: Matrix,
    transform_inv: Matrix,
    pixel_size: f64,
    half_width: f64,
    half_height: f64,

    pub fn new(hsize: usize, vsize: usize, fov: f64) Camera {
        const half_view = @tan(fov / 2.0);
        const aspect = @as(f64, @floatFromInt(hsize)) / @as(f64, @floatFromInt(vsize));
        const half_width = if (aspect >= 1.0) half_view else half_view * aspect;
        const half_height = if (aspect >= 1.0) half_view / aspect else half_view;
        const pixel_size = (half_width * 2.0) / @as(f64, @floatFromInt(hsize));
        const t = Matrix.identity();
        return .{
            .hsize = hsize,
            .vsize = vsize,
            .field_of_view = fov,
            .transform = t,
            .transform_inv = t,
            .pixel_size = pixel_size,
            .half_width = half_width,
            .half_height = half_height,
        };
    }

    pub fn setTransform(self: *Camera, t: Matrix) void {
        self.transform = t;
        self.transform_inv = t.inverse();
    }

    pub fn rayForPixel(self: Camera, px: usize, py: usize) Ray {
        const xoffset = (@as(f64, @floatFromInt(px)) + 0.5) * self.pixel_size;
        const yoffset = (@as(f64, @floatFromInt(py)) + 0.5) * self.pixel_size;
        const world_x = self.half_width - xoffset;
        const world_y = self.half_height - yoffset;
        const pixel = self.transform_inv.mulTuple(Tuple.point(world_x, world_y, -1.0));
        const origin = self.transform_inv.mulTuple(Tuple.point(0.0, 0.0, 0.0));
        const direction = pixel.sub(origin).normalize();
        return Ray.init(origin, direction);
    }

    pub fn render(
        self: Camera,
        scene: *const Scene,
        io: std.Io,
        path: []const u8,
        aa: usize,
        allocator: std.mem.Allocator,
    ) !void {
        var canvas = try Canvas.init(self.hsize, self.vsize, allocator);
        defer canvas.deinit();

        const cpu_count = std.Thread.getCpuCount() catch 4;
        const thread_count = @max(1, cpu_count);

        var row_counter = std.atomic.Value(usize).init(0);
        var done_counter = std.atomic.Value(usize).init(0);

        const WorkCtx = struct {
            camera: *const Camera,
            scene: *const Scene,
            canvas: *Canvas,
            row_counter: *std.atomic.Value(usize),
            done_counter: *std.atomic.Value(usize),
            allocator: std.mem.Allocator,
        };

        const workerFn = struct {
            fn run(ctx: *const WorkCtx) void {
                var arena = std.heap.ArenaAllocator.init(ctx.allocator);
                defer arena.deinit();

                while (true) {
                    const y = ctx.row_counter.fetchAdd(1, .monotonic);
                    if (y >= ctx.camera.vsize) break;
                    _ = arena.reset(.retain_capacity);
                    const row_alloc = arena.allocator();

                    for (0..ctx.camera.hsize) |x| {
                        const ray = ctx.camera.rayForPixel(x, y);
                        const color = ctx.scene.colorAt(ray, 5, row_alloc) catch continue;
                        ctx.canvas.writePixel(x, y, color);
                    }
                    _ = ctx.done_counter.fetchAdd(1, .monotonic);
                }
            }
        }.run;

        const work_ctx = WorkCtx{
            .camera = &self,
            .scene = scene,
            .canvas = &canvas,
            .row_counter = &row_counter,
            .done_counter = &done_counter,
            .allocator = allocator,
        };

        const threads = try allocator.alloc(std.Thread, thread_count);
        defer allocator.free(threads);

        for (threads) |*t| {
            t.* = try std.Thread.spawn(.{}, workerFn, .{&work_ctx});
        }

        // Print progress while threads run
        const total_rows = self.vsize;
        while (true) {
            const done = done_counter.load(.monotonic);
            std.debug.print("\r{d}/{d} rows", .{ done, total_rows });
            if (done >= total_rows) break;
            std.Io.sleep(io, .{ .nanoseconds = 100 * std.time.ns_per_ms }, .awake) catch {};
        }
        std.debug.print("\n", .{});

        for (threads) |t| t.join();

        try canvas.writeToPng(io, path, aa, allocator);
    }
};
