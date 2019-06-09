pub mod parser {
    use crate::distribution::ini::*;
    use nom::bytes::complete::*;
    use nom::character::complete::*;
    use nom::combinator::*;
    use nom::multi::*;
    use nom::sequence::*;
    use nom::IResult;

    pub fn parse(input: &str) -> IResult<&str, Ini> {
        let (input, _) = trivia(input)?;
        let (input, sections) = many0(section)(input)?;
        Ok((input, Ini { sections }))
    }

    fn section(input: &str) -> IResult<&str, Section> {
        let (input, _) = tag("name")(input)?;
        let (input, _) = space1(input)?;
        let (input, name) = not_line_ending(input)?;
        let (input, _) = line_ending(input)?;
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
        let (input, ((key, value), next_key)) = peek(tuple((key_value, alpha0)))(input)?;
        let (input, value) = if key == next_key {
            let (input, values) = many1(array_value(key))(input)?;
            (input, Value::Array(values))
        } else if value.contains("size=") {
            let (input, _) = key_value(input)?;
            let (input, values) = many1(file_array_element)(input)?;
            (input, Value::Array(values))
        } else {
            let (input, _) = key_value(input)?;
            (input, Value::String(value))
        };

        Ok((input, (key, value)))
    }

    fn key_value(input: &str) -> IResult<&str, (&str, &str)> {
        let (input, key) = not_space_and_line_ending(input)?;
        let (input, _) = space1(input)?;
        let (input, value) = not_line_ending(input)?;
        let (input, _) = line_ending(input)?;
        Ok((input, (key, value)))
    }

    fn array_value(key: &str) -> impl Fn(&str) -> IResult<&str, &str> + '_ {
        move |input: &str| {
            let (input, _) = tuple((tag(key), space1))(input)?;
            let (input, value) = not_line_ending(input)?;
            let (input, _) = line_ending(input)?;
            Ok((input, value))
        }
    }

    fn file_array_element(input: &str) -> IResult<&str, &str> {
        let (input, _) = space1(input)?;
        let (input, file) = not_space_and_line_ending(input)?;
        let (input, _) = space0(input)?;
        let (input, _) = opt(tuple((tag("details="), not_line_ending)))(input)?;
        let (input, _) = line_ending(input)?;
        Ok((input, file))
    }

    fn not_space_and_line_ending(input: &str) -> IResult<&str, &str> {
        take_till1(|c| " \r\n".contains(c))(input)
    }

    fn trivia(input: &str) -> IResult<&str, ()> {
        map(many0(multispace1), |_| ())(input)
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_parse_trivia() {
            let (input, _) = trivia("\r\n  \r\n").unwrap();
            assert!(input.is_empty());
        }

        #[test]
        fn test_parse_entry_string() {
            let input = format!("foo-bar baz\r\n");
            let (input, (key, value)) = entry(&input).unwrap();
            assert!(input.is_empty());
            assert_eq!(key, "foo-bar");
            match value {
                Value::String(value) => assert_eq!(value, "baz"),
                Value::Array(_) => panic!("Expected String"),
            };
        }

        #[test]
        fn test_parse_entry_array() {
            let input = format!("foo bar\r\nfoo baz\r\n");
            let (input, (key, value)) = entry(&input).unwrap();
            assert!(input.is_empty());
            assert_eq!(key, "foo");
            match value {
                Value::String(_) => panic!("Expected Array"),
                Value::Array(values) => {
                    assert_eq!(values.len(), 2);
                    assert_eq!(values[0], "bar");
                    assert_eq!(values[1], "baz");
                }
            };
        }

        #[test]
        fn test_parse_entry_file_array() {
            let input = format!("foo size=1\r\n ./bar.tex\r\n ./baz.tex\r\n");
            let (input, (key, value)) = entry(&input).unwrap();
            assert!(input.is_empty());
            assert_eq!(key, "foo");
            match value {
                Value::String(_) => panic!("Expected Array"),
                Value::Array(values) => {
                    assert_eq!(values.len(), 2);
                    assert_eq!(values[0], "./bar.tex");
                    assert_eq!(values[1], "./baz.tex");
                }
            };
        }

        #[test]
        fn test_parse_section() {
            let (input, result) = section("name foo\r\nfoo bar\r\n\r\n").unwrap();
            assert!(input.is_empty());
            assert_eq!(result.entries.len(), 1);
            assert!(result.entries.contains_key("foo"));

            match &result.entries["foo"] {
                Value::String(value) => assert_eq!(*value, "bar"),
                Value::Array(_) => panic!("Expected String"),
            };
        }

        #[test]
        fn test_parse() {
            let (input, result) =
                parse("name foo\r\nbaz baz\r\n\r\nname bar\r\nbaz baz\r\n\r\n\r\n").unwrap();

            assert!(input.is_empty());
            assert_eq!(result.sections.len(), 2);
        }
    }
}
