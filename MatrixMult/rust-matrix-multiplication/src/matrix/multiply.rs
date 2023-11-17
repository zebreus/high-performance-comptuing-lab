use std::sync::Mutex;

use super::Matrix;
use rayon::prelude::*;

impl Matrix {
    /// This multiply implementation inverts the second matrix before multiplying, so we can process the data of both matrices in continuous chunks.
    ///
    /// I would think that this one is quite performant, as we never access memory by an index
    pub fn multiply(&self, other: &Matrix) -> Matrix {
        let inverted_other = other.invert();

        let dot_product_length = self.cols;

        let data = self
            .data
            .par_chunks_exact(dot_product_length)
            .map(|row| {
                inverted_other
                    .data
                    .par_chunks_exact(dot_product_length)
                    .map(|column| row.iter().zip(column.iter()).map(|(a, b)| a * b).sum())
            })
            .flatten()
            .collect::<Vec<_>>();

        return Matrix {
            rows: self.rows,
            cols: other.cols,
            data,
        };
    }

    /// This multiply implementation is mostly faithful to the original, except that it does not create the result matrix beforehand
    ///
    /// If we would create the matrix beforehand, we would either
    /// 1. Have a mutex around the whole matrix
    /// 2. Pair up every coordinate in the result with a mutable reference to the corresponding entry in the result matrix
    /// 3. Use unsafe code to share the result matrix between threads mutably
    ///
    /// 1 Is probably quite slow, 2 results in weird code and 3 is unsafe
    pub fn multiply_faithful_iterators(&self, other: &Matrix) -> Matrix {
        let data = (0..self.rows)
            .into_par_iter()
            .flat_map(|row| {
                return (0..other.cols).into_par_iter().map(move |col| (row, col));
            })
            .map(|(row, col)| {
                let mut sum = 0.0;
                for i in 0..other.rows {
                    sum += self.read_at(&row, &i) * other.read_at(&i, &col);
                }
                sum
            })
            .collect::<Vec<_>>();

        return Matrix {
            rows: self.rows,
            cols: other.cols,
            data,
        };
    }

    /// Pair up every coordinate in the result with a mutable reference to the corresponding entry in the result matrix
    pub fn multiply_faithful_pairs(&self, other: &Matrix) -> Matrix {
        let mut result = vec![0.0f64; self.rows * other.cols];

        let parralel_iterator_with_coordinates =
            result.par_iter_mut().enumerate().map(|(i, slice)| {
                let row = i / other.cols;
                let col = i % other.cols;
                (row, col, slice)
            });

        parralel_iterator_with_coordinates.for_each(|(row, col, resulting_value)| {
            let mut sum = 0.0;
            for i in 0..other.rows {
                sum += self.read_at(&row, &i) * other.read_at(&i, &col);
            }
            *resulting_value = sum;
        });

        return Matrix {
            rows: self.rows,
            cols: other.cols,
            data: result,
        };
    }

    /// Use unsafe code to share the result matrix between threads mutably
    pub fn multiply_faithful_unsafe(&self, other: &Matrix) -> Matrix {
        unsafe {
            let mut result = vec![0.0f64; self.rows * other.cols];
            let result_matrix_pointer = result.as_mut_ptr() as usize;

            (0..self.rows)
                .into_par_iter()
                .flat_map(|row| {
                    return (0..other.cols).into_par_iter().map(move |col| (row, col));
                })
                .for_each(|(row, col)| {
                    let mut sum = 0.0;
                    for i in 0..other.rows {
                        sum += self.read_at(&row, &i) * other.read_at(&i, &col);
                    }
                    let result_pointer =
                        (result_matrix_pointer as *mut f64).add(row * other.cols + col);
                    *result_pointer = sum;
                });

            return Matrix {
                rows: self.rows,
                cols: other.cols,
                data: result,
            };
        }
    }

    /// Use unsafe code to share the result matrix between threads mutably
    pub fn multiply_faithful_mutex(&self, other: &Matrix) -> Matrix {
        let result_mutex: Mutex<Vec<f64>> = Mutex::new(vec![0.0f64; self.rows * other.cols]);

        (0..self.rows)
            .into_par_iter()
            .flat_map(|row| {
                return (0..other.cols).into_par_iter().map(move |col| (row, col));
            })
            .for_each(|(row, col)| {
                let mut sum = 0.0;
                for i in 0..other.rows {
                    sum += self.read_at(&row, &i) * other.read_at(&i, &col);
                }
                let mut result = result_mutex.lock().unwrap();
                result[row * other.cols + col] = sum;
            });

        return Matrix {
            rows: self.rows,
            cols: other.cols,
            data: result_mutex.into_inner().unwrap(),
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_multiplication_test() {
        let a = Matrix {
            rows: 1,
            cols: 1,
            data: vec![4.0],
        };
        let b = Matrix {
            rows: 1,
            cols: 1,
            data: vec![3.0],
        };
        let c = a.multiply_faithful_iterators(&b);
        assert_eq!(
            c,
            Matrix {
                rows: 1,
                cols: 1,
                data: vec![12.0]
            },
        );
    }

    #[test]
    fn asymetric_matrix_works() {
        let a = Matrix {
            rows: 2,
            cols: 1,
            data: vec![1.0, 2.0],
        };
        let b = Matrix {
            rows: 1,
            cols: 2,
            data: vec![3.0, 4.0],
        };
        let c = a.multiply_faithful_iterators(&b);
        assert_eq!(
            c,
            Matrix {
                rows: 2,
                cols: 2,
                data: vec![3.0, 4.0, 6.0, 8.0]
            },
        );
    }

    #[test]
    fn some_other_asymetric_matrix_works() {
        let a = Matrix {
            rows: 1,
            cols: 2,
            data: vec![1.0, 2.0],
        };
        let b = Matrix {
            rows: 2,
            cols: 1,
            data: vec![3.0, 4.0],
        };
        let c = a.multiply_faithful_iterators(&b);
        assert_eq!(
            c,
            Matrix {
                rows: 1,
                cols: 1,
                data: vec![11.0]
            },
        );
    }

    #[test]
    fn test_multiply() {
        let a = Matrix {
            rows: 2,
            cols: 3,
            data: vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0],
        };
        let b = Matrix {
            rows: 3,
            cols: 2,
            data: vec![7.0, 8.0, 9.0, 10.0, 11.0, 12.0],
        };
        let c = a.multiply_faithful_iterators(&b);
        assert_eq!(
            c,
            Matrix {
                rows: 2,
                cols: 2,
                data: vec![58.0, 64.0, 139.0, 154.0]
            },
            "Matrix multiplication returned incorrect result."
        );
    }
}
