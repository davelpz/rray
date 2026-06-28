const std = @import("std");

pub fn build(b: *std.Build) void {
    const target = b.standardTargetOptions(.{});
    const optimize = b.option(std.builtin.OptimizeMode, "optimize", "Prioritize performance, safety, or binary size") orelse .ReleaseSafe;

    const zigimg_dep = b.dependency("zigimg", .{ .target = target, .optimize = optimize });
    const yaml_dep = b.dependency("yaml", .{ .target = target, .optimize = optimize });

    const exe_module = b.createModule(.{
        .root_source_file = b.path("src/main.zig"),
        .target = target,
        .optimize = optimize,
    });
    exe_module.addImport("zigimg", zigimg_dep.module("zigimg"));
    exe_module.addImport("yaml", yaml_dep.module("yaml"));

    const exe = b.addExecutable(.{
        .name = "rray",
        .root_module = exe_module,
    });

    b.installArtifact(exe);

    const run_cmd = b.addRunArtifact(exe);
    run_cmd.step.dependOn(b.getInstallStep());
    if (b.args) |args| run_cmd.addArgs(args);
    const run_step = b.step("run", "Run the raytracer");
    run_step.dependOn(&run_cmd.step);
}
