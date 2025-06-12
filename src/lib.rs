use std::fmt::{self, Display};

/// Binary S-Expression
/// - Atoms are in bytes
/// - Lists are represented as vector
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BSExp {
    Atom(Vec<u8>),
    List(Vec<BSExp>),
}

impl BSExp {
    /// Create a new BSExp from an atom
    pub fn atom<T: Into<BSExp>>(value: T) -> Self {
        value.into()
    }

    /// Create a new BSExp from a list of BSExp
    pub fn list<T: Into<Vec<BSExp>>>(value: T) -> Self {
        BSExp::List(value.into())
    }
}

impl From<&str> for BSExp {
    fn from(s: &str) -> Self {
        BSExp::Atom(s.as_bytes().to_vec())
    }
}

impl From<String> for BSExp {
    fn from(s: String) -> Self {
        BSExp::Atom(s.into_bytes())
    }
}

#[macro_export]
macro_rules! bsexp {
    // Match a list expression: [ ... ]
    ( [ $( $elem:tt ),* ] ) => {
        $crate::BSExp::list(vec![
            $( bsexp!($elem) ),*
        ])
    };

    // Match a single atom expression
    ( $atom:expr ) => {
        $crate::BSExp::atom($atom)
    };
}

impl Display for BSExp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            self.fmt_pretty(f, 0)
        } else {
            match self {
                BSExp::Atom(item) => match String::from_utf8(item.clone()) {
                    Ok(s) => f.write_str(s.as_str()),
                    Err(_) => write!(
                        f,
                        "{}",
                        item.iter()
                            .map(|b| b.to_string())
                            .collect::<Vec<_>>()
                            .join(" ")
                    ),
                },
                BSExp::List(items) => write!(
                    f,
                    "({})",
                    items
                        .iter()
                        .map(|x| x.to_string())
                        .collect::<Vec<_>>()
                        .join(" ")
                ),
            }
        }
    }
}
impl BSExp {
    fn indention(f: &mut fmt::Formatter<'_>, indent: usize) -> Result<(), std::fmt::Error> {
        (0..indent).map(|_| f.write_str(" ")).collect()
    }

    fn fmt_pretty(&self, f: &mut fmt::Formatter<'_>, indent: usize) -> fmt::Result {
        match self {
            BSExp::Atom(_) => {
                BSExp::indention(f, indent)?;
                write!(f, "{self}")
            }

            BSExp::List(v) => {
                let mut buf = String::new();
                use std::fmt::Write as _;
                write!(&mut buf, "{self}")?;

                if buf.len() < f.width().unwrap_or(60) {
                    BSExp::indention(f, indent)?;
                    f.write_str(&buf)
                } else {
                    BSExp::indention(f, indent)?;
                    f.write_str("(")?;

                    let mut it = v.iter();
                    if let Some(x) = it.next() {
                        match x {
                            BSExp::Atom(_) => write!(f, "{x}")?,
                            _ => {
                                f.write_str("\n")?;
                                x.fmt_pretty(f, indent + 1)?
                            }
                        }
                    }

                    for x in it {
                        f.write_str("\n")?;
                        x.fmt_pretty(f, indent + 1)?;
                    }

                    f.write_str(")")
                }
            }
        }
    }
}

#[test]
fn test_bsexp_format() {
    let fib = bsexp!([
        "define",
        ["fibonacci", "n"],
        [
            "define",
            ["fib-iter", "a", "b", "count"],
            [
                "if",
                ["=", "count", "0"],
                "a",
                ["fib-iter", "b", ["+", "a", "b"], ["-", "count", "1"]]
            ]
        ],
        ["fib-iter", "0", "1", "n"]
    ]);
    assert_eq!(
        format!("{fib:#}"),
        "(define\n (fibonacci n)\n (define\n  (fib-iter a b count)\n  (if (= count 0) a (fib-iter b (+ a b) (- count 1))))\n (fib-iter 0 1 n))"
    );
}

pub mod vli;
