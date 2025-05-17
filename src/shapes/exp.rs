use crate::shapes::bindings::{ShapeBindingSource, collect_binding_map, lookup_binding};
use crate::shapes::parser::{cached_parse_shape_pattern, parse_shape_pattern};
use std::collections::HashMap;
use std::fmt::Display;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ShapePattern {
    ellipsis_pos: Option<usize>,
    components: Vec<PatternComponent>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PatternComponent {
    Dim(String),
    Ellipsis,
    Composite(Vec<String>),
}

#[derive(thiserror::Error, Debug, PartialEq, Eq, Hash)]
pub enum ShapePatternError {
    #[error("Parse error for \"{pattern}\"")]
    ParseError { pattern: String },

    #[error("Invalid pattern \"{pattern}\": {message}")]
    InvalidPattern { pattern: String, message: String },

    #[error("Shape \"{shape:?}\" !~= \"{pattern}\" with {bindings:?}: {message}")]
    MatchError {
        shape: Vec<usize>,
        pattern: String,
        bindings: Vec<(String, usize)>,
        message: String,
    },
}

#[derive(Debug, Clone)]
pub struct ShapeMatch {
    pub shape: Vec<usize>,
    pub bindings: HashMap<String, usize>,
    pub ellipsis_range: Option<std::ops::Range<usize>>,
}

impl ShapeMatch {
    /// Select a subset of the bindings.
    ///
    /// ## Parameters
    ///
    /// - `keys`: The keys to select.
    ///
    /// ## Returns
    ///
    /// Returns the selected bindings.
    ///
    /// ## Panics
    ///
    /// Panics if a key is not found in the bindings.
    #[must_use]
    pub fn select<const D: usize>(
        &self,
        keys: [&str; D],
    ) -> [usize; D] {
        let mut result = [0; D];
        for (i, key) in keys.iter().enumerate() {
            result[i] = lookup_binding(&self.bindings, key).unwrap();
        }
        result
    }
}

impl Display for ShapePattern {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        for (idx, comp) in self.components.iter().enumerate() {
            if idx > 0 {
                write!(f, " ")?;
            }
            write!(f, "{comp}")?;
        }
        Ok(())
    }
}

impl Display for PatternComponent {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        match self {
            PatternComponent::Dim(id) => write!(f, "{id}"),
            PatternComponent::Ellipsis => write!(f, "..."),
            PatternComponent::Composite(ids) => {
                write!(f, "(")?;
                for (idx, id) in ids.iter().enumerate() {
                    if idx > 0 {
                        write!(f, " ")?;
                    }
                    write!(f, "{id}")?;
                }
                write!(f, ")")
            }
        }
    }
}

fn check_ellipsis_pos(components: &[PatternComponent]) -> Result<Option<usize>, ShapePatternError> {
    let mut ellipsis_pos = None;
    for (i, component) in components.iter().enumerate() {
        if let PatternComponent::Ellipsis = component {
            if ellipsis_pos.is_some() {
                return Err(ShapePatternError::InvalidPattern {
                    pattern: components
                        .iter()
                        .map(std::string::ToString::to_string)
                        .collect(),
                    message: "Only one ellipsis is allowed".to_string(),
                });
            }
            ellipsis_pos = Some(i);
        }
    }
    Ok(ellipsis_pos)
}

impl ShapePattern {
    /// Create a new `ShapePattern` from a list of `DimPatterns`
    ///
    /// ## Parameters
    ///
    /// - `components`: A list of `DimPatterns`
    ///
    /// ## Errors
    ///
    /// Returns an error if there are too many ellipses
    pub fn new(components: Vec<PatternComponent>) -> Result<Self, ShapePatternError> {
        Ok(Self {
            ellipsis_pos: check_ellipsis_pos(components.as_slice())?,
            components,
        })
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
    pub fn parse(input: &str) -> Result<Self, ShapePatternError> {
        parse_shape_pattern(input)
    }

    /// Parse a `ShapePattern` from a string, using a cache
    ///
    /// ## Parameters
    ///
    /// - `input`: A string representation of the `ShapePattern`
    ///
    /// ## Errors
    ///
    /// Returns an error if the input string cannot be parsed;
    /// or the pattern is invalid.
    pub fn cached_parse(input: &str) -> Result<Self, ShapePatternError> {
        cached_parse_shape_pattern(input)
    }

    /// Get the components of the `ShapePattern`.
    #[must_use]
    pub fn components(&self) -> &[PatternComponent] {
        &self.components
    }

    /// Get the position of the ellipsis in the `ShapePattern`; if any.
    #[must_use]
    pub fn ellipsis_pos(&self) -> Option<usize> {
        self.components
            .iter()
            .position(|c| matches!(c, PatternComponent::Ellipsis))
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
    pub fn match_bindings<B: ShapeBindingSource>(
        &self,
        shape: &[usize],
        bindings: B,
    ) -> Result<ShapeMatch, ShapePatternError> {
        // FIXME: Reconsider result contents.
        // - We can skip returning the source shape.
        // - returned bindings should be an assoc vec OR fixed array?
        //   - alloc size vs speed considerations
        // - return ellipsis dims, locations; both?
        // - multi-pass to resolve composite bindings?

        let bindings: HashMap<String, usize> = collect_binding_map(bindings);

        let dims = shape.len();
        let ellipsis_pos = self.ellipsis_pos();
        let non_e_comps = match ellipsis_pos {
            Some(_) => self.components.len() - 1,
            None => self.components.len(),
        };
        if non_e_comps > dims {
            return Err(ShapePatternError::MatchError {
                shape: shape.to_vec(),
                pattern: self.to_string(),
                bindings: bindings.iter().map(|(k, v)| (k.clone(), *v)).collect(),
                message: "Too few dimensions".to_string(),
            });
        }
        let ellipsis_range = ellipsis_pos.map(|pos| pos..pos + dims - non_e_comps);

        let mut export = HashMap::new();

        fn readthrough_lookup(
            bindings: &HashMap<String, usize>,
            target: &mut HashMap<String, usize>,
            id: &str,
        ) -> Option<usize> {
            match target.get(id) {
                Some(value) => Some(*value),
                None => match bindings.get(id) {
                    Some(value) => {
                        target.insert(id.to_string(), *value);
                        Some(*value)
                    }
                    None => None,
                },
            }
        }

        let mut i = 0;
        for component in &self.components {
            let dim_shape = shape[i];
            match component {
                PatternComponent::Ellipsis => {
                    i = ellipsis_range.clone().unwrap().end;
                }
                PatternComponent::Dim(id) => {
                    match readthrough_lookup(&bindings, &mut export, id) {
                        Some(bound_value) => {
                            if bound_value != dim_shape {
                                let message = format!(
                                    "Constraint Mismatch @{id}: {bound_value} != {dim_shape}"
                                );

                                return Err(ShapePatternError::MatchError {
                                    shape: shape.to_vec(),
                                    pattern: self.to_string(),
                                    bindings: bindings
                                        .iter()
                                        .map(|(k, v)| (k.clone(), *v))
                                        .collect(),
                                    message,
                                });
                            }
                        }
                        None => {
                            export.insert(id.clone(), dim_shape);
                        }
                    }
                    i += 1;
                }
                PatternComponent::Composite(ids) => {
                    let mut acc = 1;
                    let mut unbound: Option<String> = None;
                    for factor in ids {
                        if let Some(value) = readthrough_lookup(&bindings, &mut export, factor) {
                            acc *= value;
                        } else {
                            if unbound.is_some() {
                                return Err(ShapePatternError::MatchError {
                                    shape: shape.to_vec(),
                                    pattern: self.to_string(),
                                    bindings: bindings
                                        .iter()
                                        .map(|(k, v)| (k.clone(), *v))
                                        .collect(),
                                    message: "Multiple unbound factors in composite".to_string(),
                                });
                            }
                            unbound = Some(factor.clone());
                        }
                    }
                    if let Some(factor) = unbound {
                        if dim_shape % acc != 0 {
                            return Err(ShapePatternError::MatchError {
                                shape: shape.to_vec(),
                                pattern: self.to_string(),
                                bindings: bindings.iter().map(|(k, v)| (k.clone(), *v)).collect(),
                                message: format!(
                                    "Composite factor \"{factor}\" * {acc} != shape {dim_shape}",
                                ),
                            });
                        }
                        export.insert(factor, dim_shape / acc);
                    }
                    i += 1;
                }
            }
        }

        Ok(ShapeMatch {
            shape: shape.to_vec(),
            bindings: export,
            ellipsis_range,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::error::Error;

    #[test]
    fn test_display_pattern() {
        let pattern = ShapePattern::new(vec![
            PatternComponent::Dim("b".to_string()),
            PatternComponent::Ellipsis,
            PatternComponent::Composite(vec!["h".to_string(), "w".to_string()]),
            PatternComponent::Dim("c".to_string()),
        ])
        .unwrap();

        assert_eq!(pattern.to_string(), "b ... (h w) c");
    }

    #[test]
    #[allow(clippy::many_single_char_names)]
    fn test_parser_example() -> Result<(), Box<dyn Error>> {
        let shape = [2, 9, 9, 20 * 4, 10 * 4, 3];

        let [b, h, w, c] = ShapePattern::cached_parse("b ... (h p) (w p) c")?
            .match_bindings(&shape, &[("b", 2), ("p", 4)])?
            .select(["b", "h", "w", "c"]);

        assert_eq!(b, 2);
        assert_eq!(h, 20);
        assert_eq!(w, 10);
        assert_eq!(c, 3);

        Ok(())
    }

    #[test]
    #[allow(clippy::many_single_char_names)]
    fn test_assert() -> Result<(), Box<dyn Error>> {
        let b = 2;
        let h = 3;
        let w = 4;
        let p = 2;
        let c = 3;

        let extra = 7;

        let shape = [b, 9, 9, h * p, w * p, c];

        let mut bindings = HashMap::new();
        bindings.insert("b".to_string(), b);
        bindings.insert("p".to_string(), p);
        bindings.insert("extra".to_string(), extra);

        let m = ShapePattern::cached_parse("b ... (h p) (w p) c")?
            .match_bindings(shape.as_ref(), &bindings)?;

        assert_eq!(m.shape, shape);
        assert_eq!(m.ellipsis_range, Some(1..3));
        assert_eq!(m.bindings["b"], b);
        assert_eq!(m.bindings["h"], h);
        assert_eq!(m.bindings["w"], w);
        assert_eq!(m.bindings["p"], p);
        assert_eq!(m.bindings["c"], c);

        let [sel_b, sel_h, sel_w] = m.select(["b", "h", "w"]);
        assert_eq!(sel_b, b);
        assert_eq!(sel_h, h);
        assert_eq!(sel_w, w);

        Ok(())
    }
}
