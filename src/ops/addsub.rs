use {Style, Float};

use std::{cmp, mem, u32};
use std::ops::{Add, AddAssign, Sub, SubAssign,
               Neg};

impl Neg for Float {
    type Output = Float;
    fn neg(mut self) -> Float {
        self.debug_assert_valid();
        self.sign = -self.sign;
        self
    }
}
impl<'a> Neg for &'a Float {
    type Output = Float;

    fn neg(self) -> Float {
        -self.clone()
    }
}

#[derive(Copy, Clone)]
enum Operation { Add, Sub }
fn can_use_unsigned_add(x: &Float, y: &Float, op: Operation) -> bool {
    x.exp <= y.exp && match op {
        Operation::Add => x.sign == y.sign,
        Operation::Sub => x.sign != y.sign
    }
}
fn can_use_unsigned_sub(x: &Float, y: &Float, op: Operation) -> bool {
    x.exp >= y.exp && match op {
        Operation::Add => x.sign != y.sign,
        Operation::Sub => x.sign == y.sign
    }
}
fn can_use_unsigned_op(x: &Float, y: &Float, op: Operation) -> bool {
    can_use_unsigned_add(x, y, op) || can_use_unsigned_sub(x, y, op)
}

// this does an "unsigned" addition, which means it is just adding the
// two significands directly, even if the signs of the two floats are
// opposite (i.e. so a conventional addition would be a subtraction of
// their significands)
fn unsigned_add(x: &mut Float, y: &Float, op: Operation) {
    debug_assert!(can_use_unsigned_add(x, y, op));

    let diff = y.exp.saturating_sub(x.exp) as u64;
    let diff = cmp::min(diff, u32::MAX as u64) as usize;
    let mut half_ulp_bit = false;
    let mut has_trailing_one = false;
    if diff > 0 {
        let half_ulp = diff as u32 - 1;
        half_ulp_bit = x.signif.bit(half_ulp);
        has_trailing_one = x.signif.trailing_zeros() < half_ulp;
    }

    x.signif >>= diff;
    x.signif += &y.signif;
    x.exp = y.exp;

    let overflowed = x.signif.bit(x.prec);

    let ulp_bit;
    if overflowed {
        has_trailing_one |= half_ulp_bit;
        half_ulp_bit = x.signif.bit(0);
        ulp_bit = x.signif.bit(1);
    } else {
        ulp_bit = x.signif.bit(0);
    }

    if half_ulp_bit && (ulp_bit || has_trailing_one) {
        x.signif += 1 << overflowed as u8;
    }
    x.normalise(true);
}

fn unsigned_sub(x: &mut Float, y: &Float, op: Operation) {
    debug_assert!(can_use_unsigned_sub(x, y, op));

    let diff = x.exp.saturating_sub(y.exp) as u64;
    let diff = cmp::min(diff, u32::MAX as u64) as u32;
    if diff <= x.prec + 2 {
        x.exp = y.exp;
        // FIXME(#14)
        x.signif <<= diff as usize;
        x.signif -= &y.signif;
        if x.signif.sign() < 0 {
            x.signif.negate();
            x.sign = -x.sign;
        }
        let shift = x.signif.bit_length() as i64 - x.prec as i64;
        if shift > 0 {
            x.exp = x.exp.saturating_add(shift);
            let shift = shift as u32;
            let ulp_bit = x.signif.bit(shift);
            let half_ulp_bit = x.signif.bit(shift - 1);
            let has_trailing_ones = x.signif.trailing_zeros() < shift - 1;

            x.signif >>= shift as usize;

            let round = half_ulp_bit && (ulp_bit || has_trailing_ones);
            if round {
                x.signif += 1;
            }
        }
    }
    x.normalise(true);
}

impl Add<Float> for Float {
    type Output = Float;
    fn add(mut self, mut other: Float) -> Float {
        self.debug_assert_valid();
        other.debug_assert_valid();
        assert_eq!(self.prec, other.prec);
        let prec = self.prec;

        match (self.style, other.style) {
            (Style::NaN, _) | (_, Style::NaN) => self = Float::nan(prec),
            (Style::Infinity, Style::Infinity) => {
                self = if self.sign == other.sign { Float::inf(prec, self.sign) } else { Float::nan(prec) }
            }
            (Style::Infinity, _) => self = Float::inf(prec, self.sign),
            (_, Style::Infinity) => self = Float::inf(prec, other.sign),
            (Style::Zero, _) => self = other,
            (_, Style::Zero) => {}
            (Style::Normal, Style::Normal) => {
                if self.exp < other.exp {
                    mem::swap(&mut self, &mut other);
                }

                if self.sign == other.sign {
                    unsigned_add(&mut other, &self, Operation::Add);
                    self = other;
                } else {
                    unsigned_sub(&mut self, &other, Operation::Add);
                }
            }
        }
        self
    }
}
impl<'a> Add<&'a Float> for Float {
    type Output = Float;
    fn add(mut self, other: &'a Float) -> Float {
        self.debug_assert_valid();
        other.debug_assert_valid();
        assert_eq!(self.prec, other.prec);
        let prec = self.prec;

        match (self.style, other.style) {
            (Style::NaN, _) | (_, Style::NaN) => self = Float::nan(prec),
            (Style::Infinity, Style::Infinity) => {
                self = if self.sign == other.sign { Float::inf(prec, self.sign) } else { Float::nan(prec) }
            }
            (Style::Infinity, _) => self = Float::inf(prec, self.sign),
            (_, Style::Infinity) => self = Float::inf(prec, other.sign),
            (Style::Zero, _) => {
                self.clone_from(other);
            }
            (_, Style::Zero) => {},
            (Style::Normal, Style::Normal) => {
                if can_use_unsigned_add(&self, other, Operation::Add) {
                    unsigned_add(&mut self, other, Operation::Add);
                } else if can_use_unsigned_sub(&self, other, Operation::Add) {
                    unsigned_sub(&mut self, other, Operation::Add);
                } else {
                    // FIXME(#15)
                    self += other.clone()
                }
            }
        }
        self
    }
}
impl<'a> Add<Float> for &'a Float {
    type Output = Float;
    fn add(self, other: Float) -> Float {
        other + self
    }
}
impl<'a> Add<&'a Float> for &'a Float {
    type Output = Float;
    fn add(self, other: &'a Float) -> Float {
        let clone_self = can_use_unsigned_op(self, other, Operation::Add);

        if clone_self {
            self.clone() + other
        } else {
            other.clone() + self
        }
    }
}

impl AddAssign<Float> for Float {
    fn add_assign(&mut self, other: Float) {
        self.modify(|x| x + other)
    }
}
impl<'a> AddAssign<&'a Float> for Float {
    fn add_assign(&mut self, other: &'a Float) {
        self.modify(|x| x + other)
    }
}

impl Sub<Float> for Float {
    type Output = Float;

    fn sub(self, mut other: Float) -> Float {
        other = -other;
        self + other
    }
}
impl<'a> Sub<&'a Float> for Float {
    type Output = Float;
    fn sub(self, other: &'a Float) -> Float {
        -(-self + other)
    }
}
impl<'a> Sub<Float> for &'a Float {
    type Output = Float;
    fn sub(self, other: Float) -> Float {
        -other + self
    }
}
impl<'a> Sub<&'a Float> for &'a Float {
    type Output = Float;
    fn sub(self, other: &'a Float) -> Float {
        let clone_self = can_use_unsigned_op(self, other, Operation::Sub);
        if clone_self {
            self.clone() - other
        } else {
            -(other.clone() - self)
        }
    }
}

impl SubAssign<Float> for Float {
    fn sub_assign(&mut self, other: Float) {
        self.modify(|x| x - other)
    }
}
impl<'a> SubAssign<&'a Float> for Float {
    fn sub_assign(&mut self, other: &'a Float) {
        self.modify(|x| x - other)
    }
}
