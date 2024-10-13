#![allow(unreachable_pub)]

pub mod blue;
pub mod textinputf;

mod _private {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct NonExhaustive;
}
