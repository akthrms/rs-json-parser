use nom::{branch::alt, bytes::complete::tag, IResult};
use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub enum Json {
    String(String),
    Number(f64),
    Boolean(bool),
    Array(Vec<Json>),
    Object(HashMap<String, Json>),
    Null,
}

fn parse_boolean(input: &str) -> IResult<&str, Json> {
    fn parse_true(input: &str) -> IResult<&str, Json> {
        let (input, _) = tag("true")(input)?;
        Ok((input, Json::Boolean(true)))
    }

    fn parse_false(input: &str) -> IResult<&str, Json> {
        let (input, _) = tag("false")(input)?;
        Ok((input, Json::Boolean(false)))
    }

    alt((parse_true, parse_false))(input)
}

fn parse_null(input: &str) -> IResult<&str, Json> {
    let (input, _) = tag("null")(input)?;
    Ok((input, Json::Null))
}

#[cfg(test)]
mod tests {
    use crate::{parse_boolean, parse_null, Json};

    #[test]
    fn test_parse_boolean() -> Result<(), Box<dyn std::error::Error>> {
        let (_, json) = parse_boolean("true")?;
        assert_eq!(json, Json::Boolean(true));

        let (_, json) = parse_boolean("false")?;
        assert_eq!(json, Json::Boolean(false));

        Ok(())
    }

    #[test]
    fn test_parse_null() -> Result<(), Box<dyn std::error::Error>> {
        let (_, json) = parse_null("null")?;
        assert_eq!(json, Json::Null);
        Ok(())
    }
}
