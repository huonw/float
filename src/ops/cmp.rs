use {Style, Sign, Float};

use std::cmp::Ordering;

impl PartialEq for Float {
    fn eq(&self, other: &Float) -> bool {
        self.partial_cmp(other) == Some(Ordering::Equal)
    }
    fn ne(&self, other: &Float) -> bool {
        self.partial_cmp(other).map_or(false, |e| e != Ordering::Equal)
    }
}
impl PartialOrd for Float {
    fn partial_cmp(&self, other: &Float) -> Option<Ordering> {
        self.debug_assert_valid();
        other.debug_assert_valid();
        assert_eq!(self.prec, other.prec);

        Some(match (self.style, other.style) {
            (Style::NaN, _) | (_, Style::NaN) => return None,

            (Style::Infinity, Style::Infinity) => self.sign.cmp(&other.sign),
            (Style::Infinity, _) => {
                match self.sign {
                    Sign::Pos => Ordering::Greater,
                    Sign::Neg => Ordering::Less
                }
            }
            (_, Style::Infinity) => {
                match other.sign {
                    Sign::Pos => Ordering::Less,
                    Sign::Neg => Ordering::Greater
                }
            }
            // |LHS| > |RHS| guaranteed
            (Style::Normal, Style::Zero) => {
                match self.sign.cmp(&other.sign) {
                    Ordering::Less => Ordering::Less,
                    Ordering::Equal | Ordering::Greater => Ordering::Greater
                }
            }

            // |LHS| < |RHS| guaranteed
            (Style::Zero, Style::Normal) => {
                match self.sign.cmp(&other.sign) {
                    Ordering::Less | Ordering::Equal => Ordering::Less,
                    Ordering::Greater => Ordering::Greater
                }
            }

            (Style::Zero, Style::Zero) => Ordering::Equal,
            (Style::Normal, Style::Normal) => {
                match (self.sign, other.sign) {
                    (Sign::Pos, Sign::Pos) => (self.exp, &self.signif).cmp(&(other.exp, &other.signif)),
                    (Sign::Pos, Sign::Neg) => Ordering::Greater,
                    (Sign::Neg, Sign::Neg) => (other.exp, &other.signif).cmp(&(self.exp, &self.signif)),
                    (Sign::Neg, Sign::Pos) => Ordering::Less,
                }
            }
        })
    }
}

// FIXME (#4): these shouldn't need to allocate.
// FIXME (#5): these probably shouldn't truncate the Float if its precision
// is less than the input (e.g. 0b1111_i32 !=
// Float::from(0b1111_i32).with_precision(3))
macro_rules! prim_cmp {
    ($($t: ty),*) => {
        $(
        impl PartialEq<$t> for Float {
            fn eq(&self, other: &$t) -> bool {
                *self == Float::from(*other).with_precision(self.precision())
            }

            fn ne(&self, other: &$t) -> bool {
                *self != Float::from(*other).with_precision(self.precision())
            }
        }

        impl PartialEq<Float> for $t {
            fn eq(&self, other: &Float) -> bool {
                Float::from(*self).with_precision(other.precision()) == *other
            }

            fn ne(&self, other: &Float) -> bool {
                Float::from(*self).with_precision(other.precision()) != *other
            }
        }

        impl PartialOrd<$t> for Float {
            fn partial_cmp(&self, other: &$t) -> Option<Ordering> {
                self.partial_cmp(&Float::from(*other).with_precision(self.precision()))
            }
        }
        impl PartialOrd<Float> for $t {
            fn partial_cmp(&self, other: &Float) -> Option<Ordering> {
                Float::from(*self).with_precision(other.precision()).partial_cmp(other)
            }
        }
            )*
    }
}

prim_cmp!(f32, f64,
          i8, i16, i32, i64, isize,
          u8, u16, u32, u64, usize);
