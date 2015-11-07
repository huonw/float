use {Style, Sign, Float};

impl Float {
    pub fn sqrt(mut self) -> Float {
        self.debug_assert_valid();
        let prec = self.prec;

        match self.style {
            Style::NaN => Float::nan(prec),
            Style::Infinity => {
                match self.sign {
                    Sign::Pos => Float::inf(prec, Sign::Pos),
                    Sign::Neg => Float::nan(prec),
                }
            }
            Style::Zero => Float::zero_(prec, self.sign),
            Style::Normal => {
                if self.sign == Sign::Neg {
                    return Float::nan(prec);
                }

                // use this instead of % 2 to get the right sign
                // (should be 0 or 1, even if exp is negative)
                let c = self.exp & 1;
                let exp = (self.exp - c) / 2;

                // we compute sqrt(m1 * 2**c * 2**(p + 1)) to ensure
                // we get the full significand, and the rounding bit,
                // and can use the remainder to check for sticky bits.
                let shift = prec as usize + 1 + c as usize;
                self.signif <<= shift;

                let (mut sqrt, rem) = self.signif.sqrt_rem().unwrap();

                let bits = sqrt.bit_length();
                assert!(bits >= prec + 1);
                let unshift = bits - prec;

                let ulp_bit = sqrt.bit(unshift);
                let half_ulp_bit = sqrt.bit(unshift - 1);
                let has_trailing_ones = rem != 0 || sqrt.trailing_zeros() < unshift - 1;

                sqrt >>= unshift as usize;

                let mut ret = Float {
                    prec: prec,
                    sign: Sign::Pos,
                    exp: exp,
                    signif: sqrt,
                    style: Style::Normal,
                };

                if half_ulp_bit && (ulp_bit || has_trailing_ones) {
                    ret.add_ulp();
                }

                ret
            }
        }
    }
}
