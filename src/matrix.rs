#![allow(dead_code)]

use crate::EPSILON;
use crate::tuple::Tuple;
use std::ops::Mul;
use std::sync::Mutex;

/// Represents a matrix with numerical data and operations for linear algebra.
///
/// This structure holds the dimensions of the matrix (rows and columns) and the matrix data itself.
/// It also caches the inverse of the matrix for efficiency, using a thread-safe `Mutex`.
///
/// # Fields
///
/// * `rows` - The number of rows in the matrix.
/// * `cols` - The number of columns in the matrix.
/// * `data` - A flat `Vec<f64>` storing the matrix data in row-major order.
/// * `inverse_cache` - A thread-safe `Mutex` wrapping an `Option` that may contain the cached inverse of the matrix.
#[derive(Debug)]
pub struct Matrix {
    pub rows: usize,
    pub cols: usize,
    pub data: Vec<f64>,
    inverse_cache: Mutex<Option<Box<Matrix>>>,
}

impl Clone for Matrix {
    /// Creates a clone of the `Matrix` instance.
    ///
    /// This method provides a deep copy of the matrix, including a new copy of its data.
    /// The `inverse_cache` is reinitialized as empty because the cached inverse does not necessarily
    /// apply to the cloned instance.
    ///
    /// # Returns
    ///
    /// A new `Matrix` instance with the same data as the original but with an uninitialized inverse cache.
    fn clone(&self) -> Self {
        let data = self.data.clone();
        Matrix {
            rows: self.rows,
            cols: self.cols,
            data,
            inverse_cache: Mutex::new(None), // Initialize a new Mutex for the clone
        }
    }
}

impl PartialEq for Matrix {
    /// Implements the equality comparison for `Matrix` instances.
    ///
    /// This method delegates the comparison to the `equals` method of `Matrix`,
    /// allowing for a custom definition of equality that can include tolerances
    /// for floating-point comparisons or other criteria specific to `Matrix`.
    ///
    /// # Arguments
    ///
    /// * `other` - A reference to another `Matrix` instance to compare with `self`.
    ///
    /// # Returns
    ///
    /// * `true` if the matrices are considered equal,
    /// * `false` otherwise.
    fn eq(&self, other: &Self) -> bool {
        self.equals(other)
    }
}

/// The `Matrix` struct and its associated methods.
///
/// This implementation covers a variety of operations essential for working with matrices in the context of computer graphics,
/// particularly in ray tracing. It includes basic matrix creation, manipulation (such as getting and setting individual elements),
/// and more complex operations like matrix multiplication, calculating determinants, submatrices, minors, cofactors, and inverses.
/// Additionally, it provides methods for transforming points and vectors in 3D space, including translations, scaling, rotations,
/// shearing, and constructing a view transformation matrix.
///
/// # Examples
///
/// Basic usage:
///
/// ```
/// let m = Matrix::new(4, 4); // Creates a 4x4 matrix filled with zeros.
/// let identity = Matrix::identity(4); // Creates a 4x4 identity matrix.
/// let tuple = Tuple::new(1.0, 2.0, 3.0, 1.0);
/// let transformed = identity.multiply_tuple(&tuple); // Transforms a tuple using the identity matrix.
/// ```
///
/// Advanced transformations:
///
/// ```
/// let rotate = Matrix::rotate_x(std::f64::consts::PI / 4.0);
/// let scaled = Matrix::scale(2.0, 3.0, 4.0);
/// let translated = Matrix::translate(5.0, -3.0, 2.0);
/// ```
impl Matrix {
    /// Creates a new `Matrix` instance with specified dimensions.
    ///
    /// Initializes a matrix of the given size, filled with zeros. This function also initializes
    /// the inverse cache as empty, indicating that the inverse of the matrix has not yet been calculated.
    ///
    /// # Arguments
    ///
    /// * `rows` - The number of rows in the matrix.
    /// * `cols` - The number of columns in the matrix.
    ///
    /// # Returns
    ///
    /// A new `Matrix` instance with all elements set to 0.0 and an empty inverse cache.
    pub fn new(rows: usize, cols: usize) -> Matrix {
        let data = vec![0.0; rows * cols];
        Matrix { rows, cols, data, inverse_cache: Mutex::new(None)}
    }

    /// Retrieves the value at the specified row and column in the matrix.
    ///
    /// This method calculates the index into the flat data vector using the row and column numbers,
    /// assuming row-major order of the matrix elements. It then returns the value at that index.
    ///
    /// # Arguments
    ///
    /// * `row` - The zero-based row index of the desired element.
    /// * `col` - The zero-based column index of the desired element.
    ///
    /// # Returns
    ///
    /// The `f64` value at the specified row and column.
    pub fn get(&self, row: usize, col: usize) -> f64 {
        self.data[row * self.cols + col]
    }

    /// Sets the value at a specified row and column in the matrix.
    ///
    /// This method directly modifies the matrix's data, updating the value at the given row and column indices.
    /// It calculates the index in the flat data vector corresponding to the row and column, then updates that position
    /// with the new value. This operation does not check bounds, so it's the caller's responsibility to ensure
    /// the row and column are within the matrix's dimensions.
    ///
    /// # Arguments
    ///
    /// * `row` - The zero-based row index where the value will be set.
    /// * `col` - The zero-based column index where the value will be set.
    /// * `value` - The new value to set at the specified row and column.
    pub fn set(&mut self, row: usize, col: usize, value: f64) {
        self.data[row * self.cols + col] = value;
    }

    /// Compares the current matrix with another matrix for equality.
    ///
    /// This method checks if two matrices are equal by comparing their dimensions
    /// and then each corresponding element. Two elements are considered equal if
    /// the absolute difference between them is less than or equal to `EPSILON`,
    /// which accounts for floating-point imprecision.
    ///
    /// # Arguments
    ///
    /// * `other` - A reference to another matrix to compare against.
    ///
    /// # Returns
    ///
    /// Returns `true` if the matrices are equal within the bounds of `EPSILON`;
    /// otherwise, it returns `false`.
    ///
    /// # Examples
    ///
    /// ```
    /// let matrix1 = Matrix::new(2, 2);
    /// let matrix2 = Matrix::new(2, 2);
    /// assert!(matrix1.equals(&matrix2));
    /// ```
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

    /// Multiplies the current matrix with another matrix.
    ///
    /// This method performs matrix multiplication, which is a fundamental operation in linear algebra.
    /// The multiplication is done by taking the dot product of rows from the current matrix with columns from the other matrix.
    /// The result is a new matrix where each element at position (i, j) is the sum of the products of the corresponding elements
    /// from the ith row of the current matrix and the jth column of the other matrix.
    ///
    /// # Arguments
    ///
    /// * `other` - A reference to the matrix to multiply with.
    ///
    /// # Returns
    ///
    /// A new `Matrix` instance representing the result of the multiplication.
    ///
    /// # Examples
    ///
    /// ```
    /// let a = Matrix::new(2, 3);
    /// let b = Matrix::new(3, 2);
    /// let c = a.multiply(&b);
    /// ```
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

    /// Multiplies this matrix by a tuple, effectively transforming the tuple.
    ///
    /// This method applies a matrix transformation to a tuple, which can represent a point or a vector in space.
    /// The transformation is performed by multiplying the matrix by the tuple, using matrix multiplication rules.
    /// Each component of the resulting tuple is calculated as a weighted sum of the tuple's components, with weights
    /// given by the corresponding row of the matrix. This operation is fundamental in computer graphics for transforming
    /// points and vectors in 3D space.
    ///
    /// # Arguments
    ///
    /// * `other` - The `Tuple` to be transformed by this matrix.
    ///
    /// # Returns
    ///
    /// A new `Tuple` that is the result of the transformation.
    pub fn multiply_tuple(&self, other: &Tuple) -> Tuple {
        let x = self.get(0, 0) * other.x + self.get(0, 1) * other.y + self.get(0, 2) * other.z + self.get(0, 3) * other.w;
        let y = self.get(1, 0) * other.x + self.get(1, 1) * other.y + self.get(1, 2) * other.z + self.get(1, 3) * other.w;
        let z = self.get(2, 0) * other.x + self.get(2, 1) * other.y + self.get(2, 2) * other.z + self.get(2, 3) * other.w;
        let w = self.get(3, 0) * other.x + self.get(3, 1) * other.y + self.get(3, 2) * other.z + self.get(3, 3) * other.w;
        Tuple::new(x, y, z, w)
    }

    /// Creates an identity matrix of a given size.
    ///
    /// An identity matrix is a square matrix with ones on the main diagonal and zeros elsewhere.
    /// This function generates an identity matrix of the specified size, which can be used for various
    /// linear algebra operations, such as matrix multiplication, without altering the other matrix.
    ///
    /// # Arguments
    ///
    /// * `size` - The size of the rows and columns of the identity matrix.
    ///
    /// # Returns
    ///
    /// Returns a new `Matrix` instance representing the identity matrix of the given size.
    pub fn identity(size: usize) -> Matrix {
        let mut m = Matrix::new(size, size);
        for i in 0..size {
            m.set(i, i, 1.0);
        }
        m
    }

    /// Transposes the matrix.
    ///
    /// Transposition of a matrix is an operation that flips a matrix over its diagonal,
    /// switching the row and column indices of the matrix. This method creates a new matrix
    /// where each element at position (i, j) in the original matrix is moved to position (j, i)
    /// in the new matrix.
    ///
    /// # Returns
    ///
    /// Returns a new `Matrix` instance that is the transpose of the original matrix.
    pub fn transpose(&self) -> Matrix {
        let mut result = Matrix::new(self.cols, self.rows);
        for i in 0..self.rows {
            for j in 0..self.cols {
                result.set(j, i, self.get(i, j));
            }
        }
        result
    }

    /// Calculates the determinant of the matrix.
    ///
    /// The determinant is a scalar value that can be computed from the elements of a square matrix and
    /// encapsulates certain properties of the matrix. For a 2x2 matrix, the determinant is calculated
    /// directly. For matrices larger than 2x2, the determinant is calculated recursively using cofactors.
    ///
    /// # Returns
    ///
    /// The determinant of the matrix as a `f64`.
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

    /// Generates a submatrix by removing the specified row and column.
    ///
    /// A submatrix is formed by deleting one row and one column from a matrix. This operation is
    /// often used in the calculation of a determinant or a cofactor of a matrix.
    ///
    /// # Arguments
    ///
    /// * `row` - The zero-based index of the row to remove.
    /// * `col` - The zero-based index of the column to remove.
    ///
    /// # Returns
    ///
    /// A new `Matrix` instance representing the submatrix.
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

    /// Calculates the minor of the matrix at a given row and column.
    ///
    /// The minor of an element in a matrix is the determinant of the submatrix formed by removing
    /// the element's row and column. This function is used in the calculation of cofactors and the
    /// determinant of larger matrices.
    ///
    /// # Arguments
    ///
    /// * `row` - The zero-based index of the row of the element.
    /// * `col` - The zero-based index of the column of the element.
    ///
    /// # Returns
    ///
    /// The minor of the element at the specified row and column as a `f64`.
    pub fn minor(&self, row: usize, col: usize) -> f64 {
        self.submatrix(row, col).determinant()
    }

    /// Calculates the cofactor of the matrix at a given row and column.
    ///
    /// The cofactor is calculated by taking the minor of the element at the specified row and column
    /// and multiplying it by -1 if the sum of the row and column indices is odd. Cofactors are used
    /// in the calculation of the determinant and the inverse of a matrix.
    ///
    /// # Arguments
    ///
    /// * `row` - The zero-based index of the row of the element.
    /// * `col` - The zero-based index of the column of the element.
    ///
    /// # Returns
    ///
    /// The cofactor of the element at the specified row and column as a `f64`.
    pub fn cofactor(&self, row: usize, col: usize) -> f64 {
        let minor = self.minor(row, col);
        if (row + col) % 2 == 0 {
            minor
        } else {
            -minor
        }
    }

    /// Calculates the inverse of the matrix.
    ///
    /// This method first attempts to retrieve a cached inverse from `inverse_cache`. If a cached inverse is not available,
    /// it proceeds to calculate the inverse. The calculation involves determining the cofactor for each element of the matrix,
    /// transposing the matrix of cofactors, and then dividing each element by the determinant of the original matrix.
    /// The result is then cached for future use. This method assumes the matrix is invertible (i.e., has a non-zero determinant).
    ///
    /// # Returns
    ///
    /// A new `Matrix` instance representing the inverse of the original matrix. If the matrix is not invertible (determinant is zero),
    /// the behavior is undefined as the method does not currently handle this case explicitly.
    pub fn inverse(&self) -> Matrix {
        let mut inverse_cache = self.inverse_cache.lock().unwrap();

        // If the inverse is already cached, return it
        if let Some(inverse) = &*inverse_cache {
            return (**inverse).clone();
        }

        // Otherwise, calculate the inverse
        let det = self.determinant();
        let mut result = Matrix::new(self.rows, self.cols);
        for i in 0..self.rows {
            for j in 0..self.cols {
                let c = self.cofactor(i, j);
                result.set(j, i, c / det);
            }
        }
        let inverse = result;

        // Cache the inverse
        *inverse_cache = Some(Box::new(inverse.clone()));

        inverse
    }

    /// Translates a matrix by the given x, y, and z distances.
    ///
    /// This function creates a translation matrix by applying the given x, y, and z translations
    /// to the identity matrix. The translation values are set in the last column of the matrix.
    /// This type of transformation is commonly used in graphics programming to move objects
    /// in 3D space.
    ///
    /// # Arguments
    ///
    /// * `x` - The distance to translate along the x-axis.
    /// * `y` - The distance to translate along the y-axis.
    /// * `z` - The distance to translate along the z-axis.
    ///
    /// # Returns
    ///
    /// Returns a new `Matrix` instance representing the translation.
    pub fn translate(x: f64, y: f64, z: f64) -> Matrix {
        let mut m = Matrix::identity(4);
        m.set(0, 3, x);
        m.set(1, 3, y);
        m.set(2, 3, z);
        m
    }

    /// Scales a matrix by the given x, y, and z factors.
    ///
    /// This function creates a scaling matrix by applying the given x, y, and z scaling factors
    /// to the identity matrix. The scaling values are set on the main diagonal of the matrix.
    /// Scaling transformations are used in graphics programming to change the size of objects
    /// in 3D space.
    ///
    /// # Arguments
    ///
    /// * `x` - The factor to scale along the x-axis.
    /// * `y` - The factor to scale along the y-axis.
    /// * `z` - The factor to scale along the z-axis.
    ///
    /// # Returns
    ///
    /// Returns a new `Matrix` instance representing the scaling transformation.
    pub fn scale(x: f64, y: f64, z: f64) -> Matrix {
        let mut m = Matrix::identity(4);
        m.set(0, 0, x);
        m.set(1, 1, y);
        m.set(2, 2, z);
        m
    }

    /// Rotates a matrix around the x-axis by a given angle.
    ///
    /// This function creates a rotation matrix for rotation around the x-axis by the angle `r` (in radians).
    /// The rotation follows the right-hand rule, so a positive angle indicates a counter-clockwise rotation
    /// when looking from the positive end of the x-axis towards the origin.
    ///
    /// # Arguments
    ///
    /// * `r` - The angle of rotation in radians.
    ///
    /// # Returns
    ///
    /// Returns a new `Matrix` instance representing the rotation.
    pub fn rotate_x(r: f64) -> Matrix {
        let mut m = Matrix::identity(4);
        m.set(1, 1, r.cos());
        m.set(1, 2, -r.sin());
        m.set(2, 1, r.sin());
        m.set(2, 2, r.cos());
        m
    }

    /// Rotates a matrix around the y-axis by a given angle.
    ///
    /// This function creates a rotation matrix for rotation around the y-axis by the angle `r` (in radians).
    /// The rotation follows the right-hand rule, so a positive angle indicates a counter-clockwise rotation
    /// when looking from the positive end of the y-axis towards the origin.
    ///
    /// # Arguments
    ///
    /// * `r` - The angle of rotation in radians.
    ///
    /// # Returns
    ///
    /// Returns a new `Matrix` instance representing the rotation.
    pub fn rotate_y(r: f64) -> Matrix {
        let mut m = Matrix::identity(4);
        m.set(0, 0, r.cos());
        m.set(0, 2, r.sin());
        m.set(2, 0, -r.sin());
        m.set(2, 2, r.cos());
        m
    }

    /// Rotates a matrix around the z-axis by a given angle.
    ///
    /// This function creates a rotation matrix for rotation around the z-axis by the angle `r` (in radians).
    /// The rotation follows the right-hand rule, so a positive angle indicates a counter-clockwise rotation
    /// when looking from the positive end of the z-axis towards the origin.
    ///
    /// # Arguments
    ///
    /// * `r` - The angle of rotation in radians.
    ///
    /// # Returns
    ///
    /// Returns a new `Matrix` instance representing the rotation.
    pub fn rotate_z(r: f64) -> Matrix {
        let mut m = Matrix::identity(4);
        m.set(0, 0, r.cos());
        m.set(0, 1, -r.sin());
        m.set(1, 0, r.sin());
        m.set(1, 1, r.cos());
        m
    }

    /// Applies a shearing transformation to a matrix.
    ///
    /// Shearing (or skewing) is a transformation that displaces each point in a fixed direction,
    /// by an amount proportional to its signed distance from a line parallel to that direction.
    /// This function creates a shearing matrix that can be applied to points or vectors in 3D space.
    ///
    /// # Arguments
    ///
    /// * `xy` - The factor by which coordinates in the x direction are displaced in proportion to their y coordinate.
    /// * `xz` - The factor by which coordinates in the x direction are displaced in proportion to their z coordinate.
    /// * `yx` - The factor by which coordinates in the y direction are displaced in proportion to their x coordinate.
    /// * `yz` - The factor by which coordinates in the y direction are displaced in proportion to their z coordinate.
    /// * `zx` - The factor by which coordinates in the z direction are displaced in proportion to their x coordinate.
    /// * `zy` - The factor by which coordinates in the z direction are displaced in proportion to their y coordinate.
    ///
    /// # Returns
    ///
    /// Returns a new `Matrix` instance representing the shearing transformation.
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

    /// Constructs a view transformation matrix.
    ///
    /// This function creates a view transformation matrix used in rendering scenes from a specific viewpoint.
    /// It effectively transforms the world space into the camera (or view) space. The transformation is composed
    /// of two main steps: orientation and translation. The orientation aligns the world axes with the camera axes,
    /// while the translation moves the scene to align the camera position with the origin.
    ///
    /// # Arguments
    ///
    /// * `from` - A `Tuple` representing the camera's position in world space.
    /// * `to` - A `Tuple` representing the point in world space the camera is looking at.
    /// * `up` - A `Tuple` representing the up direction for the camera, typically (0, 1, 0) for an upright camera.
    ///
    /// # Returns
    ///
    /// Returns a new `Matrix` instance representing the view transformation.
    ///
    /// # Examples
    ///
    /// ```
    /// let from = Tuple::point(0.0, 0.0, 8.0);
    /// let to = Tuple::point(0.0, 0.0, 0.0);
    /// let up = Tuple::vector(0.0, 1.0, 0.0);
    /// let view_transform = Matrix::view_transform(from, to, up);
    /// ```
    pub fn view_transform(from: Tuple, to: Tuple, up: Tuple) -> Matrix {
        let forward = (to - from).normalize();
        let left = forward.cross(&up.normalize());
        let true_up = left.cross(&forward);
        let mut orientation = Matrix::new(4, 4);
        orientation.set(0, 0, left.x);
        orientation.set(0, 1, left.y);
        orientation.set(0, 2, left.z);
        orientation.set(1, 0, true_up.x);
        orientation.set(1, 1, true_up.y);
        orientation.set(1, 2, true_up.z);
        orientation.set(2, 0, -forward.x);
        orientation.set(2, 1, -forward.y);
        orientation.set(2, 2, -forward.z);
        orientation.set(3, 0, 0f64);
        orientation.set(3, 1, 0f64);
        orientation.set(3, 2, 0f64);
        orientation.set(3, 3, 1f64);

        let translation = Matrix::translate(-from.x, -from.y, -from.z);
        orientation.multiply(&translation)
    }
}

/// Implements the multiplication operator for `Matrix` structs.
///
/// This trait implementation allows two `Matrix` instances to be multiplied using the `*` operator,
/// facilitating a more intuitive syntax for matrix multiplication. The multiplication is delegated
/// to the `multiply` method of the `Matrix` struct, which performs the actual matrix multiplication operation.
///
/// # Examples
///
/// ```
/// let a = Matrix::new(2, 3);
/// let b = Matrix::new(3, 2);
/// let result = a * b; // Uses the `mul` implementation under the hood.
/// ```
impl Mul for Matrix {
    type Output = Matrix;

    fn mul(self, other: Matrix) -> Matrix {
        self.multiply(&other)
    }
}


#[cfg(test)]
mod tests {
    use super::Matrix;
    use crate::tuple::Tuple;
    use super::EPSILON;

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

    #[test]
    fn test_matrix_view_transform() {
        let from = Tuple::point(0.0, 0.0, 0.0);
        let to = Tuple::point(0.0, 0.0, -1.0);
        let up = Tuple::vector(0.0, 1.0, 0.0);
        let t = Matrix::view_transform(from, to, up);
        assert_eq!(t, Matrix::identity(4));

        let from = Tuple::point(0.0, 0.0, 0.0);
        let to = Tuple::point(0.0, 0.0, 1.0);
        let up = Tuple::vector(0.0, 1.0, 0.0);
        let t = Matrix::view_transform(from, to, up);
        assert_eq!(t, Matrix::scale(-1.0, 1.0, -1.0));

        let from = Tuple::point(0.0, 0.0, 8.0);
        let to = Tuple::point(0.0, 0.0, 0.0);
        let up = Tuple::vector(0.0, 1.0, 0.0);
        let t = Matrix::view_transform(from, to, up);
        assert_eq!(t, Matrix::translate(0.0, 0.0, -8.0));

        let from = Tuple::point(1.0, 3.0, 2.0);
        let to = Tuple::point(4.0, -2.0, 8.0);
        let up = Tuple::vector(1.0, 1.0, 0.0);
        let t = Matrix::view_transform(from, to, up);
        let mut expected = Matrix::new(4, 4);
        expected.set(0, 0, -0.50709);
        expected.set(0, 1, 0.50709);
        expected.set(0, 2, 0.67612);
        expected.set(0, 3, -2.36643);
        expected.set(1, 0, 0.76772);
        expected.set(1, 1, 0.60609);
        expected.set(1, 2, 0.12122);
        expected.set(1, 3, -2.82843);
        expected.set(2, 0, -0.35857);
        expected.set(2, 1, 0.59761);
        expected.set(2, 2, -0.71714);
        expected.set(2, 3, 0.00000);
        expected.set(3, 0, 0.00000);
        expected.set(3, 1, 0.00000);
        expected.set(3, 2, 0.00000);
        expected.set(3, 3, 1.00000);
        assert_eq!(t, expected);
    }
}