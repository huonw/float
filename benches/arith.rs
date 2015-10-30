#![feature(test, augmented_assignments)]

extern crate float;

extern crate rand;
extern crate test;

use float::Float;
use rand::{Rng, XorShiftRng};

fn random(rng: &mut XorShiftRng, p: u32) -> Float {
    let mut f = Float::from(1.0).with_precision(p);
    for _ in 0..(p / 53) + 1 {
        let x = rng.gen::<f64>() - 0.5;
        let x = x + 0.5 * x.signum();
        f *= &Float::from(x).with_precision(p);
    }
    f
}

fn bench<F>(b: &mut test::Bencher, p: u32, mut f: F)
    where F: FnMut(Float, Float) -> Float
{
    let mut rng = rand::random::<XorShiftRng>();
    let x = random(&mut rng, p);
    let y = random(&mut rng, p);
    b.iter(|| {
        f(x.clone(), y.clone())
    })
}

macro_rules! expr ( ($e: expr) => { $e } );

macro_rules! benches {
    (with_prec $p: expr,
     $($name: ident, $op: tt;)*) => {
        $(#[bench]
          fn $name(b: &mut ::test::Bencher) {
              ::bench(b, $p, |x, y| expr!(x $op y))
          })*
    };
    ($($name: ident, $p: expr;)*) => {
        $(
            mod $name {
                #[bench]
                fn noop(b: &mut ::test::Bencher) {
                    ::bench(b, $p, |x, _y| x)
                }

                benches!(with_prec $p,
                         add, +;
                         sub, -;
                         mul, *;
                         div, /;);
            })*
    }
}

benches! {
    p00024, 24;
    p00053, 53;
    p00100, 100;
    p01000, 1000;
    p10000, 10000;
}
