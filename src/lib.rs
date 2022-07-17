use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{alphanumeric0, char, digit1, multispace0},
    combinator::{eof, opt, recognize},
    multi::separated_list0,
    sequence::{delimited, tuple},
    IResult,
};
use std::{collections::HashMap, error::Error, fmt};

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
    pub fn parse<'a>(input: &'a str) -> Result<Json, Box<dyn Error + 'a>> {
        let (_, (json, _)) = tuple((alt((parse_array, parse_object)), eof))(input)?;
        Ok(json)
    }
}

fn parse_string(input: &str) -> IResult<&str, Json> {
    let (input, s) = delimited(char('"'), alphanumeric0, char('"'))(input)?;
    Ok((input, Json::String(s.to_string())))
}

fn parse_number(input: &str) -> IResult<&str, Json> {
    fn parse_float(input: &str) -> IResult<&str, f64> {
        let (input, n) = recognize(tuple((digit1, char('.'), digit1)))(input)?;
        Ok((input, n.parse().unwrap()))
    }

    fn parse_integer(input: &str) -> IResult<&str, f64> {
        let (input, n) = digit1(input)?;
        Ok((input, n.parse().unwrap()))
    }

    let (input, op) = opt(char('-'))(input)?;
    let (input, n) = alt((parse_float, parse_integer))(input)?;
    Ok((input, Json::Number(if op.is_some() { -n } else { n })))
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

fn parse_array(input: &str) -> IResult<&str, Json> {
    fn parse_by_comma(input: &str) -> IResult<&str, Vec<Json>> {
        let (input, items) = separated_list0(
            ws_char(','),
            alt((
                parse_string,
                parse_number,
                parse_boolean,
                parse_array,
                parse_object,
                parse_null,
            )),
        )(input)?;
        Ok((input, items))
    }

    let (input, items) = delimited(ws_char('['), parse_by_comma, ws_char(']'))(input)?;
    Ok((input, Json::Array(items)))
}

fn parse_object(input: &str) -> IResult<&str, Json> {
    fn parse_key_value(input: &str) -> IResult<&str, (String, Json)> {
        let (input, k) = delimited(char('"'), alphanumeric0, char('"'))(input)?;
        let (input, _) = ws_char(':')(input)?;
        let (input, v) = alt((
            parse_string,
            parse_number,
            parse_boolean,
            parse_array,
            parse_object,
            parse_null,
        ))(input)?;
        Ok((input, (k.to_string(), v)))
    }

    let (input, _) = ws_char('{')(input)?;
    let (input, key_value_list) = separated_list0(ws_char(','), parse_key_value)(input)?;
    let (input, _) = ws_char('}')(input)?;
    Ok((input, Json::Object(key_value_list.into_iter().collect())))
}

fn parse_null(input: &str) -> IResult<&str, Json> {
    let (input, _) = tag("null")(input)?;
    Ok((input, Json::Null))
}

fn ws_char<'a>(c: char) -> impl FnMut(&'a str) -> IResult<&'a str, char> {
    delimited(multispace0, char(c), multispace0)
}

impl fmt::Display for Json {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Json::String(s) => write!(f, "\"{}\"", s),
            Json::Number(n) => write!(f, "{}", n),
            Json::Boolean(b) => write!(f, "{}", b),
            Json::Null => write!(f, "null"),
            _ => unimplemented!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::Json;
    use std::collections::HashMap;

    #[test]
    fn test_parse() {
        let json = Json::parse(r#"{ "name": "Tanaka", "age": 26 }"#).unwrap();

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
            r#"{"persons": [{ "name": "Tanaka", "age": 26 }, { "name": "Yamada", "age": 28 }]}"#,
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
}
