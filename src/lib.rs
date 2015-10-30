#![feature(augmented_assignments,
           op_assign_traits)]

extern crate ramp;
extern crate ieee754;

use ramp::Int;
use std::{fmt, i64, mem};

mod ops;

mod sign;
pub use sign::Sign;

#[derive(Copy, Clone, Debug)]
enum Style {
    NaN,
    Infinity,
    Zero,
    Normal,
    // no subnormals... awkward to handle, and, the exponent goes down
    // to -2**63 + 1, which is somewhat small.
}

#[derive(Clone)]
pub struct Float {
    prec: u32,
    sign: Sign,
    exp: i64,
    signif: Int,
    style: Style
}
impl fmt::Debug for Float {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.debug_assert_valid();
        match self.style {
            Style::NaN => write!(f, "NaN"),
            Style::Zero => write!(f, "{}0.0", self.sign),
            Style::Infinity => write!(f, "{}inf", self.sign),
            Style::Normal => write!(f, "{}{:b} * 2^({} - {})",
                                    self.sign, self.signif, self.exp, self.prec),
        }
    }
}

impl Float {
    pub fn zero(p: u32, sign: Sign) -> Float {
        // this zero is infinitely precise
        Float {
            prec: p,
            sign: sign,
            exp: i64::MIN,
            signif: Int::zero(),
            style: Style::Zero,
        }
    }

    pub fn abs(mut self) -> Float {
        self.debug_assert_valid();
        self.sign = Sign::Pos;
        self
    }
    pub fn sign(&self) -> Option<Sign> {
        self.debug_assert_valid();
        if let Style::NaN = self.style {
            None
        } else {
            Some(self.sign)
        }
    }
    pub fn precision(&self) -> u32 {
        self.debug_assert_valid();
        self.prec
    }
    pub fn with_precision(mut self, prec: u32) -> Float {
        self.debug_assert_valid();
        assert!(prec > 0);
        let old_prec = self.prec;
        self.prec = prec;
        match self.style {
            Style::NaN | Style::Infinity | Style::Zero => {}
            Style::Normal => {
                if prec < old_prec {
                    // less precision:
                    let trailing_zeros = self.signif.trailing_zeros();
                    let has_trailing_one = trailing_zeros < old_prec - prec - 1;
                    let half_ulp_bit = self.signif.bit(old_prec - prec - 1);
                    let ulp_bit = self.signif.bit(old_prec - prec);
                    let round = half_ulp_bit & (ulp_bit | has_trailing_one);

                    self.signif >>= (old_prec - prec) as usize;
                    if round {
                        self.add_ulp(prec);
                    }
                } else if prec > old_prec {
                    self.signif <<= (prec - old_prec) as usize;
                }
            }
        }
        self.debug_assert_valid();
        self
    }

    // self * 2**exp
    pub fn mul_exp2(mut self, exp: i64) -> Float {
        self.debug_assert_valid();
        match self.style {
            Style::Normal => {
                // FIXME (#1): handle overflow
                self.exp = self.exp.checked_add(exp).unwrap_or_else(|| unimplemented!());
            }
            Style::NaN | Style::Infinity | Style::Zero => {}
        }
        self
    }

    fn modify<F: FnOnce(Float) -> Float>(&mut self, f: F) {
        self.debug_assert_valid();
        let p = self.prec;
        let val = mem::replace(self, Float::nan(p));
        *self = f(val);
        self.debug_assert_valid();
    }
    fn nan(p: u32) -> Float {
        Float {
            prec: p,
            sign: Sign::Pos,
            exp: i64::MAX,
            signif: Int::zero(),
            style: Style::NaN,
        }
    }
    fn inf(p: u32, sign: Sign) -> Float {
        Float {
            prec: p,
            sign: sign,
            exp: i64::MAX,
            signif: Int::zero(),
            style: Style::Infinity
        }
    }

    fn add_ulp(&mut self, prec: u32) {
        self.debug_assert_valid();
        self.signif += 1;
        if self.signif.bit(prec) {
            self.signif >>= 1;
            // FIXME (#1): handle overflow
            self.exp += 1;
        }
    }

    fn is_valid(&self) -> bool {
        match self.style {
            Style::NaN => {
                // theoretically we could require self.signif != 0,
                // but that would force an allocation
                self.exp == i64::MAX
            }
            Style::Infinity => {
                self.exp == i64::MAX && self.signif == 0
            }
            Style::Zero => {
                self.exp == i64::MIN && self.signif == 0
            }
            Style::Normal => {
                i64::MIN < self.exp && self.exp < i64::MAX &&
                    self.signif.bit_length() == self.prec
            }
        }
    }
    fn debug_assert_valid(&self) {
        // FIXME: everything is debugging at the moment
        assert!(self.is_valid());
    }
}
