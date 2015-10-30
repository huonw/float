use {Style, Sign, Float};
use ramp::Int;

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


                let mut s = Int::from(1) << prec as usize;
                let mut q = s.clone();
                let mut r = (self.signif << shift as usize) - &q;

                for _ in 0..prec {
                    s >>= 1;
                    let two_r = r << 1;
                    let two_q_p = (&q << 1) + &s;
                    if two_r < two_q_p {
                        r = two_r
                    } else {
                        q += &s;
                        r = two_r - two_q_p;
                    }
                }
                let round = q.bit(0);
                self.signif = (q >> 1) + (round as i32);

                self
            }
        }
    }
}
