use std::borrow::Cow;

use rustc_hash::FxHashMap;

pub fn replace_placeholders<'a>(
    input: &'a str,
    placeholders: &FxHashMap<char, &str>,
) -> Cow<'a, str> {
    match input
        .strip_prefix('"')
        .and_then(|input| input.strip_suffix('"'))
    {
        Some(input) => Cow::Borrowed(input),
        None => {
            let mut output = String::new();
            let mut chars = input.chars();
            while let Some(ch) = chars.next() {
                if ch == '%' {
                    match chars.next() {
                        Some(key) => match placeholders.get(&key) {
                            Some(value) => output.push_str(&value),
                            None => output.push(key),
                        },
                        None => output.push('%'),
                    };
                } else {
                    output.push(ch);
                }
            }

            Cow::Owned(output)
        }
    }
}
