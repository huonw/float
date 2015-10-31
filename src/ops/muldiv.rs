use {Style, Sign, Float};
use ramp::Int;

use std::i64;
use std::ops::{Mul, MulAssign, Div, DivAssign};

impl<'a> Mul<&'a Float> for Float {
    type Output = Float;

    fn mul(mut self, other: &'a Float) -> Float {
        self.debug_assert_valid();
        other.debug_assert_valid();
        assert_eq!(self.prec, other.prec);
        let prec = self.prec;

        match (self.style, other.style) {
            (Style::NaN, _) | (_, Style::NaN) => Float::nan(prec),
            // 0.0 * inf, inf * 0.0 are NaN
            (Style::Infinity, Style::Zero) | (Style::Zero, Style::Infinity) => {
                Float::nan(prec)
            }
            (Style::Infinity, _) | (_, Style::Infinity) => {
                Float::inf(prec, self.sign ^ other.sign)
            }
            (Style::Zero, _) | (_, Style::Zero) => {
                // FIXME (#3): need to get the right sign
                Float::zero(prec, Sign::Pos)
            }
            (Style::Normal, Style::Normal) => {
                if self.exp > 0 && other.exp > i64::MAX - self.exp {
                    // overflow
                    return Float::inf(prec, self.sign ^ other.sign)
                } else if self.exp < 0 && other.exp < i64::MIN - self.exp {
                    // FIXME (#1): handle underflow
                    unimplemented!()
                }

                self.exp += other.exp;
                self.signif *= &other.signif;
                self.sign = self.sign ^ other.sign;

                let bits = self.signif.bit_length();
                let shift = bits - prec;

                let ulp_bit = self.signif.bit(shift);
                let half_ulp_bit = self.signif.bit(shift - 1);
                let has_trailing_ones = self.signif.trailing_zeros() < shift - 1;

                self.signif >>= shift as usize;
                self.exp += shift as i64 - (prec - 1) as i64;

                let round = half_ulp_bit && (ulp_bit || has_trailing_ones);

                if round {
                    self.add_ulp(prec);
                }
                self
            }
        }
    }
}

impl Mul<Float> for Float {
    type Output = Float;

    fn mul(self, other: Float) -> Float {
        // FIXME (#6): this could/should decide to use the Normal one
        // with the largest backing storage as the by-value arg
        // (i.e. reuse the biggest allocation)
        self * &other
    }
}

impl<'a> Mul<Float> for &'a Float {
    type Output = Float;

    fn mul(self, other: Float) -> Float {
        other * self
    }
}

impl<'a> Mul<&'a Float> for &'a Float {
    type Output = Float;

    fn mul(self, other: &'a Float) -> Float {
        // FIXME: this could clone and reserve enough space for the
        // intermediate multiplication, to avoid needing to realloc
        self.clone() * other
    }
}

impl MulAssign<Float> for Float {
    fn mul_assign(&mut self, other: Float) {
        self.modify(|x| x * other)
    }
}

impl<'a> MulAssign<&'a Float> for Float {
    fn mul_assign(&mut self, other: &'a Float) {
        self.modify(|x| x * other)
    }
}

impl<'a> Div<&'a Float> for Float {
    type Output = Float;

    fn div(mut self, other: &'a Float) -> Float {
        self.debug_assert_valid();
        other.debug_assert_valid();
        assert_eq!(self.prec, other.prec);
        let prec = self.prec;

        match (self.style, other.style) {
            (Style::NaN, _) | (_, Style::NaN) => Float::nan(prec),
            // 0.0 / 0.0 is NaN
            (Style::Zero, Style::Zero) => Float::nan(prec),
            // 0.0 / x == 0.0
            (Style::Zero, _) => Float::zero(prec, self.sign),
            // x / 0.0 == inf
            (_, Style::Zero) => Float::inf(prec, other.sign),
            (Style::Infinity, Style::Infinity) => {
                Float::nan(prec)
            }
            // x / inf == 0.0
            (_, Style::Infinity) => Float::zero(prec, other.sign),
            // inf / x == inf (x != 0)
            (Style::Infinity, _) => {
                self.sign = self.sign ^ other.sign;
                self
            }
            (Style::Normal, Style::Normal) => {
                if self.exp > 0 && -other.exp > i64::MAX - self.exp {
                    // overflow
                    return Float::inf(prec, self.sign ^ other.sign)
                } else if self.exp < 0 && -other.exp < i64::MIN - self.exp {
                    // FIXME (#1): handle underflow
                    unimplemented!()
                }

                let sign = self.sign ^ other.sign;

                let mut n = self.signif;
                let m = &other.signif;
                let c = if n < *m { 0 } else { 1 };
                let exp = self.exp + -other.exp - 1 + c;

                if c == 0 {
                    n <<= 1;
                }

                let mut q = Int::from(1);
                let mut r = n - m;

                for _ in 0..prec {
                    let t = (r << 1) - m;
                    if t < 0 {
                        q <<= 1;
                        r = t + m;
                    } else {
                        q <<= 1;
                        q += 1;
                        r = t;
                    }
                }
                let round = q.bit(0);
                let signif = (q >> 1) + (round as i32);

                Float {
                    prec: prec,
                    sign: sign,
                    exp: exp,
                    signif: signif,
                    style: Style::Normal,
                }
            }
        }
    }
}

impl Div<Float> for Float {
    type Output = Float;

    fn div(self, other: Float) -> Float {
        self / &other
    }
}
impl<'a> Div<Float> for &'a Float {
    type Output = Float;

    fn div(self, other: Float) -> Float {
        self.clone() / &other
    }
}
impl<'a> Div<&'a Float> for &'a Float {
    type Output = Float;

    fn div(self, other: &'a Float) -> Float {
        self.clone() / other
    }
}

impl DivAssign<Float> for Float {
    fn div_assign(&mut self, other: Float) {
        self.modify(|x| x / other)
    }
}
impl<'a> DivAssign<&'a Float> for Float {
    fn div_assign(&mut self, other: &'a Float) {
        self.modify(|x| x / other)
    }
}
