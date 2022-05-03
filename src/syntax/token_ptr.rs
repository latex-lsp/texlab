pub struct TokenPtr<'a, T> {
    buf: Vec<(T, &'a str)>,
}

impl<'a, T> From<Vec<(T, &'a str)>> for TokenPtr<'a, T> {
    fn from(mut buf: Vec<(T, &'a str)>) -> Self {
        buf.reverse();
        Self { buf }
    }
}

impl<'a, T: Eq + Copy> TokenPtr<'a, T> {
    #[must_use]
    pub fn at(&self, kind: T) -> bool {
        self.buf.last().map_or(false, |(k, _)| *k == kind)
    }

    #[must_use]
    pub fn at_cond(&self, predicate: impl FnOnce(T) -> bool) -> bool {
        self.buf.last().map_or(false, |(k, _)| predicate(*k))
    }

    #[must_use]
    pub fn current(&self) -> Option<T> {
        self.buf.last().map(|(k, _)| *k)
    }

    #[must_use]
    #[allow(dead_code)]
    pub fn nth(&self, n: usize) -> Option<T> {
        self.buf.get(self.buf.len() - n).map(|(k, _)| *k)
    }

    #[must_use]
    pub fn bump(&mut self) -> (T, &'a str) {
        self.buf.pop().unwrap()
    }
}
