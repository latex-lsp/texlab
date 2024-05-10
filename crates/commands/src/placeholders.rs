use rustc_hash::FxHashMap;

pub fn replace_placeholders(args: &[String], pairs: &[(char, &str)]) -> Vec<String> {
    let map = FxHashMap::from_iter(pairs.iter().copied());
    args.iter()
        .map(|input| {
            let quoted = input
                .strip_prefix('"')
                .and_then(|input| input.strip_suffix('"'));

            match quoted {
                Some(output) => String::from(output),
                None => {
                    let mut output = String::new();
                    let mut chars = input.chars();
                    while let Some(ch) = chars.next() {
                        if ch == '%' {
                            match chars.next() {
                                Some(key) => match map.get(&key) {
                                    Some(value) => output.push_str(value),
                                    None => output.push(key),
                                },
                                None => output.push('%'),
                            };
                        } else {
                            output.push(ch);
                        }
                    }

                    output
                }
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::replace_placeholders;

    #[test]
    fn test_quoted() {
        let output = replace_placeholders(
            &["foo".into(), "\"%f\"".into(), "%%f".into(), "%fbar".into()],
            &[('f', "foo")],
        );

        assert_eq!(output, vec!["foo", "%f", "%f", "foobar"]);
    }
}
