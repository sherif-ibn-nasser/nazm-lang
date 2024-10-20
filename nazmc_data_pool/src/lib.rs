use std::{collections::HashMap, ops::Index};

use itertools::Itertools;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PoolIdx(usize);

impl PoolIdx {
    pub const MAIN: Self = Self(0);
    pub const LAMBDA_IMPLICIT_PARAM: Self = Self(1);
}

mod private {
    pub trait DisplayTableStateGuard {}
}

pub trait DisplayTableState: private::DisplayTableStateGuard {}

#[derive(Clone)]
pub struct Init(HashMap<String, PoolIdx>);

#[derive(Clone)]
pub struct Built(Vec<String>);

impl private::DisplayTableStateGuard for Init {}
impl private::DisplayTableStateGuard for Built {}

impl DisplayTableState for Init {}
impl DisplayTableState for Built {}

#[derive(Clone)]
pub struct DataPool<S>
where
    S: DisplayTableState + Clone,
{
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

        // FIXME: Optimize
        for (str, idx) in self.state.0 {
            if idx.0 >= table.len() {
                for _ in table.len()..=idx.0 {
                    table.push("".to_string());
                }
            }
            table[idx.0] = str;
        }

        // unsafe { table.set_len(self.state.0.len()) };

        // for (str, idx) in self.state.0 {
        //     table[idx.0] = str;
        // }

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
