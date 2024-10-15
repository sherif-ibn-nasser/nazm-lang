use std::{collections::HashMap, ops::Index};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PoolIdx(usize);

mod private {
    pub trait DisplayTableStateGuard {}
}

pub trait DisplayTableState: private::DisplayTableStateGuard {}

pub struct Init(HashMap<String, PoolIdx>);

pub struct Built(Vec<String>);

impl private::DisplayTableStateGuard for Init {}
impl private::DisplayTableStateGuard for Built {}

impl DisplayTableState for Init {}
impl DisplayTableState for Built {}

pub struct DataPool<S: DisplayTableState> {
    state: S,
}

impl DataPool<Init> {
    pub fn new() -> Self {
        Self {
            state: Init(HashMap::new()),
        }
    }

    pub fn get(&mut self, s: &str) -> PoolIdx {
        let option = self.state.0.get(s);

        if let Some(display_idx) = option {
            *display_idx
        } else {
            let len = self.state.0.len();
            self.state.0.insert(s.to_string(), PoolIdx(len));
            PoolIdx(len)
        }
    }

    pub fn build(self) -> DataPool<Built> {
        let mut table = Vec::with_capacity(self.state.0.len());

        unsafe { table.set_len(self.state.0.len()) };

        for (str, idx) in self.state.0 {
            table[idx.0] = str;
        }

        DataPool {
            state: Built(table),
        }
    }
}

impl Index<PoolIdx> for DataPool<Built> {
    type Output = str;

    fn index(&self, index: PoolIdx) -> &Self::Output {
        &self.state.0[index.0]
    }
}
