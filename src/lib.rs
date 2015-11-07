#![feature(augmented_assignments,
           op_assign_traits,
           core_intrinsics)]

extern crate ramp;
extern crate ieee754;
extern crate rand;

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
    pub fn zero(p: u32) -> Float {
        Float::zero_(p, Sign::Pos)
    }

    pub fn neg_zero(p: u32) -> Float {
        Float::zero_(p, Sign::Neg)
    }

    pub fn max(p: u32) -> Float {
        Float {
            prec: p,
            sign: Sign::Pos,
            exp: i64::MAX - 1,
            signif: (Int::from(1) << p as usize) - 1,
            style: Style::Normal
        }
    }
    pub fn min(p: u32) -> Float {
        -Float::max(p)
    }
    pub fn min_positive(p: u32) -> Float {
        Float {
            prec: p,
            sign: Sign::Pos,
            exp: i64::MIN + 1,
            signif: (Int::from(1) << (p as usize - 1)),
            style: Style::Normal,
        }
    }

    pub fn infinity(p: u32) -> Float {
        Float::inf(p, Sign::Pos)
    }

    pub fn neg_infinity(p: u32) -> Float {
        Float::inf(p, Sign::Neg)
    }

    pub fn nan(p: u32) -> Float {
        Float {
            prec: p,
            sign: Sign::Pos,
            exp: i64::MAX,
            signif: Int::zero(),
            style: Style::NaN,
        }
    }

    pub fn next_above(mut self) -> Float {
        if self.sign == Sign::Pos {
            self.add_ulp();
        } else {
            self.sub_ulp();
        }
        self
    }
    pub fn next_below(mut self) -> Float {
        if self.sign == Sign::Neg {
            self.add_ulp();
        } else {
            self.sub_ulp();
        }
        self
    }
    pub fn next_toward(self, target: &Float) -> Float {
        use std::cmp::Ordering;
        match self.partial_cmp(target) {
            None | Some(Ordering::Equal) => self,
            Some(Ordering::Less) => self.next_above(),
            Some(Ordering::Greater) => self.next_below(),
        }
    }

    /// Generate a float in [0, 1), with the same distribution as
    /// generating an integer in [0, 2**p) and dividing by
    /// 2**p. (i.e. pretty close to uniform, but the smallest bits
    /// aren't quite right).
    pub fn rand<R: rand::Rng>(r: &mut R, p: u32) -> Float {
        use ramp::RandomInt;

        let mut signif = RandomInt::gen_uint(r, p as usize);
        let bits = signif.bit_length();
        assert!(bits <= p);
        let shift = p - bits;

        let (exp, style) = if signif == 0 {
            (i64::MIN, Style::Zero)
        } else {
            signif <<= shift as usize;

            (-1 - (shift as i64), Style::Normal)
        };
        Float {
            prec: p,
            sign: Sign::Pos,
            exp: exp,
            signif: signif,
            style: style
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
                        self.add_ulp();
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
                self.exp = self.exp.saturating_add(exp);
                self.normalise(false);
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
    fn zero_(p: u32, sign: Sign) -> Float {
        Float {
            prec: p,
            sign: sign,
            exp: i64::MIN,
            signif: Int::zero(),
            style: Style::Zero,
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
    fn normalise(&mut self, check_bits: bool) {
        let diff = if check_bits {
            self.signif.bit_length() as i64 - self.prec as i64
        } else {
            0
        };
        self.exp = self.exp.saturating_add(diff);
        match self.exp {
            // FIXME (#13)
            i64::MAX => *self = Float::inf(self.prec, self.sign),
            i64::MIN => *self = Float::zero_(self.prec, self.sign),
            _ => {
                use std::cmp::Ordering;
                match diff.cmp(&0) {
                    Ordering::Greater => self.signif >>= diff as usize,
                    Ordering::Equal => {},
                    Ordering::Less => self.signif <<= (-diff) as usize,
                }
            }
        }
    }

    fn add_ulp(&mut self) {
        self.debug_assert_valid();
        match self.style {
            Style::NaN => {},
            Style::Infinity => {}
            Style::Zero => {
                let s = self.sign;
                // FIXME (#13)
                *self = Float::min_positive(self.prec);
                self.sign = s;
            }
            Style::Normal => {
                self.signif += 1;
                if self.signif.bit(self.prec) {
                    self.exp += 1;
                    if self.exp == i64::MAX {
                        // FIXME (#13)
                        *self = Float::inf(self.prec, self.sign)
                    } else {
                        self.signif >>= 1;
                    }
                }
            }
        }
    }
    fn sub_ulp(&mut self) {
        self.debug_assert_valid();
        match self.style {
            Style::NaN => {},
            Style::Infinity => {
                // FIXME (#13)
                *self = Float::max(self.prec)
            }
            Style::Zero => {
                // FIXME (#13)
                *self = Float::min_positive(self.prec);
                self.sign = -self.sign;
            }
            Style::Normal => {
                self.signif -= 1;
                if !self.signif.bit(self.prec - 1) {
                    self.exp -= 1;
                    if self.exp == i64::MIN {
                        // FIXME (#13)
                        *self = Float::zero_(self.prec, self.sign);
                    } else {
                        self.signif <<= 1;
                        self.signif |= 1;
                    }
                }
            }
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
