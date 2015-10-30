use std::ops::{Neg, BitXor};
use std::fmt;

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd,
         Debug)]
pub enum Sign {
    Neg = -1,
    Pos = 1,
}
impl fmt::Display for Sign {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Sign::Pos => write!(f, "+"),
            Sign::Neg => write!(f, "-"),
        }
    }
}

impl Neg for Sign {
    type Output = Sign;
    fn neg(self) -> Sign {
        match self {
            Sign::Neg => Sign::Pos,
            Sign::Pos => Sign::Neg,
        }
    }
}
impl BitXor for Sign {
    type Output = Sign;
    fn bitxor(self, other: Sign) -> Sign {
        if self == other {
            Sign::Pos
        } else {
            Sign::Neg
        }
    }
}
