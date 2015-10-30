#![feature(float_extras)]

extern crate ramp;
extern crate float;

use float::Float;

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

fn floats<F: FnMut(f64), E: Clone + Iterator<Item = i64>>(p: u32, region: Region, zero: bool, exps: E, mut f: F) {
    if zero {
        f(0.0)
    }
    let pos = region != Region::Neg;
    let neg = region != Region::Pos;

    for raw_x in AtPrecision::new(p) {
        for exp in exps.clone() {
            let x = f64::ldexp(raw_x, exp as isize);
            if pos { f(x) }
            if neg { f(-x) }
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
        let est = op(Float::from(x).with_precision(p));
        let real = op_f64(x);

        let real = Float::from(real).with_precision(p);
        assert!(est.clone() == real.clone(),
                "{} => {:?} != {:?}",
                x, est, real);
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
            let est = op(Float::from(x).with_precision(p),
                         Float::from(y).with_precision(p));
            let real = op_f64(x, y);

            let real = Float::from(real).with_precision(p);
            assert!(est.clone() == real.clone(),
                    "{}, {} => {:?} != {:?}",
                    x, y, est, real);
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
fn sub() {
    bin_op(5,
           -10..10 + 1, Region::NegPos, true,
           -10..10 + 1, Region::NegPos, true,
           |x, y| x - y,
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
