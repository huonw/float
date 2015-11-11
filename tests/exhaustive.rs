extern crate ramp;
extern crate float;

use float::Float;
use std::i64;

struct AtPrecision {
    val: u32,
    prec: u32,
}

impl AtPrecision {
    fn new(p: u32) -> AtPrecision {
        assert!(p >= 1);
        AtPrecision {
            val: 1 << (p - 1),
            prec: p
        }
    }
}

impl Iterator for AtPrecision {
    type Item = f64;

    fn next(&mut self) -> Option<f64> {
        if self.val >= (1 << self.prec) {
            None
        } else {
            let f = (self.val as f64) / (1 << (self.prec - 1)) as f64;
            self.val += 1;
            Some(f)
        }
    }
}

fn floats<F: FnMut(Float), E: Clone + Iterator<Item = i64>>(p: u32, region: Region, zero: bool, exps: E, mut f: F) {
    if zero {
        f(Float::zero(p))
    }
    for raw_x in AtPrecision::new(p) {
        for exp in exps.clone() {
            let x = Float::from(raw_x).with_precision(p).mul_exp2(exp);
            match region {
                Region::Neg => f(-x),
                Region::NegPos => { f(-x.clone()); f(x) }
                Region::Pos => f(x)
            }
        }
    }
}

#[derive(Copy, Clone, PartialEq)]
enum Region {
    Neg,
    NegPos,
    Pos
}
fn un_op<F, G, E>(p: u32, exps: E,
               region: Region, zero: bool,
               mut op: F, mut op_f64: G)
    where F: FnMut(Float) -> Float, G: FnMut(f64) -> f64, E: Clone + Iterator<Item = i64>
{
    floats(p, region, zero, exps, |x| {
        let est = op(x.clone());
        let x_f64 = x.into();
        let real = op_f64(x_f64);

        let real = Float::from(real).with_precision(p);
        assert!(est == real,
                "{} => {:?} != {:?}",
                x_f64, est, real);
    });
}

fn bin_op<F, G, E1, E2>(p: u32,
                        exps1: E1, region1: Region, zero1: bool,
                        exps2: E2, region2: Region, zero2: bool,
                        mut op: F, mut op_f64: G)
    where F: FnMut(Float, Float) -> Float, G: FnMut(f64, f64) -> f64,
          E1: Clone + Iterator<Item = i64>,
          E2: Clone + Iterator<Item = i64>
{
    floats(p, region1, zero1, exps1, |x| {
        floats(p, region2, zero2, exps2.clone(), |y| {
            let est = op(x.clone(), y.clone());
            let x_f64 = x.clone().into();
            let y_f64 = y.clone().into();
            let real = op_f64(x_f64, y_f64);

            let real = Float::from(real).with_precision(p);
            assert!(est == real,
                    "{:?} ({}), {:?} ({}) => {:?} != {:?}",
                    x, x_f64, y, y_f64, est, real);
        })
    })
}

fn bin_op_extreme<F, E1, E2>(p: u32,
                             exps1: E1, region1: Region, zero1: bool, mul1: i64,
                             exps2: E2, region2: Region, zero2: bool, mul2: i64,
                             unmul: i64,
                             mut op: F)
    where F: FnMut(Float, Float) -> Float,
          E1: Clone + Iterator<Item = i64>,
          E2: Clone + Iterator<Item = i64>
{
    floats(p, region1, zero1, exps1, |x| {
        floats(p, region2, zero2, exps2.clone(), |y| {
            let scaled = op(x.clone().mul_exp2(mul1),
                            y.clone().mul_exp2(mul2));
            let expected = scaled.mul_exp2(unmul);
            let est = op(x.clone(), y.clone());
            assert!(est == expected,
                    "{:?}, {:?} => {:?} != {:?}",
                    x, y, est, expected)
        })
    })
}

#[test]
fn sqrt() {
    un_op(10,
          -10..10 + 1, Region::Pos, true,
          |x| x.sqrt(),
          |y| y.sqrt())
}

macro_rules! expr { ($e: expr) => { $e } }
macro_rules! by_val_by_ref {
    ($op: tt, $tester: ident) => {
        #[test] fn val_val() { expr!($tester(|x, y| x $op y)) }
        #[test] fn val_ref() { expr!($tester(|x, y| x $op &y)) }
        #[test] fn ref_val() { expr!($tester(|x, y| &x $op y)) }
        #[test] fn ref_ref() { expr!($tester(|x, y| &x $op &y)) }
    }
}
macro_rules! tests {
    ($($name: ident, $op: tt, $y_zero: expr,
       $extreme_x_exp: expr, $x_shift: expr,
       $extreme_y_exp: expr, $y_shift: expr,
       $unshift: expr;)*) => {
        $(mod $name {
            use {bin_op, Region};
            use float::Float;

            fn test<F: FnMut(Float, Float) -> Float>(f: F) {
                expr!(bin_op(5,
                             -5..5 + 1, Region::NegPos, true,
                             -5..5 + 1, Region::NegPos, $y_zero,
                             f,
                             |x, y| x $op y))
            }

            by_val_by_ref! { $op, test }

            #[allow(unused_imports)]
            mod overflow {
                use {bin_op_extreme, Region};
                use float::Float;
                use HALF_EXP;
                use std::i64;

                fn test<F: FnMut(Float, Float) -> Float>(f: F) {
                    expr!(bin_op_extreme(5,
                                         $extreme_x_exp, Region::NegPos, true, -$x_shift,
                                         $extreme_y_exp, Region::NegPos, $y_zero, -$y_shift,
                                         $unshift,
                                         f))
                }

                by_val_by_ref! { $op, test }
            }
            #[allow(unused_imports)]
            mod underflow {
                use {bin_op_extreme, Region};
                use float::Float;
                use HALF_EXP;
                use std::i64;
                use std::ops::Neg;

                fn test<F: FnMut(Float, Float) -> Float>(f: F) {
                    expr!(bin_op_extreme(5,
                                         $extreme_x_exp.map(Neg::neg as fn(i64) -> i64), Region::NegPos, true, $x_shift,
                                         $extreme_y_exp.map(Neg::neg as fn(i64) -> i64), Region::NegPos, $y_zero, $y_shift,
                                         -$unshift,
                                         f))
                }

                by_val_by_ref! { $op, test }
            }
        })*
    }
}

const HALF_EXP: i64 = i64::MAX / 2;
tests! {
    add, +, true, i64::MAX - 6..i64::MAX, i64::MAX, i64::MAX - 6..i64::MAX, i64::MAX, i64::MAX;
    sub, -, true, i64::MAX - 6..i64::MAX, i64::MAX, i64::MAX - 6..i64::MAX, i64::MAX, i64::MAX;
    mul, *, true,
    (i64::MAX - 4..i64::MAX).chain(HALF_EXP - 4..HALF_EXP + 5).chain(1..4), HALF_EXP,
    (i64::MAX - 4..i64::MAX).chain(HALF_EXP - 4..HALF_EXP + 5).chain(1..4), HALF_EXP,
    HALF_EXP * 2;
    div, /, false,
    (i64::MAX - 4..i64::MAX).chain(HALF_EXP - 4..HALF_EXP + 5).chain(0..4), HALF_EXP,
    (i64::MIN + 1.. i64::MIN + 5).chain(-HALF_EXP - 5..-HALF_EXP + 5).chain(-4..0), -HALF_EXP,
    HALF_EXP * 2;
}
