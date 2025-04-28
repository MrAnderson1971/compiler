use std::fmt::{Display, Formatter};

pub(crate) type Position = (i32, String);

pub(crate) type Identifier = String;

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Const {
    ConstInt(u32),
    ConstLong(u64),
    ConstUInt(u32),
    ConstULong(u64),
}

impl Const {
    pub(crate) fn size(&self) -> i32 {
        match self {
            Const::ConstInt(_) | Const::ConstUInt(_) => 4,
            Const::ConstLong(_) | Const::ConstULong(_) => 8,
        }
    }
}

impl Display for Const {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self {
            Const::ConstInt(i) => write!(f, "{}", i),
            Const::ConstLong(i) => write!(f, "{}", i),
            Const::ConstUInt(i) => write!(f, "{}", i),
            Const::ConstULong(i) => write!(f, "{}", i),
        }
    }
}

impl From<u32> for Const {
    fn from(v: u32) -> Self {
        Const::ConstInt(v)
    }
}

impl From<u64> for Const {
    fn from(v: u64) -> Self {
        Const::ConstLong(v)
    }
}
