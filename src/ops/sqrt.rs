use {Style, Sign, Float};
use ramp::Int;

use std::mem;

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
            Style::Zero => Float::zero(prec, self.sign),
            Style::Normal => {
                if self.sign == Sign::Neg {
                    return Float::nan(prec);
                }
                // use this instead of % 2 to get the right sign
                // (should be 0 or 1, even if exp is negative)
                let c = self.exp & 1;
                self.exp = (self.exp - c) / 2;
                let shift = 1 + c;

                // this code is equivalent to that described in the
                // Handbook of Floating-Point Arithmetic, but
                // carefully uses operations on single bits, for speed
                // (the shifts and additions reduce to such trivial
                // bit ops)
                let mut r = mem::replace(&mut self.signif, Int::from(1) << prec as usize);
                r <<= shift as usize;
                r -= &self.signif;
                let mut two_q_p = &self.signif << 1;

                let mut s_i = prec;
                while s_i > 0 {
                    s_i -= 1;

                    r <<= 1;
                    if r < two_q_p { continue }

                    two_q_p.set_bit(s_i, true);
                    if r < two_q_p {
                        // nothing
                    } else {
                        self.signif.set_bit(s_i, true);
                        r -= &two_q_p;
                        two_q_p.set_bit(s_i + 1, true);
                    }
                    two_q_p.set_bit(s_i, false);
                }
                let round = self.signif.bit(0);
                self.signif >>= 1;
                self.signif += round as i32;

                self
            }
        }
    }
}
