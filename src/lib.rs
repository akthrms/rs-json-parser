use anyhow;
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{alpha1, alphanumeric0, alphanumeric1, char, digit1, multispace0},
    combinator::{eof, map, opt, recognize},
    error::{Error, ErrorKind},
    multi::{many0, separated_list0},
    sequence::{delimited, tuple},
    Finish, IResult,
};
use std::{collections::HashMap, fmt};
use thiserror::Error;

#[derive(Debug, Error)]
#[error("parse error: {{ input: `{}`, code: `{}` }}", input, code.description())]
pub struct JsonParseError {
    input: String,
    code: ErrorKind,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Json {
    String(String),
    Number(f64),
    Boolean(bool),
    Array(Vec<Json>),
    Object(HashMap<String, Json>),
    Null,
}

impl Json {
    pub fn parse(input: &str) -> anyhow::Result<Json> {
        match tuple((alt((array, object)), eof))(input).finish() {
            Ok((_, (json, _))) => Ok(json),
            Err(Error { input, code }) => Err(JsonParseError {
                input: input.to_string(),
                code,
            }
            .into()),
        }
    }
}

fn string(input: &str) -> IResult<&str, Json> {
    let (input, value) = delimited(char('"'), alphanumeric0, char('"'))(input)?;
    Ok((input, Json::String(value.to_string())))
}

fn number(input: &str) -> IResult<&str, Json> {
    let (input, (unary_minus, value)) = tuple((
        opt(char('-')),
        alt((recognize(tuple((digit1, char('.'), digit1))), digit1)),
    ))(input)?;
    Ok((
        input,
        Json::Number(if unary_minus.is_some() {
            -value.parse::<f64>().unwrap()
        } else {
            value.parse::<f64>().unwrap()
        }),
    ))
}

fn boolean(input: &str) -> IResult<&str, Json> {
    let (input, value) = alt((tag("true"), tag("false")))(input)?;
    Ok((input, Json::Boolean(value.parse().unwrap())))
}

fn array(input: &str) -> IResult<&str, Json> {
    let (input, json_list) = delimited(
        ws_char('['),
        separated_list0(
            ws_char(','),
            alt((string, number, boolean, array, object, null)),
        ),
        ws_char(']'),
    )(input)?;
    Ok((input, Json::Array(json_list)))
}

fn object(input: &str) -> IResult<&str, Json> {
    let (input, key_value_list) = delimited(
        ws_char('{'),
        separated_list0(
            ws_char(','),
            map(
                tuple((
                    delimited(
                        char('"'),
                        recognize(tuple((alpha1, many0(alphanumeric1)))),
                        char('"'),
                    ),
                    ws_char(':'),
                    alt((string, number, boolean, array, object, null)),
                )),
                |(key, _, value)| (key.to_string(), value),
            ),
        ),
        ws_char('}'),
    )(input)?;
    Ok((input, Json::Object(key_value_list.into_iter().collect())))
}

fn null(input: &str) -> IResult<&str, Json> {
    let (input, _) = tag("null")(input)?;
    Ok((input, Json::Null))
}

fn ws_char<'a>(c: char) -> impl FnMut(&'a str) -> IResult<&'a str, char> {
    delimited(multispace0, char(c), multispace0)
}

impl fmt::Display for Json {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Json::String(value) => write!(f, "\"{}\"", value),
            Json::Number(value) => write!(f, "{}", value),
            Json::Boolean(value) => write!(f, "{}", value),
            Json::Array(json_list) => write!(
                f,
                "[{}]",
                json_list
                    .iter()
                    .map(|item| format!("{}", item))
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            Json::Object(json_map) => write!(
                f,
                "{{{}}}",
                json_map
                    .iter()
                    .map(|(key, value)| format!("\"{}\": {}", key, value))
                    .collect::<Vec<String>>()
                    .join(", ")
            ),
            Json::Null => write!(f, "null"),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::Json;
    use std::collections::HashMap;

    #[test]
    fn test_parse() {
        let json = Json::parse(r#"{"name": "Tanaka", "age": 26}"#).unwrap();

        assert_eq!(
            json,
            Json::Object(
                vec![
                    ("name".to_string(), Json::String("Tanaka".to_string())),
                    ("age".to_string(), Json::Number(26.0))
                ]
                .into_iter()
                .collect::<HashMap<String, Json>>()
            )
        );

        let json = Json::parse(r#"[true, false, null]"#).unwrap();

        assert_eq!(
            json,
            Json::Array(vec![Json::Boolean(true), Json::Boolean(false), Json::Null])
        );

        let json = Json::parse(
            r#"{"persons": [{"name": "Tanaka", "age": 26}, {"name": "Yamada", "age": 28}]}"#,
        )
        .unwrap();

        assert_eq!(
            json,
            Json::Object(
                vec![(
                    "persons".to_string(),
                    Json::Array(vec![
                        Json::Object(
                            vec![
                                ("name".to_string(), Json::String("Tanaka".to_string())),
                                ("age".to_string(), Json::Number(26.0)),
                            ]
                            .into_iter()
                            .collect::<HashMap<String, Json>>(),
                        ),
                        Json::Object(
                            vec![
                                ("name".to_string(), Json::String("Yamada".to_string())),
                                ("age".to_string(), Json::Number(28.0)),
                            ]
                            .into_iter()
                            .collect::<HashMap<String, Json>>(),
                        ),
                    ])
                ),]
                .into_iter()
                .collect::<HashMap<String, Json>>()
            )
        );
    }

    #[test]
    fn test_display() {
        let json = Json::Object(
            vec![
                ("name".to_string(), Json::String("Tanaka".to_string())),
                ("age".to_string(), Json::Number(26.0)),
            ]
            .into_iter()
            .collect::<HashMap<String, Json>>(),
        );

        assert_eq!(Json::parse(format!("{}", json).as_str()).unwrap(), json);

        let json = Json::Array(vec![Json::Boolean(true), Json::Boolean(false), Json::Null]);

        assert_eq!(Json::parse(format!("{}", json).as_str()).unwrap(), json);

        let json = Json::Object(
            vec![(
                "persons".to_string(),
                Json::Array(vec![
                    Json::Object(
                        vec![
                            ("name".to_string(), Json::String("Tanaka".to_string())),
                            ("age".to_string(), Json::Number(26.0)),
                        ]
                        .into_iter()
                        .collect::<HashMap<String, Json>>(),
                    ),
                    Json::Object(
                        vec![
                            ("name".to_string(), Json::String("Yamada".to_string())),
                            ("age".to_string(), Json::Number(28.0)),
                        ]
                        .into_iter()
                        .collect::<HashMap<String, Json>>(),
                    ),
                ]),
            )]
            .into_iter()
            .collect::<HashMap<String, Json>>(),
        );

        assert_eq!(Json::parse(format!("{}", json).as_str()).unwrap(), json);
    }
}
