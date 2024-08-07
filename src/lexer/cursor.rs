
pub(crate) struct Cursor<'a>{
    line: &'a str,
    start: usize,
    end: usize,
}

impl <'a> Cursor<'a> {

    /// New cursor
    pub(crate) fn new(line: &'a str) -> Self {
        Cursor { line: line, start: 0, end: 0 }
    }

    /// Returns the current selected text
    pub(crate) fn get_selceted(&self) -> &'a str {
        &self.line[self.start..=self.end]
    }

    /// Returns the start index of current selected text
    pub(crate) fn get_start(&self) -> usize {
        self.start
    }

    /// Returns the end index of current selected text
    pub(crate) fn get_end(&self) -> usize {
        self.end
    }

    pub(crate) fn has_remaining(&self) -> bool {
        self.end < self.line.len() - 1
    }

    /// Select the next character (or `\0` if end is reached) and return it
    pub(crate) fn select_next(&mut self) -> char {
        if !self.has_remaining() {
            return char::default();
        }
        self.end += 1;
        self.last_selected()
    }

    /// Returns the last selected char
    pub(crate) fn last_selected(&mut self) -> char {
        return self.get_selceted().chars().next_back().unwrap_or_default();
    }

    /// Cut the current selection and return it (to paste it elsewhere) with its range
    pub(crate) fn cut(&mut self) -> (&'a str, usize, usize) {
        let start = self.start;
        let end = self.end;
        let selected = self.get_selceted();
        self.end += 1;
        self.start = self.end;
        (selected, start, end)
    }
}