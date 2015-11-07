use {Style, Sign, Float};

use std::mem;
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

impl Add<Float> for Float {
    type Output = Float;
    fn add(mut self, mut other: Float) -> Float {
        self.debug_assert_valid();
        other.debug_assert_valid();
        assert_eq!(self.prec, other.prec);
        let prec = self.prec;

        match (self.style, other.style) {
            (Style::NaN, _) | (_, Style::NaN) => Float::nan(prec),
            (Style::Infinity, Style::Infinity) => {
                if self.sign == other.sign { Float::inf(prec, self.sign) } else { Float::nan(prec) }
            }
            (Style::Infinity, _) => Float::inf(prec, self.sign),
            (_, Style::Infinity) => Float::inf(prec, other.sign),
            (Style::Zero, _) => other,
            (_, Style::Zero) => self,
            (Style::Normal, Style::Normal) => {
                if self.exp < other.exp {
                    mem::swap(&mut self, &mut other);
                }

                let s1 = self.sign;
                let s2 = other.sign;
                let s = s1 ^ s2;

                self.signif <<= 3;
                other.signif <<= 3;

                // FIXME (#2): integer types are wrong here
                let shift = self.exp.saturating_sub(other.exp) as usize;
                let middle_case = (other.signif.trailing_zeros() as usize) < shift;
                other.signif >>= shift;
                if shift <= 3 {
                    // nothing
                } else if shift >= prec as usize + 3 {
                    other.signif |= 1;
                } else {
                    other.signif |= middle_case as i32;
                };
                if s == Sign::Pos {
                    self.signif += other.signif;
                } else {
                    self.signif -= other.signif;
                }

                if self.signif == 0 {
                    // FIXME (#3): should this always return +0.0?
                    return Float::zero(prec);
                }
                self.sign = if self.signif < 0 {
                    self.signif = self.signif.abs();
                    -s1
                } else {
                    s1
                };

                let bits = self.signif.bit_length();
                let round = if bits > prec + 3 {
                    // carried
                    self.exp += 1;

                    let ulp_bit = self.signif.bit(4);
                    let half_ulp_bit = self.signif.bit(3);
                    let has_trailing_one = self.signif.bit(2) | self.signif.bit(1) | self.signif.bit(0);
                    self.signif >>= 4;
                    half_ulp_bit && (ulp_bit || has_trailing_one)
                } else if bits == prec + 3 {
                    let ulp_bit = self.signif.bit(3);
                    let half_ulp_bit = self.signif.bit(2);
                    let has_trailing_one = self.signif.bit(1) | self.signif.bit(0);
                    self.signif >>= 3;
                    half_ulp_bit && (ulp_bit || has_trailing_one)
                } else {
                    let refill = prec + 3 - bits;
                    self.exp = self.exp.checked_sub(refill as i64).unwrap_or_else(|| unimplemented!());
                    if refill <= 3 {
                        let b0 = self.signif.bit(0);
                        let b1 = self.signif.bit(1);
                        let b2 = self.signif.bit(2);

                        self.signif >>= (3 - refill) as usize;

                        if refill == 1 { b1 && (b2 || b0) } else if refill == 2 { b0 && b1 } else { false }
                    } else {
                        self.signif <<= (refill - 3) as usize;
                        false
                    }
                };
                if round {
                    self.add_ulp();
                }

                self
            }
        }
    }
}
impl<'a> Add<&'a Float> for Float {
    type Output = Float;
    fn add(self, other: &'a Float) -> Float {
        self + other.clone()
    }
}
impl<'a> Add<Float> for &'a Float {
    type Output = Float;
    fn add(self, other: Float) -> Float {
        self.clone() + other
    }
}
impl<'a> Add<&'a Float> for &'a Float {
    type Output = Float;
    fn add(self, other: &'a Float) -> Float {
        self.clone() + other.clone()
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
        self - other.clone()
    }
}
impl<'a> Sub<Float> for &'a Float {
    type Output = Float;
    fn sub(self, other: Float) -> Float {
        self.clone() - other
    }
}
impl<'a> Sub<&'a Float> for &'a Float {
    type Output = Float;
    fn sub(self, other: &'a Float) -> Float {
        self.clone() - other.clone()
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
