#![allow(dead_code)]

use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{alphanumeric0, char},
    sequence::delimited,
    IResult,
};
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

fn parse_string(input: &str) -> IResult<&str, Json> {
    let (input, s) = delimited(char('"'), alphanumeric0, char('"'))(input)?;
    Ok((input, Json::String(s.to_string())))
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
    use crate::{parse_boolean, parse_null, parse_string, Json};

    #[test]
    fn test_parse_string() -> Result<(), Box<dyn std::error::Error>> {
        let (_, json) = parse_string("\"aaa\"")?;
        assert_eq!(json, Json::String("aaa".to_string()));

        let (_, json) = parse_string("\"123\"")?;
        assert_eq!(json, Json::String("123".to_string()));

        Ok(())
    }

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
