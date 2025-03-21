use crate::shapes::exp::{PatternComponent, ShapePattern, ShapePatternError};
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::{alpha1, alphanumeric1, multispace0, multispace1};
use nom::combinator::{map, recognize};
use nom::multi::{many0, many1, separated_list1};
use nom::sequence::{delimited, pair, preceded, terminated};
use nom::IResult;
use nom::Parser;

use once_cell::sync::Lazy;

use quick_cache::sync::Cache;

const PARSE_CACHE: Lazy<Cache<String, ShapePattern>> = Lazy::new(|| Cache::new(1000));

/// Parse a `ShapePattern` and cache the result.
///
/// This function is thread-safe; and uses a process-wide cache,
/// so expressions should be parsed-once and shared.
///
/// ## Parameters
///
/// - `input`: A string representation of the `ShapePattern`
///
/// ## Errors
///
/// Returns an error if the input string cannot be parsed;
/// or the pattern is invalid.
pub fn cached_parse_shape_pattern(input: &str) -> Result<ShapePattern, ShapePatternError> {
    PARSE_CACHE.get_or_insert_with(input, || parse_shape_pattern(input))
}

/// Parse a `ShapePattern`.
///
/// ## Parameters
///
/// - `input`: A string representation of the `ShapePattern`
///
/// ## Errors
///
/// Returns an error if the input string cannot be parsed;
/// or the pattern is invalid.
pub fn parse_shape_pattern(input: &str) -> Result<ShapePattern, ShapePatternError> {
    match components_parser(input.trim()) {
        Ok((remaining, components)) => {
            if remaining.is_empty() {
                Ok(ShapePattern::new(components)?)
            } else {
                Err(ShapePatternError::ParseError {
                    input: input.to_string(),
                })
            }
        }
        Err(_) => Err(ShapePatternError::ParseError {
            input: input.to_string(),
        }),
    }
}

/// Parse an identifier r"[_a-zA-Z][_a-zA-Z0-9]*" -> String
fn ident_parser(input: &str) -> IResult<&str, String> {
    map(
        recognize(pair(
            alt((alpha1, tag("_"))),
            many0(alt((alphanumeric1, tag("_")))),
        )),
        String::from,
    )
    .parse(input)
}

/// Parse an ellipsis: r"..." -> Ellipsis
fn ellipsis_parser(input: &str) -> IResult<&str, PatternComponent> {
    map(tag("..."), |_| PatternComponent::Ellipsis).parse(input)
}

/// Parse a dimension: identifier -> Dim
fn dim_parser(input: &str) -> IResult<&str, PatternComponent> {
    map(ident_parser, PatternComponent::Dim).parse(input)
}

/// Parse a composite dimension: (id1 id2 ...) -> Composite
fn composite_parser(input: &str) -> IResult<&str, PatternComponent> {
    map(
        delimited(
            terminated(tag("("), multispace0),
            separated_list1(multispace1, ident_parser),
            preceded(multispace0, tag(")")),
        ),
        PatternComponent::Composite,
    )
    .parse(input)
}

/// Parse a list of components separated by whitespace
fn components_parser(input: &str) -> IResult<&str, Vec<PatternComponent>> {
    many1(terminated(
        alt((ellipsis_parser, dim_parser, composite_parser)),
        multispace0,
    ))
    .parse(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ellipsis() {
        assert_eq!(ellipsis_parser("..."), Ok(("", PatternComponent::Ellipsis)));
        assert_eq!(
            ellipsis_parser("... "),
            Ok((" ", PatternComponent::Ellipsis))
        );
        assert_eq!(
            ellipsis_parser("...x"),
            Ok(("x", PatternComponent::Ellipsis))
        );
    }

    #[test]
    fn test_identifier() {
        for prefix in &["_", "a", "A"] {
            for suffix in &["", "_", "a", "A", "1", "_"] {
                let id = format!("{prefix}{suffix}");
                let input = format!("{id} z");
                assert_eq!(ident_parser(&input), Ok((" z", id)));
            }
        }

        // TODO: bad inputs
    }

    #[test]
    fn test_dimension() {
        for id in &["x", "X", "_", "x1", "X1", "_1"] {
            let input = format!("{id} z");
            assert_eq!(
                dim_parser(&input),
                Ok((" z", PatternComponent::Dim((*id).to_string())))
            );
        }
    }

    #[test]
    fn test_composite() {
        assert_eq!(
            composite_parser("(x)"),
            Ok(("", PatternComponent::Composite(vec!["x".to_string()])))
        );
        assert_eq!(
            composite_parser("(x y)"),
            Ok((
                "",
                PatternComponent::Composite(vec!["x".to_string(), "y".to_string()])
            ))
        );
    }

    #[test]
    fn test_parse_shape_pattern() {
        assert_eq!(
            parse_shape_pattern("..."),
            ShapePattern::new(vec![PatternComponent::Ellipsis])
        );
        assert_eq!(
            parse_shape_pattern("x"),
            ShapePattern::new(vec![PatternComponent::Dim("x".to_string())])
        );
        assert_eq!(
            parse_shape_pattern("b ...( x  y ) c"),
            ShapePattern::new(vec![
                PatternComponent::Dim("b".to_string()),
                PatternComponent::Ellipsis,
                PatternComponent::Composite(vec!["x".to_string(), "y".to_string()]),
                PatternComponent::Dim("c".to_string())
            ])
        );
    }
}
