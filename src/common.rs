use std::fmt::{Display, Formatter};

pub(crate) type Position = (i32, String);

pub(crate) type Identifier = String;

#[derive(Debug, Clone)]
pub(crate) enum Const {
    ConstInt(i32),
    ConstLong(i64),
}

impl Display for Const {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self {
            Const::ConstInt(i) => {
                write!(f, "{}", i)
            }
            Const::ConstLong(i) => {
                write!(f, "{}", i)
            }
        }
    }
}

impl From<i32> for Const {
    fn from(v: i32) -> Self {
        Const::ConstInt(v)
    }
}

impl From<i64> for Const {
    fn from(v: i64) -> Self {
        Const::ConstLong(v)
    }
}
