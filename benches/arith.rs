#![feature(test, augmented_assignments)]

extern crate float;

extern crate rand;
extern crate test;

use float::Float;
use rand::{Rng, XorShiftRng};

fn random(rng: &mut XorShiftRng, p: u32) -> Float {
    let mut f = Float::rand(rng, p);
    let p = p as i64;
    let exp = rng.gen_range(-p, p);
    if rng.gen() {
        f = -f;
    }

    f.mul_exp2(exp)
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
