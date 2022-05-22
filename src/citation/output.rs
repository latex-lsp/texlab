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
            (Self::Nothing, Self::Nothing) => Self::Nothing,
            (Self::Nothing, Self::Space)
            | (Self::Space, Self::Nothing)
            | (Self::Space, Self::Space) => Self::Space,
            (Self::Nothing, Self::Comma)
            | (Self::Space, Self::Comma)
            | (Self::Comma, Self::Nothing)
            | (Self::Comma, Self::Space)
            | (Self::Comma, Self::Comma)
            | (Self::Comma, Self::Dot)
            | (Self::Dot, Self::Comma) => Self::Comma,
            (Self::Nothing, Self::Dot)
            | (Self::Space, Self::Dot)
            | (Self::Dot, Self::Nothing)
            | (Self::Dot, Self::Space)
            | (Self::Dot, Self::Dot) => Self::Dot,
            (Self::Nothing, Self::Colon)
            | (Self::Space, Self::Colon)
            | (Self::Comma, Self::Colon)
            | (Self::Dot, Self::Colon)
            | (Self::Colon, Self::Nothing)
            | (Self::Colon, Self::Space)
            | (Self::Colon, Self::Comma)
            | (Self::Colon, Self::Dot)
            | (Self::Colon, Self::Colon) => Self::Colon,
        }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Hash)]
pub enum Inline {
    Regular(String),
    Italic(String),
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
