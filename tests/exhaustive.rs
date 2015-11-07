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
            let y_f64 = y.into();
            let real = op_f64(x_f64, y_f64);

            let real = Float::from(real).with_precision(p);
            assert!(est == real,
                    "{}, {} => {:?} != {:?}",
                    x_f64, y_f64, est, real);
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

#[test]
fn add() {
    bin_op(5,
           -10..10 + 1, Region::NegPos, true,
           -10..10 + 1, Region::NegPos, true,
           |x, y| x + y,
           |x, y| x + y)
}

#[test]
fn add_extreme_overflow() {
    bin_op_extreme(5,
                   i64::MAX - 6..i64::MAX, Region::NegPos, true, -i64::MAX,
                   i64::MAX - 6..i64::MAX, Region::NegPos, true, -i64::MAX,
                   i64::MAX,
                   |x, y| x + y)
}

#[test]
fn add_extreme_underflow() {
    bin_op_extreme(5,
                   i64::MIN..i64::MIN + 6, Region::NegPos, true, i64::MAX,
                   i64::MIN..i64::MIN + 6, Region::NegPos, true, i64::MAX,
                   -i64::MAX,
                   |x, y| x + y)
}

#[test]
fn sub() {
    bin_op(5,
           -10..10 + 1, Region::NegPos, true,
           -10..10 + 1, Region::NegPos, true,
           |x, y| x - y,
           |x, y| x - y)
}

#[test]
fn sub_extreme_overflow() {
    bin_op_extreme(5,
                   i64::MAX - 6..i64::MAX, Region::NegPos, true, -i64::MAX,
                   i64::MAX - 6..i64::MAX, Region::NegPos, true, -i64::MAX,
                   i64::MAX,
                   |x, y| x - y)
}

#[test]
fn sub_extreme_underflow() {
    bin_op_extreme(5,
                   i64::MIN..i64::MIN + 6, Region::NegPos, true, i64::MAX,
                   i64::MIN..i64::MIN + 6, Region::NegPos, true, i64::MAX,
                   -i64::MAX,
                   |x, y| x - y)
}

#[test]
fn mul() {
    bin_op(5,
           -10..10 + 1, Region::NegPos, true,
           -10..10 + 1, Region::NegPos, true,
           |x, y| x * y,
           |x, y| x * y)
}

#[test]
fn div() {
    bin_op(5,
           -10..10 + 1, Region::NegPos, true,
           -10..10 + 1, Region::NegPos, false,
           |x, y| x / y,
           |x, y| x / y)
}
