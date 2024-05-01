#![allow(dead_code)]

pub mod matrix {
    pub const EPSILON: f64 = 0.00001;
    use crate::tuple::tuple::Tuple;

    pub struct Matrix {
        pub rows: usize,
        pub cols: usize,
        pub data: Vec<f64>,
    }

    impl Matrix {
        pub fn new(rows: usize, cols: usize) -> Matrix {
            let data = vec![0.0; rows * cols];
            Matrix { rows, cols, data }
        }

        pub fn get(&self, row: usize, col: usize) -> f64 {
            self.data[row * self.cols + col]
        }

        pub fn set(&mut self, row: usize, col: usize, value: f64) {
            self.data[row * self.cols + col] = value;
        }

        pub fn equals(&self, other: &Matrix) -> bool {
            if self.rows != other.rows || self.cols != other.cols {
                return false;
            }
            for i in 0..self.rows {
                for j in 0..self.cols {
                    if (self.get(i, j) - other.get(i, j)).abs() > EPSILON {
                        return false;
                    }
                }
            }
            true
        }

        pub fn multiply(&self, other: &Matrix) -> Matrix {
            let mut result = Matrix::new(self.rows, other.cols);
            for i in 0..self.rows {
                for j in 0..other.cols {
                    let mut sum = 0.0;
                    for k in 0..self.cols {
                        sum += self.get(i, k) * other.get(k, j);
                    }
                    result.set(i, j, sum);
                }
            }
            result
        }

        pub fn multiply_tuple(&self, other: &Tuple) -> Tuple {
            let x = self.get(0, 0) * other.x + self.get(0, 1) * other.y + self.get(0, 2) * other.z + self.get(0, 3) * other.w;
            let y = self.get(1, 0) * other.x + self.get(1, 1) * other.y + self.get(1, 2) * other.z + self.get(1, 3) * other.w;
            let z = self.get(2, 0) * other.x + self.get(2, 1) * other.y + self.get(2, 2) * other.z + self.get(2, 3) * other.w;
            let w = self.get(3, 0) * other.x + self.get(3, 1) * other.y + self.get(3, 2) * other.z + self.get(3, 3) * other.w;
            Tuple::new(x, y, z, w)
        }

        pub fn identity(size: usize) -> Matrix {
            let mut m = Matrix::new(size, size);
            for i in 0..size {
                m.set(i, i, 1.0);
            }
            m
        }

        pub fn transpose(&self) -> Matrix {
            let mut result = Matrix::new(self.cols, self.rows);
            for i in 0..self.rows {
                for j in 0..self.cols {
                    result.set(j, i, self.get(i, j));
                }
            }
            result
        }

        pub fn determinant(&self) -> f64 {
            if self.rows == 2 && self.cols == 2 {
                self.get(0, 0) * self.get(1, 1) - self.get(0, 1) * self.get(1, 0)
            } else {
                let mut det = 0.0;
                for i in 0..self.cols {
                    det += self.get(0, i) * self.cofactor(0, i);
                }
                det
            }
        }

        pub fn submatrix(&self, row: usize, col: usize) -> Matrix {
            let mut result = Matrix::new(self.rows - 1, self.cols - 1);
            let mut r = 0;
            for i in 0..self.rows {
                if i == row {
                    continue;
                }
                let mut c = 0;
                for j in 0..self.cols {
                    if j == col {
                        continue;
                    }
                    result.set(r, c, self.get(i, j));
                    c += 1;
                }
                r += 1;
            }
            result
        }

        pub fn minor(&self, row: usize, col: usize) -> f64 {
            self.submatrix(row, col).determinant()
        }

        pub fn cofactor(&self, row: usize, col: usize) -> f64 {
            let minor = self.minor(row, col);
            if (row + col) % 2 == 0 {
                minor
            } else {
                -minor
            }
        }

        pub fn inverse(&self) -> Matrix {
            let det = self.determinant();
            let mut result = Matrix::new(self.rows, self.cols);
            for i in 0..self.rows {
                for j in 0..self.cols {
                    let c = self.cofactor(i, j);
                    result.set(j, i, c / det);
                }
            }
            result
        }

        pub fn translate(x: f64, y: f64, z: f64) -> Matrix {
            let mut m = Matrix::identity(4);
            m.set(0, 3, x);
            m.set(1, 3, y);
            m.set(2, 3, z);
            m
        }

        pub fn scale(x: f64, y: f64, z: f64) -> Matrix {
            let mut m = Matrix::identity(4);
            m.set(0, 0, x);
            m.set(1, 1, y);
            m.set(2, 2, z);
            m
        }

        pub fn rotate_x(r: f64) -> Matrix {
            let mut m = Matrix::identity(4);
            m.set(1, 1, r.cos());
            m.set(1, 2, -r.sin());
            m.set(2, 1, r.sin());
            m.set(2, 2, r.cos());
            m
        }

        pub fn rotate_y(r: f64) -> Matrix {
            let mut m = Matrix::identity(4);
            m.set(0, 0, r.cos());
            m.set(0, 2, r.sin());
            m.set(2, 0, -r.sin());
            m.set(2, 2, r.cos());
            m
        }

        pub fn rotate_z(r: f64) -> Matrix {
            let mut m = Matrix::identity(4);
            m.set(0, 0, r.cos());
            m.set(0, 1, -r.sin());
            m.set(1, 0, r.sin());
            m.set(1, 1, r.cos());
            m
        }

        pub fn shear(xy: f64, xz: f64, yx: f64, yz: f64, zx: f64, zy: f64) -> Matrix {
            let mut m = Matrix::identity(4);
            m.set(0, 1, xy);
            m.set(0, 2, xz);
            m.set(1, 0, yx);
            m.set(1, 2, yz);
            m.set(2, 0, zx);
            m.set(2, 1, zy);
            m
        }
    }
}

#[cfg(test)]
mod tests {
    use super::matrix::Matrix;
    use crate::tuple::tuple::Tuple;
    use super::matrix::EPSILON;

    #[test]
    fn test_matrix() {
        let mut m = Matrix::new(4, 4);
        assert_eq!(m.get(0, 0), 0.0);
        m.set(0, 0, 1.0);
        assert_eq!(m.get(0, 0), 1.0);
    }

    #[test]
    fn test_matrix_equality() {
        let mut m1 = Matrix::new(4, 4);
        let mut m2 = Matrix::new(4, 4);
        m1.set(0, 0, 1.0);
        m1.set(0, 1, 2.0);
        m2.set(0, 0, 1.0);
        m2.set(0, 1, 2.0);
        assert_eq!(m1.equals(&m2), true);

        m2.set(0, 0, 3.0);
        assert_eq!(m1.equals(&m2), false);

        let m3 = Matrix::new(4, 3);
        assert_eq!(m1.equals(&m3), false);
    }

    #[test]
    fn test_matrix_multiply() {
        let mut m1 = Matrix::new(4, 4);
        let mut m2 = Matrix::new(4, 4);
        m1.set(0, 0, 1.0);
        m1.set(0, 1, 2.0);
        m1.set(0, 2, 3.0);
        m1.set(0, 3, 4.0);
        m1.set(1, 0, 2.0);
        m1.set(1, 1, 3.0);
        m1.set(1, 2, 4.0);
        m1.set(1, 3, 5.0);
        m1.set(2, 0, 3.0);
        m1.set(2, 1, 4.0);
        m1.set(2, 2, 5.0);
        m1.set(2, 3, 6.0);
        m1.set(3, 0, 4.0);
        m1.set(3, 1, 5.0);
        m1.set(3, 2, 6.0);
        m1.set(3, 3, 7.0);
        m2.set(0, 0, 0.0);
        m2.set(0, 1, 1.0);
        m2.set(0, 2, 2.0);
        m2.set(0, 3, 4.0);
        m2.set(1, 0, 1.0);
        m2.set(1, 1, 2.0);
        m2.set(1, 2, 4.0);
        m2.set(1, 3, 8.0);
        m2.set(2, 0, 2.0);
        m2.set(2, 1, 4.0);
        m2.set(2, 2, 8.0);
        m2.set(2, 3, 16.0);
        m2.set(3, 0, 4.0);
        m2.set(3, 1, 8.0);
        m2.set(3, 2, 16.0);
        m2.set(3, 3, 32.0);
        let m3 = m1.multiply(&m2);
        assert_eq!(m3.get(0, 0), 24.0);
        assert_eq!(m3.get(0, 1), 49.0);
        assert_eq!(m3.get(0, 2), 98.0);
        assert_eq!(m3.get(0, 3), 196.0);
        assert_eq!(m3.get(1, 0), 31.0);
        assert_eq!(m3.get(1, 1), 64.0);
        assert_eq!(m3.get(1, 2), 128.0);
        assert_eq!(m3.get(1, 3), 256.0);
        assert_eq!(m3.get(2, 0), 38.0);
        assert_eq!(m3.get(2, 1), 79.0);
        assert_eq!(m3.get(2, 2), 158.0);
        assert_eq!(m3.get(2, 3), 316.0);
        assert_eq!(m3.get(3, 0), 45.0);
        assert_eq!(m3.get(3, 1), 94.0);
        assert_eq!(m3.get(3, 2), 188.0);
        assert_eq!(m3.get(3, 3), 376.0);
    }

    #[test]
    fn test_matrix_tuple_multiply() {
        let mut m = Matrix::new(4, 4);
        m.set(0, 0, 1.0);
        m.set(0, 1, 2.0);
        m.set(0, 2, 3.0);
        m.set(0, 3, 4.0);
        m.set(1, 0, 2.0);
        m.set(1, 1, 4.0);
        m.set(1, 2, 4.0);
        m.set(1, 3, 2.0);
        m.set(2, 0, 8.0);
        m.set(2, 1, 6.0);
        m.set(2, 2, 4.0);
        m.set(2, 3, 1.0);
        m.set(3, 0, 0.0);
        m.set(3, 1, 0.0);
        m.set(3, 2, 0.0);
        m.set(3, 3, 1.0);
        let t = Tuple::new(1.0, 2.0, 3.0, 1.0);
        let result = m.multiply_tuple(&t);
        assert_eq!(result.x, 18.0);
        assert_eq!(result.y, 24.0);
        assert_eq!(result.z, 33.0);
        assert_eq!(result.w, 1.0);
    }

    #[test]
    fn test_matrix_identity() {
        let m = Matrix::identity(4);
        let t = Tuple::new(1.0, 2.0, 3.0, 4.0);
        let result = m.multiply_tuple(&t);
        assert_eq!(result, t);
    }

    #[test]
    fn test_matrix_transpose() {
        let mut m = Matrix::new(4, 4);
        m.set(0, 0, 0.0);
        m.set(0, 1, 9.0);
        m.set(0, 2, 3.0);
        m.set(0, 3, 0.0);
        m.set(1, 0, 9.0);
        m.set(1, 1, 8.0);
        m.set(1, 2, 0.0);
        m.set(1, 3, 8.0);
        m.set(2, 0, 1.0);
        m.set(2, 1, 8.0);
        m.set(2, 2, 5.0);
        m.set(2, 3, 3.0);
        m.set(3, 0, 0.0);
        m.set(3, 1, 0.0);
        m.set(3, 2, 5.0);
        m.set(3, 3, 8.0);
        let m_t = m.transpose();
        assert_eq!(m_t.get(0, 0), 0.0);
        assert_eq!(m_t.get(0, 1), 9.0);
        assert_eq!(m_t.get(0, 2), 1.0);
        assert_eq!(m_t.get(0, 3), 0.0);
        assert_eq!(m_t.get(1, 0), 9.0);
        assert_eq!(m_t.get(1, 1), 8.0);
        assert_eq!(m_t.get(1, 2), 8.0);
        assert_eq!(m_t.get(1, 3), 0.0);
        assert_eq!(m_t.get(2, 0), 3.0);
        assert_eq!(m_t.get(2, 1), 0.0);
        assert_eq!(m_t.get(2, 2), 5.0);
        assert_eq!(m_t.get(2, 3), 5.0);
        assert_eq!(m_t.get(3, 0), 0.0);
        assert_eq!(m_t.get(3, 1), 8.0);
        assert_eq!(m_t.get(3, 2), 3.0);
        assert_eq!(m_t.get(3, 3), 8.0);
    }

    #[test]
    fn test_minor() {
        let mut m = Matrix::new(3, 3);
        m.set(0, 0, 3.0);
        m.set(0, 1, 5.0);
        m.set(0, 2, 0.0);
        m.set(1, 0, 2.0);
        m.set(1, 1, -1.0);
        m.set(1, 2, -7.0);
        m.set(2, 0, 6.0);
        m.set(2, 1, -1.0);
        m.set(2, 2, 5.0);
        let submatrix = m.submatrix(0, 2);
        assert_eq!(submatrix.get(0, 0), 2.0);
        assert_eq!(submatrix.get(0, 1), -1.0);
        assert_eq!(submatrix.get(1, 0), 6.0);
        assert_eq!(submatrix.get(1, 1), -1.0);
        assert_eq!(m.minor(0, 0), -12.0);
        assert_eq!(m.minor(1, 0), 25.0);
        assert_eq!(m.cofactor(0, 0), -12.0);
        assert_eq!(m.cofactor(1, 0), -25.0);
    }

    #[test]
    fn test_matrix_determinant() {
        let mut m = Matrix::new(2, 2);
        m.set(0, 0, 1.0);
        m.set(0, 1, 5.0);
        m.set(1, 0, -3.0);
        m.set(1, 1, 2.0);
        assert_eq!(m.determinant(), 17.0);

        let mut m = Matrix::new(3, 3);
        m.set(0, 0, 1.0);
        m.set(0, 1, 2.0);
        m.set(0, 2, 6.0);
        m.set(1, 0, -5.0);
        m.set(1, 1, 8.0);
        m.set(1, 2, -4.0);
        m.set(2, 0, 2.0);
        m.set(2, 1, 6.0);
        m.set(2, 2, 4.0);
        assert_eq!(m.cofactor(0, 0), 56.0);
        assert_eq!(m.cofactor(0, 1), 12.0);
        assert_eq!(m.minor(1, 0), -28.0);
        assert_eq!(m.cofactor(0, 2), -46.0);
        assert_eq!(m.determinant(), -196.0);

        let mut m = Matrix::new(4, 4);
        m.set(0, 0, -2.0);
        m.set(0, 1, -8.0);
        m.set(0, 2, 3.0);
        m.set(0, 3, 5.0);
        m.set(1, 0, -3.0);
        m.set(1, 1, 1.0);
        m.set(1, 2, 7.0);
        m.set(1, 3, 3.0);
        m.set(2, 0, 1.0);
        m.set(2, 1, 2.0);
        m.set(2, 2, -9.0);
        m.set(2, 3, 6.0);
        m.set(3, 0, -6.0);
        m.set(3, 1, 7.0);
        m.set(3, 2, 7.0);
        m.set(3, 3, -9.0);
        assert_eq!(m.cofactor(0, 0), 690.0);
        assert_eq!(m.cofactor(0, 1), 447.0);
        assert_eq!(m.cofactor(0, 2), 210.0);
        assert_eq!(m.cofactor(0, 3), 51.0);
        assert_eq!(m.determinant(), -4071.0);
    }

    #[test]
    fn test_matrix_inverse() {
        let mut m = Matrix::new(4, 4);
        m.set(0, 0, 6.0);
        m.set(0, 1, 4.0);
        m.set(0, 2, 4.0);
        m.set(0, 3, 4.0);
        m.set(1, 0, 5.0);
        m.set(1, 1, 5.0);
        m.set(1, 2, 7.0);
        m.set(1, 3, 6.0);
        m.set(2, 0, 4.0);
        m.set(2, 1, -9.0);
        m.set(2, 2, 3.0);
        m.set(2, 3, -7.0);
        m.set(3, 0, 9.0);
        m.set(3, 1, 1.0);
        m.set(3, 2, 7.0);
        m.set(3, 3, -6.0);

        assert_eq!(m.determinant(), -2120.0);

        m.set(0, 0, -5.0);
        m.set(0, 1, 2.0);
        m.set(0, 2, 6.0);
        m.set(0, 3, -8.0);
        m.set(1, 0, 1.0);
        m.set(1, 1, -5.0);
        m.set(1, 2, 1.0);
        m.set(1, 3, 8.0);
        m.set(2, 0, 7.0);
        m.set(2, 1, 7.0);
        m.set(2, 2, -6.0);
        m.set(2, 3, -7.0);
        m.set(3, 0, 1.0);
        m.set(3, 1, -3.0);
        m.set(3, 2, 7.0);
        m.set(3, 3, 4.0);
        let m_inv = m.inverse();
        assert_eq!(m.determinant(), 532.0);
        assert_eq!(m.cofactor(2, 3), -160.0);
        assert_eq!(m_inv.get(3, 2), -160.0 / 532.0);
        assert_eq!(m.cofactor(3, 2), 105.0);
        assert_eq!(m_inv.get(2, 3), 105.0 / 532.0);
        assert_eq!((m_inv.get(0, 0) - 0.21805).abs() < EPSILON, true);
        assert_eq!((m_inv.get(0, 1) - 0.45113).abs() < EPSILON, true);
        assert_eq!((m_inv.get(0, 2) - 0.24060).abs() < EPSILON, true);
        assert_eq!((m_inv.get(0, 3) - -0.04511).abs() < EPSILON, true);

        assert_eq!((m_inv.get(1, 0) - -0.80827).abs() < EPSILON, true);
        assert_eq!((m_inv.get(1, 1) - -1.45677).abs() < EPSILON, true);
        assert_eq!((m_inv.get(1, 2) - -0.44361).abs() < EPSILON, true);
        assert_eq!((m_inv.get(1, 3) - 0.52068).abs() < EPSILON, true);

        assert_eq!((m_inv.get(2, 0) - -0.07895).abs() < EPSILON, true);
        assert_eq!((m_inv.get(2, 1) - -0.22368).abs() < EPSILON, true);
        assert_eq!((m_inv.get(2, 2) - -0.05263).abs() < EPSILON, true);
        assert_eq!((m_inv.get(2, 3) - 0.19737).abs() < EPSILON, true);

        assert_eq!((m_inv.get(3, 0) - -0.52256).abs() < EPSILON, true);
        assert_eq!((m_inv.get(3, 1) - -0.81391).abs() < EPSILON, true);
        assert_eq!((m_inv.get(3, 2) - -0.30075).abs() < EPSILON, true);
        assert_eq!((m_inv.get(3, 3) - 0.30639).abs() < EPSILON, true);

        let c = m.multiply(&m_inv);
        assert_eq!(c.equals(&Matrix::identity(4)), true);
    }

    #[test]
    fn test_matrix_translate() {
        let t = Matrix::translate(5.0, -3.0, 2.0);
        let p = Tuple::point(-3.0, 4.0, 5.0);
        let result = t.multiply_tuple(&p);
        assert_eq!(result, Tuple::point(2.0, 1.0, 7.0));

        let t_inv = t.inverse();
        let p = Tuple::point(-3.0, 4.0, 5.0);
        let result = t_inv.multiply_tuple(&p);
        assert_eq!(result, Tuple::point(-8.0, 7.0, 3.0));

        let v = Tuple::vector(-3.0, 4.0, 5.0);
        let result = t.multiply_tuple(&v);
        assert_eq!(result, v);
    }

    #[test]
    fn test_matrix_scale() {
        let t = Matrix::scale(2.0, 3.0, 4.0);
        let p = Tuple::point(-4.0, 6.0, 8.0);
        let result = t.multiply_tuple(&p);
        assert_eq!(result, Tuple::point(-8.0, 18.0, 32.0));

        let v = Tuple::vector(-4.0, 6.0, 8.0);
        let result = t.multiply_tuple(&v);
        assert_eq!(result, Tuple::vector(-8.0, 18.0, 32.0));

        let t_inv = t.inverse();
        let v = Tuple::vector(-4.0, 6.0, 8.0);
        let result = t_inv.multiply_tuple(&v);
        assert_eq!(result, Tuple::vector(-2.0, 2.0, 2.0));

        let t = Matrix::scale(-1.0, 1.0, 1.0);
        let p = Tuple::point(2.0, 3.0, 4.0);
        let result = t.multiply_tuple(&p);
        assert_eq!(result, Tuple::point(-2.0, 3.0, 4.0));
    }

    #[test]
    fn test_matrix_rotate_x() {
        let p = Tuple::point(0.0, 1.0, 0.0);
        let half_quarter = Matrix::rotate_x(std::f64::consts::FRAC_PI_4);
        let full_quarter = Matrix::rotate_x(std::f64::consts::FRAC_PI_2);
        let result = half_quarter.multiply_tuple(&p);
        assert_eq!(result, Tuple::point(0.0, 2.0_f64.sqrt() / 2.0, 2.0_f64.sqrt() / 2.0));
        let result = full_quarter.multiply_tuple(&p);
        assert_eq!(result, Tuple::point(0.0, 0.0, 1.0));
    }

    #[test]
    fn test_matrix_rotate_y() {
        let p = Tuple::point(0.0, 0.0, 1.0);
        let half_quarter = Matrix::rotate_y(std::f64::consts::FRAC_PI_4);
        let full_quarter = Matrix::rotate_y(std::f64::consts::FRAC_PI_2);
        let result = half_quarter.multiply_tuple(&p);
        assert_eq!(result, Tuple::point(2.0_f64.sqrt() / 2.0, 0.0, 2.0_f64.sqrt() / 2.0));
        let result = full_quarter.multiply_tuple(&p);
        assert_eq!(result, Tuple::point(1.0, 0.0, 0.0));
    }

    #[test]
    fn test_matrix_rotate_z() {
        let p = Tuple::point(0.0, 1.0, 0.0);
        let half_quarter = Matrix::rotate_z(std::f64::consts::FRAC_PI_4);
        let full_quarter = Matrix::rotate_z(std::f64::consts::FRAC_PI_2);
        let result = half_quarter.multiply_tuple(&p);
        assert_eq!(result, Tuple::point(-2.0_f64.sqrt() / 2.0, 2.0_f64.sqrt() / 2.0, 0.0));
        let result = full_quarter.multiply_tuple(&p);
        assert_eq!(result, Tuple::point(-1.0, 0.0, 0.0));
    }

    #[test]
    fn test_matrix_shear() {
        let t = Matrix::shear(1.0, 0.0, 0.0, 0.0, 0.0, 0.0);
        let p = Tuple::point(2.0, 3.0, 4.0);
        let result = t.multiply_tuple(&p);
        assert_eq!(result, Tuple::point(5.0, 3.0, 4.0));

        let t = Matrix::shear(0.0, 1.0, 0.0, 0.0, 0.0, 0.0);
        let p = Tuple::point(2.0, 3.0, 4.0);
        let result = t.multiply_tuple(&p);
        assert_eq!(result, Tuple::point(6.0, 3.0, 4.0));

        let t = Matrix::shear(0.0, 0.0, 1.0, 0.0, 0.0, 0.0);
        let p = Tuple::point(2.0, 3.0, 4.0);
        let result = t.multiply_tuple(&p);
        assert_eq!(result, Tuple::point(2.0, 5.0, 4.0));

        let t = Matrix::shear(0.0, 0.0, 0.0, 1.0, 0.0, 0.0);
        let p = Tuple::point(2.0, 3.0, 4.0);
        let result = t.multiply_tuple(&p);
        assert_eq!(result, Tuple::point(2.0, 7.0, 4.0));

        let t = Matrix::shear(0.0, 0.0, 0.0, 0.0, 1.0, 0.0);
        let p = Tuple::point(2.0, 3.0, 4.0);
        let result = t.multiply_tuple(&p);
        assert_eq!(result, Tuple::point(2.0, 3.0, 6.0));

        let t = Matrix::shear(0.0, 0.0, 0.0, 0.0, 0.0, 1.0);
        let p = Tuple::point(2.0, 3.0, 4.0);
        let result = t.multiply_tuple(&p);
        assert_eq!(result, Tuple::point(2.0, 3.0, 7.0));
    }
}