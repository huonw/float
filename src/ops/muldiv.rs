use {Style, Sign, Float};
use std::i64;
use std::ops::{Mul, MulAssign, Div, DivAssign};

use ramp::ll::limb::Limb;

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
                Float::zero_(prec, Sign::Pos)
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
            (Style::Zero, _) => Float::zero_(prec, self.sign),
            // x / 0.0 == inf
            (_, Style::Zero) => Float::inf(prec, other.sign),
            (Style::Infinity, Style::Infinity) => {
                Float::nan(prec)
            }
            // x / inf == 0.0
            (_, Style::Infinity) => Float::zero_(prec, other.sign),
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


                let exp = self.exp - other.exp - (self.signif < other.signif) as i64;
                let sign = self.sign ^ other.sign;

                // we compute (m1 * 2**x) / m2 for some x >= p + 1, to
                // ensure we get the full significand, and the
                // rounding bit, and can use the remainder to check
                // for sticky bits.

                // round-up so that we're shifting by whole limbs,
                // ensuring there's no need for sub-limb shifts.
                let shift = (prec as usize  + 1 + Limb::BITS - 1) / Limb::BITS * Limb::BITS;

                self.signif <<= shift;

                let (mut q, r) = self.signif.divmod(&other.signif);

                let bits = q.bit_length();
                assert!(bits >= prec + 1);
                let unshift = bits - prec;

                let ulp_bit = q.bit(unshift);
                let half_ulp_bit = q.bit(unshift - 1);
                let has_trailing_ones = r != 0 || q.trailing_zeros() < unshift - 1;

                q >>= unshift as usize;

                let mut ret = Float {
                    prec: prec,
                    sign: sign,
                    exp: exp,
                    signif: q,
                    style: Style::Normal,
                };

                if half_ulp_bit && (ulp_bit || has_trailing_ones) {
                    ret.add_ulp(prec);
                }

                ret
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
