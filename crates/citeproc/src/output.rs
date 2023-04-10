use std::ops::Add;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
pub enum Punct {
    Nothing,
    Space,
    Comma,
    Dot,
    Colon,
}

impl Punct {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Nothing => "",
            Self::Space => " ",
            Self::Comma => ", ",
            Self::Dot => ". ",
            Self::Colon => ": ",
        }
    }
}

impl Add for Punct {
    type Output = Punct;

    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Self::Nothing, punct) | (punct, Self::Nothing) => punct,
            (_, Self::Colon) | (Self::Colon, _) => Self::Colon,
            (Self::Space, Self::Space) => Self::Space,
            (Self::Space | Self::Comma | Self::Dot, Self::Comma)
            | (Self::Comma, Self::Space | Self::Dot) => Self::Comma,
            (Self::Space | Self::Dot, Self::Dot) | (Self::Dot, Self::Space) => Self::Dot,
        }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Hash)]
pub enum Inline {
    Regular(String),
    Italic(String),
    Quoted(String),
    Link { url: String, alt: String },
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Hash, Default)]
pub struct InlineBuilder {
    items: Vec<(Inline, Punct)>,
}

impl InlineBuilder {
    pub fn push(&mut self, inline: Inline, leading: Punct, trailing: Punct) {
        if let Some((_, last)) = self.items.last_mut() {
            *last = *last + leading;
        }

        self.items.push((inline, trailing));
    }

    pub fn finish(mut self) -> impl Iterator<Item = (Inline, Punct)> {
        if let Some((_, last)) = self.items.last_mut() {
            *last = Punct::Nothing;
        }

        self.items.into_iter()
    }
}
