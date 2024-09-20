use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DisplayIdx(usize);

mod private {
    pub trait DisplayTableStateGuard {}
}

pub trait DisplayTableState: private::DisplayTableStateGuard {}

pub struct Init(HashMap<String, DisplayIdx>);

pub struct Built(Vec<String>);

impl private::DisplayTableStateGuard for Init {}
impl private::DisplayTableStateGuard for Built {}

impl DisplayTableState for Init {}
impl DisplayTableState for Built {}

pub struct DisplayTable<S: DisplayTableState> {
    state: S,
}

impl DisplayTable<Init> {
    pub fn new() -> Self {
        Self {
            state: Init(HashMap::new()),
        }
    }

    pub fn get(&mut self, s: &str) -> DisplayIdx {
        let option = self.state.0.get(s);

        if let Some(display_idx) = option {
            *display_idx
        } else {
            let len = self.state.0.len();
            self.state.0.insert(s.to_string(), DisplayIdx(len));
            DisplayIdx(len)
        }
    }

    pub fn build(self) -> DisplayTable<Built> {
        let mut table = Vec::with_capacity(self.state.0.len());

        unsafe { table.set_len(self.state.0.len()) };

        for (str, idx) in self.state.0 {
            table[idx.0] = str;
        }

        DisplayTable {
            state: Built(table),
        }
    }
}
