use std::fmt;

pub mod multiply;

#[derive(Debug, PartialEq)]
pub struct Matrix {
    rows: usize,
    cols: usize,
    data: Vec<f64>,
}

impl Matrix {
    pub fn new(rows: usize, cols: usize, data: Vec<f64>) -> Self {
        Self { rows, cols, data }
    }

    /// Load a matrix from a text file.
    ///
    /// The file should contain width and height as integers on the first line, followed by the matrix data as floting point numbers.
    pub fn from_file(path: &str) -> Result<Self, std::io::Error> {
        let data = std::fs::read_to_string(path)?;
        Ok(data.as_str().into())
    }

    pub fn cols(&self) -> usize {
        self.cols
    }

    pub fn rows(&self) -> usize {
        self.rows
    }

    pub fn data(&self) -> &Vec<f64> {
        &self.data
    }

    /// Swap the rows and columns of the matrix.
    pub fn invert(&self) -> Matrix {
        let rows = self.cols;
        let cols = self.rows;

        let data = self
            .data
            .iter()
            .enumerate()
            .map(|(i, _)| {
                let new_index = (i % cols) * rows + (i / cols);
                self.data[new_index]
            })
            .collect();

        Matrix { rows, cols, data }
    }

    /// Get the sum of all values in the matrix.
    ///
    /// Useful for validating the results of a matrix multiplication.
    pub fn sum(&self) -> f64 {
        self.data.iter().sum()
    }
}

impl Into<Matrix> for &str {
    fn into(self) -> Matrix {
        list_parser::matrix(self).unwrap()
    }
}

impl fmt::Display for Matrix {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "{} {}", self.cols, self.rows)?;
        for row in self.data.chunks(self.cols) {
            for value in row {
                write!(f, "{:.32} ", value)?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

peg::parser! {
  grammar list_parser() for str {
    rule whitespace() = quiet!{[' ' | '\n' | '\t']+}

    pub rule dimension() -> usize
      = n:$(['0'..='9']+) {? n.parse().or(Err("usize")) }

    pub rule value() -> f64
      = n:$(['-']?['0'..='9']+(['.']['0'..='9']+)?(['e']['-' | '+']?['0'..='9']+)?) {? n.parse::<f64>().or(Err("f64")) }

    pub rule list() -> Vec<f64>
      = l:(value() ** whitespace()) { l }

    pub rule matrix() -> Matrix
      = whitespace()? rows:dimension() whitespace() cols:dimension() whitespace() data:list() whitespace()? {
        Matrix { rows, cols, data: data.into_iter().map(|x| x as f64).collect() }
      }
  }
}

#[test]
fn test_value() {
    assert_eq!(list_parser::value("1"), Ok(1.0));
}

#[test]
fn test_value_with_exponent() {
    assert_eq!(list_parser::value("-4.30266e-05"), Ok(-4.30266e-05));
}

#[test]
fn test_most_simple_matrix() {
    assert_eq!(
        list_parser::matrix("1 1 1"),
        Ok(Matrix {
            rows: 1,
            cols: 1,
            data: vec![1.0]
        })
    );
}

#[test]
fn works_with_leading_and_trailing_whitespace() {
    assert_eq!(
        list_parser::matrix("  1 1 1  "),
        Ok(Matrix {
            rows: 1,
            cols: 1,
            data: vec![1.0]
        })
    );
}

#[test]
fn test_two_by_two_matrix() {
    assert_eq!(
        list_parser::matrix(
            r#"2 2
        1 2
        3 4"#
        ),
        Ok(Matrix {
            rows: 2,
            cols: 2,
            data: vec![1.0, 2.0, 3.0, 4.0]
        })
    );
}

#[test]
fn test_matrix_with_weird_numbers() {
    assert_eq!(
        list_parser::matrix(
            r#"2 2
        1.3 -2.23
        22222222222223 0.0"#
        ),
        Ok(Matrix {
            rows: 2,
            cols: 2,
            data: vec![1.3, -2.23, 22222222222223.0, 0.0]
        })
    );
}

#[test]
fn invert_matrix_works() {
    let matrix = Matrix {
        rows: 2,
        cols: 2,
        data: vec![1.0, 2.0, 3.0, 4.0],
    };
    assert_eq!(
        matrix.invert(),
        Matrix {
            rows: 2,
            cols: 2,
            data: vec![1.0, 3.0, 2.0, 4.0]
        }
    );
}

#[test]
fn invert_matrix_works_b() {
    let matrix = Matrix {
        rows: 3,
        cols: 2,
        data: vec![7.0, 8.0, 9.0, 10.0, 11.0, 12.0],
    };
    assert_eq!(
        matrix.invert(),
        Matrix {
            rows: 2,
            cols: 3,
            data: vec![7.0, 9.0, 11.0, 8.0, 10.0, 12.0]
        }
    );
}
