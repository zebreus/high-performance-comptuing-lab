use super::Matrix;
use rayon::prelude::*;

impl Matrix {
    /// Inverts the matrix by swapping the rows and columns.
    pub fn multiply(&self, other: &Matrix) -> Result<Matrix, String> {
        if self.cols != other.rows {
            return Err(format!(
                "Matrix A has {} columns, but matrix B has {} rows.",
                self.cols, other.rows
            ));
        }

        // TODO: Figure out, if it is worth it to invert the other matrix before
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

        return Ok(Matrix {
            rows: self.rows,
            cols: other.cols,
            data,
        });
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
        let c = a.multiply(&b).unwrap();
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
        let c = a.multiply(&b).unwrap();
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
        let c = a.multiply(&b).unwrap();
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
        let c = a.multiply(&b).unwrap();
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
