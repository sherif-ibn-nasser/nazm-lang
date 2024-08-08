
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

    /// Returns the remaining unselected text
    pub(crate) fn get_remaining(&self) -> &'a str {
        &self.line[self.end..]
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
        !self.get_remaining().is_empty()
    }

    /// Select the remaining part and return it (or empty str if end is reached))
    pub(crate) fn select_remaining(&mut self) -> &str {
        let remaining = self.get_remaining();
        self.end += remaining.len();
        remaining
    }

    /// Select the next character and return it (or `\0` if end is reached)
    pub(crate) fn select_next(&mut self) -> char {
        if !self.has_remaining() {
            return char::default();
        }
        self.end += 1;
        self.last_selected()
    }

    /// Select the next `n` characters  and return it (or empty str if end is reached)
    pub(crate) fn select_next_n(&mut self, n: usize) -> &str {
        let new_start = self.end+1;
        if n > self.get_remaining().len() { 
            return self.select_remaining();
        }
        self.end += n;
        &self.get_selceted()[new_start..]
    }

    /// Select if it starts with a certain string and return true if it is selected
    pub(crate) fn select_if_starts_with(&mut self, s: &str) -> bool {
        if self.get_remaining().starts_with(s) {
            self.end += s.len();
            return true;
        }
        false
    }

    /// Select while `predicate` on the next character to select is matched
    /// 
    /// The `predicate` is on the character to select and its start index
    pub(crate) fn select_next_while<F>(&mut self, mut predicate: F) where F: FnMut(char, usize) -> bool{
        while 
            self.has_remaining()
            &&
            predicate(self.get_remaining().chars().next().unwrap_or_default(), self.end + 1)
        {
            self.select_next();
        }
    }

    /// Select while `predicate` on the remaining is matched
    /// 
    /// The `predicate` is on the remaining and its start index
    pub(crate) fn select_while<F>(&mut self, mut predicate: F) where F: FnMut(&str, usize) -> isize{
        while self.has_remaining(){
            
            let n = predicate(self.get_remaining(), self.end + 1);

            if n < 1 {
                break;
            }

            self.select_next_n(n as usize);
        }
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