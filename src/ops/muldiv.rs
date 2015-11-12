use {Style, Sign, Float, add_overflow, sub_overflow};
use std::i64;
use std::ops::{Mul, MulAssign, Div, DivAssign};

use ramp::ll::limb::Limb;

impl<'a> MulAssign<&'a Float> for Float {
    fn mul_assign(&mut self, other: &'a Float) {
        self.debug_assert_valid();
        other.debug_assert_valid();
        assert_eq!(self.prec, other.prec);
        let prec = self.prec;

        match (self.style, other.style) {
            (Style::NaN, _) | (_, Style::NaN) => *self = Float::nan(prec),
            // 0.0 * inf, inf * 0.0 are NaN
            (Style::Infinity, Style::Zero) | (Style::Zero, Style::Infinity) => {
                *self = Float::nan(prec)
            }
            (Style::Infinity, _) | (_, Style::Infinity) => {
                *self = Float::inf(prec, self.sign ^ other.sign)
            }
            (Style::Zero, _) | (_, Style::Zero) => {
                // FIXME (#3): need to get the right sign
                *self = Float::zero_(prec, Sign::Pos)
            }
            (Style::Normal, Style::Normal) => {
                self.signif *= &other.signif;
                self.sign = self.sign ^ other.sign;

                let bits = self.signif.bit_length();
                let shift = bits - prec;

                let (raw_mult_exp, o1) = add_overflow(self.exp, other.exp);
                let (adjusted_exp, o2) = add_overflow(raw_mult_exp,
                                                      shift as i64 - (prec - 1) as i64);
                self.exp = adjusted_exp;
                if o1 ^ o2 {
                    // if we only overflowed once, then there's a
                    // problem. A double overflow means we went over
                    // the limit and then back, but a single means we
                    // never returned.
                    let overflowed = if o1 { raw_mult_exp } else { adjusted_exp };
                    self.exp = if overflowed < 0 { i64::MAX } else { i64::MIN };
                    self.normalise(false);
                } else {
                    let ulp_bit = self.signif.bit(shift);
                    let half_ulp_bit = self.signif.bit(shift - 1);
                    let has_trailing_ones = self.signif.trailing_zeros() < shift - 1;

                    self.signif >>= shift as usize;

                    let round = half_ulp_bit && (ulp_bit || has_trailing_ones);
                    if round {
                        self.signif += 1;
                    }
                    self.normalise(true);
                }
            }
        }
    }
}

impl MulAssign<Float> for Float {
    fn mul_assign(&mut self, other: Float) {
        *self *= &other
    }
}

impl Mul<Float> for Float {
    type Output = Float;

    fn mul(mut self, other: Float) -> Float {
        // FIXME (#6): this could/should decide to use the Normal one
        // with the largest backing storage as the by-value arg
        // (i.e. reuse the biggest allocation)
        self *= other;
        self
    }
}

impl<'a> Mul<&'a Float> for Float {
    type Output = Float;

    fn mul(mut self, other: &'a Float) -> Float {
        self *= other;
        self
    }
}

impl<'a> Mul<Float> for &'a Float {
    type Output = Float;

    fn mul(self, mut other: Float) -> Float {
        other *= self;
        other
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

impl<'a> DivAssign<&'a Float> for Float {
    fn div_assign(&mut self, other: &'a Float) {
        self.debug_assert_valid();
        other.debug_assert_valid();
        assert_eq!(self.prec, other.prec);
        let prec = self.prec;

        match (self.style, other.style) {
            (Style::NaN, _) | (_, Style::NaN) => *self = Float::nan(prec),
            // 0.0 / 0.0 is NaN
            (Style::Zero, Style::Zero) => *self = Float::nan(prec),
            // 0.0 / x == 0.0
            (Style::Zero, _) => *self = Float::zero_(prec, self.sign),
            // x / 0.0 == inf
            (_, Style::Zero) => *self = Float::inf(prec, other.sign),
            (Style::Infinity, Style::Infinity) => {
                *self = Float::nan(prec)
            }
            // x / inf == 0.0
            (_, Style::Infinity) => *self = Float::zero_(prec, other.sign),
            // inf / x == inf (x != 0)
            (Style::Infinity, _) => {
                self.sign = self.sign ^ other.sign;
            }
            (Style::Normal, Style::Normal) => {
                let (raw_mult_exp, o1) = sub_overflow(self.exp, other.exp);
                let (adjusted_exp, o2) = sub_overflow(raw_mult_exp,
                                                      (self.signif < other.signif) as i64);
                self.sign = self.sign ^ other.sign;
                self.exp = adjusted_exp;

                if o1 ^ o2 {
                    // if we only overflowed once, then there's a
                    // problem. A double overflow means we went over
                    // the limit and then back, but a single means we
                    // never returned.
                    let overflowed = if o1 { raw_mult_exp } else { adjusted_exp };
                    self.exp = if overflowed < 0 { i64::MAX } else { i64::MIN };
                    self.normalise(false);
                } else {
                    // we compute (m1 * 2**x) / m2 for some x >= p + 1, to
                    // ensure we get the full significand, and the
                    // rounding bit, and can use the remainder to check
                    // for sticky bits.

                    // round-up so that we're shifting by whole limbs,
                    // ensuring there's no need for sub-limb shifts.
                    let shift = (prec as usize  + 1 + Limb::BITS - 1) / Limb::BITS * Limb::BITS;

                    self.signif <<= shift;

                    let (q, r) = self.signif.divmod(&other.signif);
                    self.signif.clone_from(&q);

                    let bits = self.signif.bit_length();
                    assert!(bits >= prec + 1);
                    let unshift = bits - prec;

                    let ulp_bit = self.signif.bit(unshift);
                    let half_ulp_bit = self.signif.bit(unshift - 1);
                    let has_trailing_ones = r != 0 || self.signif.trailing_zeros() < unshift - 1;

                    self.signif >>= unshift as usize;

                    if half_ulp_bit && (ulp_bit || has_trailing_ones) {
                        self.signif += 1;
                    }
                    self.normalise(true);
                }
            }
        }
    }
}


impl DivAssign<Float> for Float {
    fn div_assign(&mut self, other: Float) {
        *self /= &other;
    }
}
impl Div<Float> for Float {
    type Output = Float;

    fn div(mut self, other: Float) -> Float {
        self /= &other;
        self
    }
}
impl<'a> Div<&'a Float> for Float {
    type Output = Float;

    fn div(mut self, other: &'a Float) -> Float {
        self /= other;
        self
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
