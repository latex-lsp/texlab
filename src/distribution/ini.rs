use std::collections::HashMap;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Value<'a> {
    String(&'a str),
    Array(Vec<&'a str>),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Section<'a> {
    pub name: &'a str,
    pub entries: HashMap<&'a str, Value<'a>>,
}

impl<'a> Section<'a> {
    pub fn get_string_value(&self, key: &str) -> Option<&str> {
        match &self.entries.get(key)? {
            Value::String(value) => Some(value),
            Value::Array(_) => None,
        }
    }

    pub fn get_array_value(&self, key: &str) -> Option<&Vec<&str>> {
        match &self.entries.get(key)? {
            Value::String(_) => None,
            Value::Array(values) => Some(values),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Ini<'a> {
    pub sections: Vec<Section<'a>>,
}

pub mod parser {
    use super::*;
    use nom::branch::alt;
    use nom::bytes::complete::{tag, take_till1};
    use nom::character::complete::{multispace1, not_line_ending};
    use nom::combinator::{map, opt};
    use nom::multi::{fold_many0, many0, many1};
    use nom::IResult;

    pub fn parse(input: &str) -> IResult<&str, Ini> {
        let (input, _) = trivia(input)?;
        let (input, sections) = many0(section)(input)?;
        Ok((input, Ini { sections }))
    }

    fn section(input: &str) -> IResult<&str, Section> {
        let (input, _) = tag("[")(input)?;
        let (input, name) = ident(input)?;
        let (input, _) = tag("]")(input)?;
        let (input, _) = trivia(input)?;
        let (input, entries) = many1(entry)(input)?;
        let (input, _) = trivia(input)?;
        Ok((
            input,
            Section {
                name,
                entries: entries.into_iter().collect(),
            },
        ))
    }

    fn entry(input: &str) -> IResult<&str, (&str, Value)> {
        let (input, key) = ident(input)?;
        let (input, is_array) = map(opt(tag("[]")), |x| x.is_some())(input)?;
        let (input, _) = tag("=")(input)?;
        let (input, value) = not_line_ending(input)?;
        let (input, _) = trivia(input)?;

        let (input, value) = if is_array {
            let add_to_vec = |mut acc: Vec<_>, item| {
                acc.push(item);
                acc
            };

            let (input, values) = fold_many0(array_value(key), vec![value], add_to_vec)(input)?;
            (input, Value::Array(values))
        } else {
            (input, Value::String(value))
        };

        Ok((input, (key, value)))
    }

    fn array_value(key: &str) -> impl Fn(&str) -> IResult<&str, &str> + '_ {
        move |input: &str| {
            let (input, _) = tag(key)(input)?;
            let (input, _) = tag("[]")(input)?;
            let (input, _) = tag("=")(input)?;
            let (input, value) = not_line_ending(input)?;
            let (input, _) = trivia(input)?;
            Ok((input, value))
        }
    }

    fn ident(input: &str) -> IResult<&str, &str> {
        take_till1(|c| ";[]=\r\n".contains(c))(input)
    }

    fn trivia(input: &str) -> IResult<&str, ()> {
        map(many0(alt((multispace1, comment))), |_| ())(input)
    }

    fn comment(input: &str) -> IResult<&str, &str> {
        let (input, _) = tag(";")(input)?;
        not_line_ending(input)
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_parse_comment() {
            let (input, result) = comment(";foo").unwrap();
            assert!(input.is_empty());
            assert_eq!(result, "foo");
        }

        #[test]
        fn test_parse_trivia() {
            let (input, _) = trivia(";foo\r\n \r\n").unwrap();
            assert!(input.is_empty());
        }

        #[test]
        fn test_parse_ident() {
            let id = "_foo-bar";
            let (input, result) = ident(id).unwrap();
            assert!(input.is_empty());
            assert_eq!(result, id);
        }

        #[test]
        fn test_parse_entry_string() {
            let input = format!("foo=[bar]\r\n");
            let (input, (key, value)) = entry(&input).unwrap();
            assert!(input.is_empty());
            assert_eq!(key, "foo");
            match value {
                Value::String(value) => assert_eq!(value, "[bar]"),
                Value::Array(_) => panic!("Expected String"),
            };
        }

        #[test]
        fn test_parse_entry_array() {
            let input = format!("foo[]=bar\r\n");
            let (input, (key, value)) = entry(&input).unwrap();
            assert!(input.is_empty());
            assert_eq!(key, "foo");
            match value {
                Value::String(_) => panic!("Expected Array"),
                Value::Array(values) => {
                    assert_eq!(values.len(), 1);
                    assert_eq!(values[0], "bar");
                }
            };
        }

        #[test]
        fn test_parse_section() {
            let (input, result) = section("[foo]\r\nfoo[]=bar\r\nfoo[]=baz\r\nbaz=\r\n").unwrap();
            assert!(input.is_empty());
            assert_eq!(result.entries.len(), 2);
            assert!(result.entries.contains_key("foo"));
            assert!(result.entries.contains_key("baz"));

            match &result.entries["foo"] {
                Value::String(_) => {
                    panic!("Expected Array");
                }
                Value::Array(values) => {
                    assert_eq!(values.len(), 2);
                    assert_eq!(values[0], "bar");
                    assert_eq!(values[1], "baz");
                }
            };
        }

        #[test]
        fn test_parse() {
            let (input, result) =
                parse(";foo\r\n[foo]\r\nfoo=bar\r\n[bar]\r\nbar=baz\r\n").unwrap();
            assert!(input.is_empty());
            assert_eq!(result.sections.len(), 2);
        }
    }
}
