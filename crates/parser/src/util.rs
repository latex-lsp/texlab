use std::ops::{Index, Range};

use logos::{Logos, Source};

pub fn lex_command_name<'a, T>(lexer: &mut logos::Lexer<'a, T>) -> &'a str
where
    T: Logos<'a>,
    T::Source: Index<Range<usize>, Output = str>,
{
    let start = lexer.span().end;
    let input = &lexer.source()[start..lexer.source().len()];

    let mut chars = input.chars().peekable();
    let Some(c) = chars.next() else {
        return "";
    };

    if c.is_whitespace() {
        return "";
    }

    lexer.bump(c.len_utf8());

    if c.is_alphanumeric() || c == '@' {
        while let Some(c) = chars.next() {
            match c {
                '*' => {
                    lexer.bump(c.len_utf8());
                    break;
                }
                c if c.is_alphanumeric() => {
                    lexer.bump(c.len_utf8());
                }
                '_' => {
                    if !matches!(chars.peek(), Some(c) if c.is_alphanumeric()) {
                        break;
                    }

                    lexer.bump(c.len_utf8());
                }
                '@' | ':' => {
                    lexer.bump(c.len_utf8());
                }
                _ => {
                    break;
                }
            }
        }
    }

    &lexer.source()[start..lexer.span().end]
}
