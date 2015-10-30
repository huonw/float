#![feature(plugin)]
#![plugin(quickcheck_macros)]

// quickcheck in "high" precision, against either a known good answer,
// or the same computation performed in double the tested precision.

extern crate quickcheck;
extern crate float;

use float::Float;
use quickcheck::TestResult;

fn ensure_ulp(a: Float, b: Float, count: f64) -> TestResult {
    let computed_p = a.precision();
    let p = b.precision();
    let a = a.with_precision(p + 5);
    let b = b.with_precision(p + 5);

    let diff = a.clone() - b.clone();
    let rel = diff.clone() / b.clone();
    let ulp = Float::from(count).with_precision(p + 5).mul_exp2(computed_p as i64 - 1);

    assert!(rel <= ulp,
            "{:?} > {:?}, ({:?} vs. {:?}, diff {:?})",
            rel, ulp, a, b, ulp);
    TestResult::from_bool(true)
}

fn mul_to_float(x: &[f64], prec: u32) -> Float {
    let one = Float::from(1.0).with_precision(prec);
    x.iter().fold(one.clone(), |a, &b| {
        a * Float::from(b).with_precision(prec)
    })
}

fn un_ulp<D, F>(x: Vec<f64>, prec: u32, ulp: f64,
                discard: D,
                mut op: F,
                answer: Option<&Fn(&Float) -> Float>) -> TestResult
    where D: Fn(&Float) -> bool, F: FnMut(Float) -> Float
{

    let x = mul_to_float(&x, prec);
    if discard(&x) {
        return TestResult::discard();
    }

    let exact = match answer {
        None => op(x.clone().with_precision(prec * 2)),
        Some(f) => f(&x)
    };
    let computed = op(x);

    ensure_ulp(computed, exact, ulp)
}

fn bin_ulp<D, F>(x: Vec<f64>, y: Vec<f64>, prec: u32, ulp: f64,
                 discard: D,
                 mut op: F,
                 answer: Option<&Fn(&Float, &Float) -> Float>) -> TestResult
    where D: Fn(&Float, &Float) -> bool, F: FnMut(Float, Float) -> Float
{
    let x = mul_to_float(&x, prec);
    let y = mul_to_float(&y, prec);
    if discard(&x, &y) {
        return TestResult::discard();
    }

    let exact = match answer {
        None => op(x.clone().with_precision(prec * 2),
                   y.clone().with_precision(prec * 2)),
        Some(f) => f(&x, &y)
    };
    let computed = op(x, y);

    ensure_ulp(computed, exact, ulp)
}

const PREC_OFFSET: u32 = 100;

#[quickcheck]
fn add(x: Vec<f64>, y: Vec<f64>, prec: u16) -> TestResult {
    let prec = PREC_OFFSET + prec as u32;

    bin_ulp(x, y, prec, 0.5,
            |_, _| false,
            |x, y| x + y,
            None)
}

#[quickcheck]
fn sub(x: Vec<f64>, y: Vec<f64>, prec: u16) -> TestResult {
    let prec = PREC_OFFSET + prec as u32;

    bin_ulp(x, y, prec, 0.5,
            |_, _| false,
            |x, y| x - y,
            None)
}

#[quickcheck]
fn mul(x: Vec<f64>, y: Vec<f64>, prec: u16) -> TestResult {
    let prec = PREC_OFFSET + prec as u32;

    bin_ulp(x, y, prec, 0.5,
            |_, _| false,
            |x, y| x * y,
            None)
}

#[quickcheck]
fn div(x: Vec<f64>, y: Vec<f64>, prec: u16) -> TestResult {
    let prec = PREC_OFFSET + prec as u32;

    bin_ulp(x, y, prec, 0.5,
            |_, _| false,
            |x, y| x / y,
            None)
}

#[quickcheck]
fn sqrt(x: Vec<f64>, prec: u16) -> TestResult {
    let prec = PREC_OFFSET + prec as u32;

    un_ulp(x, prec, 0.5,
           |x| x.sign() != Some(::float::Sign::Pos),
           |x| x.sqrt(),
           None)
}

#[quickcheck]
fn sqrt_square(x: Vec<f64>, prec: u16) -> TestResult {
    let prec = PREC_OFFSET + prec as u32;

    un_ulp(x, prec, 1.0,
           |x| x.sign() != Some(::float::Sign::Pos),
           |x| { let y = x.sqrt(); y.clone() * y },
           Some(&|x| x.clone()))
}


#[quickcheck]
fn square_sqrt(x: Vec<f64>, prec: u16) -> TestResult {
    let prec = PREC_OFFSET + prec as u32;

    // squaring spreads things out, so the sqrt is exact
    un_ulp(x, prec, 0.0,
           |_| false,
           |x| { let y = x.clone() * x; y.sqrt() },
           Some(&|x| x.clone().abs()))
}

#[quickcheck]
fn div_mul(x: Vec<f64>, y: Vec<f64>, prec: u16) -> TestResult {
    let prec = PREC_OFFSET + prec as u32;
    bin_ulp(x, y, prec, 1.0,
            |_, y| *y == Float::from(0.0).with_precision(prec),
            |x, y| (x / y.clone()) * y,
            Some(&|x, _| x.clone()))
}
#[quickcheck]
fn mul_div(x: Vec<f64>, y: Vec<f64>, prec: u16) -> TestResult {
    let prec = PREC_OFFSET + prec as u32;
    bin_ulp(x, y, prec, 1.0,
            |_, y| *y == Float::from(0.0).with_precision(prec),
            |x, y| (x * y.clone()) / y,
            Some(&|x, _| x.clone()))
}
