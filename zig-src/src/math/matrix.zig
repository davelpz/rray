const std = @import("std");
const EPSILON = @import("../main.zig").EPSILON;
const Tuple = @import("tuple.zig").Tuple;

pub const Matrix = struct {
    data: [4][4]f64,

    pub fn identity() Matrix {
        return .{ .data = .{
            .{ 1, 0, 0, 0 },
            .{ 0, 1, 0, 0 },
            .{ 0, 0, 1, 0 },
            .{ 0, 0, 0, 1 },
        } };
    }

    pub fn zero() Matrix {
        return .{ .data = std.mem.zeroes([4][4]f64) };
    }

    pub fn get(self: Matrix, row: usize, col: usize) f64 {
        return self.data[row][col];
    }

    pub fn eql(self: Matrix, other: Matrix) bool {
        for (0..4) |r| {
            for (0..4) |c| {
                if (@abs(self.data[r][c] - other.data[r][c]) >= EPSILON) return false;
            }
        }
        return true;
    }

    pub fn mul(self: Matrix, other: Matrix) Matrix {
        var result = zero();
        for (0..4) |r| {
            for (0..4) |c| {
                var sum: f64 = 0;
                for (0..4) |k| {
                    sum += self.data[r][k] * other.data[k][c];
                }
                result.data[r][c] = sum;
            }
        }
        return result;
    }

    pub fn mulTuple(self: Matrix, t: Tuple) Tuple {
        const x = self.data[0][0] * t.x + self.data[0][1] * t.y + self.data[0][2] * t.z + self.data[0][3] * t.w;
        const y = self.data[1][0] * t.x + self.data[1][1] * t.y + self.data[1][2] * t.z + self.data[1][3] * t.w;
        const z = self.data[2][0] * t.x + self.data[2][1] * t.y + self.data[2][2] * t.z + self.data[2][3] * t.w;
        const w = self.data[3][0] * t.x + self.data[3][1] * t.y + self.data[3][2] * t.z + self.data[3][3] * t.w;
        return .{ .x = x, .y = y, .z = z, .w = w };
    }

    pub fn transpose(self: Matrix) Matrix {
        var result = zero();
        for (0..4) |r| {
            for (0..4) |c| {
                result.data[c][r] = self.data[r][c];
            }
        }
        return result;
    }

    fn submatrix3(self: Matrix, skip_row: usize, skip_col: usize) [3][3]f64 {
        var result: [3][3]f64 = undefined;
        var rr: usize = 0;
        for (0..4) |r| {
            if (r == skip_row) continue;
            var cc: usize = 0;
            for (0..4) |c| {
                if (c == skip_col) continue;
                result[rr][cc] = self.data[r][c];
                cc += 1;
            }
            rr += 1;
        }
        return result;
    }

    fn det3(m: [3][3]f64) f64 {
        return m[0][0] * (m[1][1] * m[2][2] - m[1][2] * m[2][1]) -
            m[0][1] * (m[1][0] * m[2][2] - m[1][2] * m[2][0]) +
            m[0][2] * (m[1][0] * m[2][1] - m[1][1] * m[2][0]);
    }

    fn cofactor(self: Matrix, row: usize, col: usize) f64 {
        const sub = self.submatrix3(row, col);
        const minor = det3(sub);
        return if ((row + col) % 2 == 0) minor else -minor;
    }

    pub fn determinant(self: Matrix) f64 {
        var sum: f64 = 0;
        for (0..4) |c| {
            sum += self.data[0][c] * self.cofactor(0, c);
        }
        return sum;
    }

    pub fn inverse(self: Matrix) Matrix {
        const det = self.determinant();
        var result = zero();
        for (0..4) |r| {
            for (0..4) |c| {
                result.data[c][r] = self.cofactor(r, c) / det;
            }
        }
        return result;
    }

    pub fn translate(x: f64, y: f64, z: f64) Matrix {
        var m = identity();
        m.data[0][3] = x;
        m.data[1][3] = y;
        m.data[2][3] = z;
        return m;
    }

    pub fn scale(x: f64, y: f64, z: f64) Matrix {
        var m = identity();
        m.data[0][0] = x;
        m.data[1][1] = y;
        m.data[2][2] = z;
        return m;
    }

    pub fn rotateX(radians: f64) Matrix {
        var m = identity();
        m.data[1][1] = @cos(radians);
        m.data[1][2] = -@sin(radians);
        m.data[2][1] = @sin(radians);
        m.data[2][2] = @cos(radians);
        return m;
    }

    pub fn rotateY(radians: f64) Matrix {
        var m = identity();
        m.data[0][0] = @cos(radians);
        m.data[0][2] = @sin(radians);
        m.data[2][0] = -@sin(radians);
        m.data[2][2] = @cos(radians);
        return m;
    }

    pub fn rotateZ(radians: f64) Matrix {
        var m = identity();
        m.data[0][0] = @cos(radians);
        m.data[0][1] = -@sin(radians);
        m.data[1][0] = @sin(radians);
        m.data[1][1] = @cos(radians);
        return m;
    }

    pub fn shear(xy: f64, xz: f64, yx: f64, yz: f64, zx: f64, zy: f64) Matrix {
        var m = identity();
        m.data[0][1] = xy;
        m.data[0][2] = xz;
        m.data[1][0] = yx;
        m.data[1][2] = yz;
        m.data[2][0] = zx;
        m.data[2][1] = zy;
        return m;
    }

    pub fn viewTransform(from: Tuple, to: Tuple, up: Tuple) Matrix {
        const forward = to.sub(from).normalize();
        const left = forward.cross(up.normalize());
        const true_up = left.cross(forward);

        const orientation = Matrix{ .data = .{
            .{ left.x,     left.y,     left.z,     0 },
            .{ true_up.x,  true_up.y,  true_up.z,  0 },
            .{ -forward.x, -forward.y, -forward.z, 0 },
            .{ 0,          0,          0,           1 },
        } };
        return orientation.mul(translate(-from.x, -from.y, -from.z));
    }
};

test "matrix multiply" {
    const t = std.testing;
    const a = Matrix{ .data = .{
        .{ 1, 2, 3, 4 },
        .{ 5, 6, 7, 8 },
        .{ 9, 8, 7, 6 },
        .{ 5, 4, 3, 2 },
    } };
    const b = Matrix{ .data = .{
        .{ -2, 1, 2, 3 },
        .{ 3, 2, 1, -1 },
        .{ 4, 3, 6, 5 },
        .{ 1, 2, 7, 8 },
    } };
    const expected = Matrix{ .data = .{
        .{ 20, 22, 50, 48 },
        .{ 44, 54, 114, 108 },
        .{ 40, 58, 110, 102 },
        .{ 16, 26, 46, 42 },
    } };
    try t.expect(a.mul(b).eql(expected));
}

test "matrix inverse" {
    const t = std.testing;
    const a = Matrix{ .data = .{
        .{ -5, 2, 6, -8 },
        .{ 1, -5, 1, 8 },
        .{ 7, 7, -6, -7 },
        .{ 1, -3, 7, 4 },
    } };
    const inv = a.inverse();
    try t.expect(@abs(inv.data[0][0] - 0.21805) < 0.0001);
    try t.expect(@abs(inv.data[1][0] - 0.10526) < 0.0001);
    // roundtrip
    try t.expect(a.mul(inv).eql(Matrix.identity()));
}
