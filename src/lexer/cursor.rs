
pub(crate) struct Cursor<'a>{
    line: &'a str,
    selected: usize,
    remainder: usize,
}

impl <'a> Cursor<'a> {

    /// New cursor
    pub(crate) fn new(line: &'a str) -> Self {
        Cursor { line: line, selected: 0, remainder: 0 }
    }

    /// Returns the current selected text
    pub(crate) fn get_selceted(&self) -> &'a str {
        if self.selected == self.remainder {
            return "";
        }
        &self.line[self.selected..self.remainder]
    }

    /// Returns the remaining unselected text
    pub(crate) fn get_remainder(&self) -> &'a str {
        if !self.has_remaining() {
            return "";
        }
        &self.line[self.remainder..]
    }

    /// Returns the start index of current selected text
    pub(crate) fn get_start(&self) -> usize {
        self.selected
    }

    /// Returns the end index of current selected text
    pub(crate) fn get_end(&self) -> usize {
        self.remainder - 1
    }

    /// Returns the start index of the remainder text
    pub(crate) fn get_start_remainder(&self) -> usize {
        self.remainder
    }

    pub(crate) fn has_remaining(&self) -> bool {
        self.remainder < self.line.len()
    }

    /// Select the remaining part and return it (or empty str if end is reached))
    pub(crate) fn select_remainder(&mut self) -> &str {
        let remainder_str = self.get_remainder();
        self.remainder += remainder_str.len();
        remainder_str
    }

    /// Select the next character and return it (or `\0` if end is reached)
    pub(crate) fn select_next(&mut self) -> char {
        match self.get_remainder().chars().next() {
            Some(c) => {
                self.remainder += c.len_utf8();
                c
            }
            None => '\0',
        }
    }

    /// Select the next `n` characters  and return it (or empty str if end is reached)
    pub(crate) fn select_next_n(&mut self, n: usize) -> &str {
        
        let n_bytes=self.get_remainder().chars().take(n).fold(
            0,
            |acc, c|{ acc+c.len_utf8() }
        );

        if n_bytes > self.get_remainder().len() {
            return self.select_remainder();
        }

        let new_start = self.remainder;
        self.remainder += n_bytes;
        &self.line[new_start..self.remainder]
    }

    /// Select if it starts with a certain string and return true if it is selected
    pub(crate) fn select_if_starts_with(&mut self, s: &str) -> bool {
        if self.get_remainder().starts_with(s) {
            self.remainder += s.len();
            return true;
        }
        false
    }

    /// Select while `predicate` on the next character to select is matched
    /// 
    /// The `predicate` is on the character to select and its start index
    pub(crate) fn select_next_while<F>(&mut self, mut predicate: F) where F: FnMut(char, usize) -> bool {

        let mut chars = self.get_remainder().chars();

        while chars.next().is_some_and(|c|{ predicate(c, self.remainder) }){
            self.select_next();
        }
    }

    /// Select while `predicate` on the remaining is matched
    /// 
    /// The `predicate` is on the remaining and its start index
    pub(crate) fn select_while<F>(&mut self, mut predicate: F) where F: FnMut(&str, usize) -> isize{

        while self.has_remaining(){
            
            let n = predicate(self.get_remainder(), self.remainder);

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
        let start = self.selected;
        let end = self.remainder - 1;
        let selected = self.get_selceted();
        self.selected = self.remainder;
        (selected, start, end)
    }

}

#[cfg(test)]
mod tests{
    use super::Cursor;
    
    #[test]
    fn test_cursor(){
        let mut cursor = Cursor::new("لغة نظم");

        assert_eq!(cursor.select_next_n(3), "لغة");
        assert_eq!(cursor.select_next(), ' ');
        assert_eq!(cursor.get_remainder(), "نظم");
        let (cut, start, end) = cursor.cut();
        assert_eq!(cut, "لغة ");
        assert_eq!(0, start);
        assert_eq!("لغة ".len() - 1, end);
        assert_eq!(cursor.get_remainder(), "نظم");
        assert_eq!(cursor.select_next_n(3), "نظم");
        assert_eq!(cursor.get_selceted(), "نظم");
        assert!(!cursor.has_remaining())
    }
}