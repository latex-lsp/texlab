use std::str::FromStr;

use rowan::{ast::AstNode, NodeOrToken};
use rustc_hash::FxHashSet;
use strum::EnumString;

use crate::syntax::bibtex::{
    Accent, Command, CurlyGroup, HasAccentName, HasCommandName, HasName, HasValue, HasWord, Join,
    Literal, QuoteGroup, Root, SyntaxKind::*, SyntaxToken, Value,
};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, EnumString)]
#[strum(ascii_case_insensitive)]
pub enum TextField {
    Abstract,
    Addendum,
    BookSubtitle,
    BookTitle,
    BookTitleAddon,
    Chapter,
    Doi,
    EditorType,
    EditorTypeA,
    EditorTypeB,
    EditorTypeC,
    Eid,
    Eprint,
    EprintClass,
    EprintType,
    EventTitle,
    EventTitleAddon,
    Holder,
    HowPublished,
    Isbn,
    Issn,
    Issue,
    IssueSubtitle,
    IssueTitle,
    IssueTitleAddon,
    Journal,
    JournalSubtitle,
    JournalTitle,
    JournalTitleAddon,
    Language,
    Location,
    MainTitle,
    MainSubtitle,
    MainTitleAddon,
    Note,
    OrigLanguage,
    Publisher,
    Pubstate,
    Series,
    Subtitle,
    Title,
    TitleAddon,
    Type,
    Unknown,
    Url,
    Venue,
    Version,
}

impl TextField {
    pub fn parse(input: &str) -> Option<Self> {
        Self::from_str(input).ok()
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Hash, Default)]
pub struct TextFieldData {
    pub text: String,
}

impl TextFieldData {
    pub fn parse(value: &Value) -> Option<Self> {
        let mut builder = TextFieldDataBuilder::default();
        builder.visit_value(&value)?;
        Some(builder.data)
    }
}

#[derive(Default)]
struct TextFieldDataBuilder {
    data: TextFieldData,
    string_stack: FxHashSet<String>,
}

impl TextFieldDataBuilder {
    fn visit_value(&mut self, value: &Value) -> Option<()> {
        match value {
            Value::Literal(lit) => {
                self.visit_literal(lit)?;
            }
            Value::CurlyGroup(group) => {
                self.visit_curly_group(group)?;
            }
            Value::QuoteGroup(group) => {
                self.visit_quote_group(group)?;
            }
            Value::Join(join) => {
                self.visit_join(join)?;
            }
            Value::Accent(accent) => {
                let _ = self.visit_accent(accent);
            }
            Value::Command(command) => {
                let _ = self.visit_command(command);
            }
        };

        Some(())
    }

    fn visit_literal(&mut self, lit: &Literal) -> Option<()> {
        if lit
            .name_token()
            .and_then(|name| self.visit_string_reference(&name))
            .is_none()
        {
            lit.syntax()
                .text()
                .for_each_chunk(|text| self.data.text.push_str(text));
        }

        Some(())
    }

    fn visit_string_reference(&mut self, name: &SyntaxToken) -> Option<()> {
        let root = Root::cast(name.ancestors().last()?)?;
        let name = name.text();

        let value = root
            .strings()
            .filter(|string| {
                string
                    .name_token()
                    .map_or(false, |token| token.text() == name)
            })
            .find_map(|string| string.value())?;

        if !self.string_stack.insert(name.to_string()) {
            return None;
        }

        let _ = self.visit_value(&value);
        self.string_stack.remove(name);
        Some(())
    }

    fn visit_curly_group(&mut self, group: &CurlyGroup) -> Option<()> {
        for child in group.syntax().children_with_tokens() {
            match child {
                NodeOrToken::Node(node) => {
                    self.visit_value(&Value::cast(node)?)?;
                }
                NodeOrToken::Token(token) => {
                    match token.kind() {
                        L_CURLY | R_CURLY => (),
                        WHITESPACE | NBSP => self.data.text.push(' '),
                        _ => self.data.text.push_str(token.text()),
                    };
                }
            };
        }

        Some(())
    }

    fn visit_quote_group(&mut self, group: &QuoteGroup) -> Option<()> {
        for child in group.syntax().children_with_tokens() {
            match child {
                NodeOrToken::Node(node) => {
                    self.visit_value(&Value::cast(node)?)?;
                }
                NodeOrToken::Token(token) => {
                    match token.kind() {
                        QUOTE => (),
                        WHITESPACE | NBSP => self.data.text.push(' '),
                        _ => self.data.text.push_str(token.text()),
                    };
                }
            };
        }

        Some(())
    }

    fn visit_join(&mut self, join: &Join) -> Option<()> {
        if let Some(left) = join.left_value() {
            self.visit_value(&left)?;
        }

        if let Some(right) = join.right_value() {
            self.visit_value(&right)?;
        }

        Some(())
    }

    fn visit_accent(&mut self, accent: &Accent) -> Option<()> {
        let name = accent.accent_name_token()?;
        let word = accent.word_token()?;

        let mut chars = word.text().chars();
        let a = chars.next()?;

        if chars.next().is_some() {
            self.data.text.push_str(word.text());
        } else {
            let b = match name.text() {
                r#"\`"# => '\u{0300}',
                r#"\'"# => '\u{0301}',
                r#"\^"# => '\u{0302}',
                r#"\""# => '\u{0308}',
                r#"\H"# => '\u{030B}',
                r#"\~"# => '\u{0303}',

                r#"\c"# => '\u{0327}',
                r#"\k"# => '\u{0328}',
                r#"\="# => '\u{0304}',
                r#"\b"# => '\u{0331}',
                r#"\."# => '\u{0307}',
                r#"\d"# => '\u{0323}',
                r#"\r"# => '\u{030A}',
                r#"\u"# => '\u{0306}',
                r#"\v"# => '\u{030C}',
                _ => '\u{0000}',
            };

            match unicode_normalization::char::compose(a, b) {
                Some(c) => self.data.text.push(c),
                None => self.data.text.push_str(word.text()),
            };
        }

        Some(())
    }

    fn visit_command(&mut self, command: &Command) -> Option<()> {
        let name = command.command_name_token()?;
        let replacement = match name.text() {
            r#"\l"# => "\u{0142}",
            r#"\o"# => "\u{00F8}",
            r#"\i"# => "\u{0131}",
            r#"\&"# => "&",
            r#"\$"# => "$",
            r#"\{"# => "{",
            r#"\}"# => "}",
            r#"\%"# => "%",
            r#"\#"# => "#",
            r#"\_"# => "_",
            r#"\ "# | r#"\,"# | r#"\;"# => " ",
            r#"\hyphen"# => "-",
            r#"\TeX"# => "TeX",
            r#"\LaTeX"# => "LaTeX",
            text => text,
        };

        self.data.text.push_str(replacement);
        Some(())
    }
}
