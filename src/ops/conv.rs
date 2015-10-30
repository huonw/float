use {Style, Sign, Float};
use ramp::Int;

use std::{f64, f32};
use ieee754::Ieee754;

impl From<Int> for Float {
    fn from(x: Int) -> Float {
        if x == 0 {
            Float::zero(1, Sign::Pos)
        } else {
            let bits = x.bit_length();
            let sign = if x.sign() < 0 { Sign::Neg } else { Sign::Pos };
            let signif = x.abs();
            let prec = bits;

            Float {
                prec: prec,
                sign: sign,
                signif: signif,
                exp: bits as i64 - 1,
                style: Style::Normal
            }
        }
    }
}
macro_rules! from_prim_int {
    ($($t: ty),*) => {
        $(
            impl From<$t> for Float {
                fn from(x: $t) -> Float {
                    Float::from(Int::from(x))
                }
            }
            )*
    }
}
from_prim_int!(i8, i16, i32, i64, isize,
               u8, u16, u32, u64, usize);


fn from_ieee754(prec: u32, exp_width: u32, is_negative: bool, exp: i64, signif: u64) -> Float {
    assert!(prec < 64);
    assert!(exp_width < 64);

    let sign = if is_negative { Sign::Neg } else { Sign::Pos };

    let min = -(1 << (exp_width - 1)) + 1;
    let max = 1 << (exp_width - 1);

    if exp == min {
        if signif == 0 {
            Float::zero(prec, sign)
        } else {
            // subnormal
            unimplemented!()
        }
    } else if exp == max {
        if signif == 0 {
            Float::inf(prec, sign)
        } else {
            Float::nan(prec)
        }
    } else {
        let signif = signif | (1 << (prec - 1));
        Float {
            prec: prec,
            sign: sign,
            signif: Int::from(signif),
            exp: exp,
            style: Style::Normal
        }
    }
}

impl From<f64> for Float {
    fn from(x: f64) -> Float {
        let (sign, exp, signif) = f64::decompose(x);
        from_ieee754(53, 11, sign, exp as i64, signif as u64)
    }
}

impl From<f32> for Float {
    fn from(x: f32) -> Float {
        let (sign, exp, signif) = f32::decompose(x);
        from_ieee754(24, 8, sign, exp as i64, signif as u64)
    }
}

impl From<Float> for f64 {
    fn from(f: Float) -> f64 {
        f.debug_assert_valid();
        let f = f.with_precision(53);
        match f.style {
            Style::NaN => f64::NAN,
            Style::Infinity => f64::INFINITY * f.sign as i32 as f64,
            Style::Zero => 0.0 * f.sign as i32 as f64,
            Style::Normal => {
                if f.exp > 1023 {
                    f64::INFINITY * f.sign as i32 as f64
                } else if f.exp < -1023 {
                    0.0 * f.sign as i32 as f64
                } else {
                    let mut raw = (&f.signif).into();
                    raw &= (1 << 52) - 1;
                    f64::recompose(f.sign == Sign::Neg, f.exp as i16, raw)
                }
            }
        }
    }
}
impl From<Float> for f32 {
    fn from(f: Float) -> f32 {
        f.debug_assert_valid();
        let f = f.with_precision(24);
        match f.style {
            Style::NaN => f32::NAN,
            Style::Infinity => f32::INFINITY * f.sign as i32 as f32,
            Style::Zero => 0.0 * f.sign as i32 as f32,
            Style::Normal => {
                if f.exp > 127 {
                    f32::INFINITY * f.sign as i32 as f32
                } else if f.exp < -127 {
                    0.0 * f.sign as i32 as f32
                } else {
                    let mut raw = (&f.signif).into();
                    raw &= (1 << 23) - 1;
                    f32::recompose(f.sign == Sign::Neg, f.exp as i16, raw)
                }
            }
        }
    }
}
