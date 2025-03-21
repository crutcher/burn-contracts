use crate::shapes::parser::parse_shape_pattern;
use std::collections::HashMap;
use std::fmt::Display;

#[derive(thiserror::Error, Debug, PartialEq, Eq, Hash)]
pub enum ShapeExError {
    #[error("Parse error:: \"{input}\"")]
    ParseError { input: String },

    #[error("Invalid pattern, {error}:: \"{input}\"")]
    InvalidPattern { input: String, error: String },
}

#[derive(thiserror::Error, Debug, PartialEq, Eq, Hash)]
pub enum ShapeMatchError {
    #[error("TODO")]
    Todo(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ShapeEx {
    components: Vec<DimPattern>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DimPattern {
    Dim(String),
    Ellipsis,
    Composite(Vec<String>),
}

#[derive(Debug, Clone)]
pub struct ShapeMatch {
    pub shape: Vec<usize>,
    pub bindings: HashMap<String, usize>,
    pub ellipsis_range: Option<std::ops::Range<usize>>,
}

impl Display for ShapeEx {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        for component in &self.components {
            write!(f, "{component}")?;
        }
        Ok(())
    }
}

impl Display for DimPattern {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        match self {
            DimPattern::Dim(id) => write!(f, "{id}"),
            DimPattern::Ellipsis => write!(f, "..."),
            DimPattern::Composite(ids) => {
                write!(f, "(")?;
                for id in ids {
                    write!(f, "{id} ")?;
                }
                write!(f, ")")
            }
        }
    }
}

impl ShapeEx {
    /// Create a new `ShapePattern` from a list of `DimPatterns`
    ///
    /// ## Parameters
    ///
    /// - `components`: A list of `DimPatterns`
    ///
    /// ## Errors
    ///
    /// Returns an error if there are too many ellipses
    pub fn new(components: Vec<DimPattern>) -> Result<Self, ShapeExError> {
        Self { components }.validate()
    }

    fn validate(self) -> Result<Self, ShapeExError> {
        let mut ellipsis_count = 0;
        for component in &self.components {
            if let DimPattern::Ellipsis = component {
                ellipsis_count += 1;
            }
            if ellipsis_count > 1 {
                return Err(ShapeExError::InvalidPattern {
                    input: self.to_string(),
                    error: "Only one ellipsis is allowed".to_string(),
                });
            }
        }
        Ok(self)
    }

    /// Parse a `ShapePattern` from a string
    ///
    /// ## Parameters
    ///
    /// - `input`: A string representation of the `ShapePattern`
    ///
    /// ## Errors
    ///
    /// Returns an error if the input string cannot be parsed;
    /// or the pattern is invalid.
    pub fn parse(input: &str) -> Result<Self, ShapeExError> {
        parse_shape_pattern(input)
    }

    /// Get the components of the `ShapePattern`.
    #[must_use]
    pub fn components(&self) -> &[DimPattern] {
        &self.components
    }

    /// Get the position of the ellipsis in the `ShapePattern`; if any.
    #[must_use]
    pub fn ellipsis_pos(&self) -> Option<usize> {
        self.components
            .iter()
            .position(|c| matches!(c, DimPattern::Ellipsis))
    }

    /// Check if the `ShapePattern` has an ellipsis.
    #[must_use]
    pub fn has_ellipsis(&self) -> bool {
        self.ellipsis_pos().is_some()
    }

    /// Assert that the `ShapeEx` matches a given shape.
    ///
    /// ## Parameters
    ///
    /// - `shape`: The shape to match against.
    /// - `bindings`: The bindings to use for matching.
    ///
    /// ## Errors
    ///
    /// Returns an error if the shape does not match the pattern.
    ///
    /// ## Returns
    ///
    /// Returns a `ShapeMatch` if the shape matches the pattern.
    #[allow(clippy::missing_panics_doc)]
    pub fn assert(
        &self,
        shape: &[usize],
        bindings: &HashMap<String, usize>,
    ) -> Result<ShapeMatch, ShapeMatchError> {
        let dims = shape.len();
        let ellipsis_pos = self.ellipsis_pos();
        let non_e_comps = match ellipsis_pos {
            Some(_) => self.components.len() - 1,
            None => self.components.len(),
        };
        if non_e_comps > dims {
            return Err(ShapeMatchError::Todo("Not Enough Dims".to_string()));
        }
        let ellipsis_range = ellipsis_pos.map(|pos| pos..pos + dims - non_e_comps);

        let mut bindings = bindings.clone();

        let mut i = 0;
        for component in &self.components {
            let dim_shape = shape[i];
            match component {
                DimPattern::Ellipsis => {
                    i = ellipsis_range.clone().unwrap().end;
                }
                DimPattern::Dim(id) => {
                    match bindings.get(id) {
                        Some(value) => {
                            if *value != dim_shape {
                                return Err(ShapeMatchError::Todo("Mismatch".to_string()));
                            }
                        }
                        None => {
                            bindings.insert(id.clone(), dim_shape);
                        }
                    }
                    i += 1;
                }
                DimPattern::Composite(ids) => {
                    let mut acc = 1;
                    let mut unbound: Option<String> = None;
                    for factor in ids {
                        if let Some(value) = bindings.get(factor) {
                            acc *= *value;
                        } else {
                            if unbound.is_some() {
                                return Err(ShapeMatchError::Todo("Multiple Unbound".to_string()));
                            }
                            unbound = Some(factor.clone());
                        }
                    }
                    if let Some(factor) = unbound {
                        if dim_shape % acc != 0 {
                            return Err(ShapeMatchError::Todo("Mismatch".to_string()));
                        }
                        bindings.insert(factor, dim_shape / acc);
                    }
                    i += 1;
                }
            }
        }

        Ok(ShapeMatch {
            shape: shape.to_vec(),
            bindings,
            ellipsis_range,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use once_cell::sync::Lazy;
    use std::error::Error;

    #[test]
    #[allow(clippy::many_single_char_names)]
    fn test_assert() -> Result<(), Box<dyn Error>> {
        // TODO: resolve how to handle init-once vrs LRU cache for pattern parsing?
        static PATTERN: Lazy<ShapeEx> = Lazy::new(|| match ShapeEx::parse("b ... (h p) (w p) c") {
            Ok(p) => p,
            Err(e) => panic!("{}", e),
        });

        let b = 2;
        let h = 3;
        let w = 4;
        let p = 2;
        let c = 3;

        let extra_key = 7;

        let shape = [b, 9, 9, h * p, w * p, c];

        let mut bindings = HashMap::new();
        bindings.insert("b".to_string(), b);
        bindings.insert("p".to_string(), p);
        bindings.insert("x".to_string(), extra_key);

        let m = PATTERN.assert(shape.as_ref(), &bindings)?;

        assert_eq!(m.shape, shape);
        assert_eq!(m.ellipsis_range, Some(1..3));
        assert_eq!(m.bindings["b"], b);
        assert_eq!(m.bindings["h"], h);
        assert_eq!(m.bindings["w"], w);
        assert_eq!(m.bindings["p"], p);
        assert_eq!(m.bindings["c"], c);
        assert_eq!(m.bindings["x"], extra_key);

        Ok(())
    }
}
