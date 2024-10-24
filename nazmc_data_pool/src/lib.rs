use slotmap::{new_key_type, SlotMap};

pub use slotmap;

new_key_type! { pub struct IdKey; }

new_key_type! { pub struct StrKey; }

pub type IdPool = SlotMap<IdKey, String>;

pub type StrPool = SlotMap<StrKey, String>;
