#![feature(test, augmented_assignments)]

extern crate float;

extern crate rand;
extern crate test;

use float::Float;
use rand::{Rng, XorShiftRng};

fn random(rng: &mut XorShiftRng, p: u32) -> Float {
    let mut f = Float::rand(rng, p);
    let exp = rng.gen::<i32>();
    if rng.gen() {
        f = -f;
    }

    f.mul_exp2(exp as i64)
}

fn bench<F>(b: &mut test::Bencher, p: u32, mut f: F)
    where F: FnMut(Float) -> Float
{
    let mut rng = rand::random::<XorShiftRng>();
    b.iter(|| {
        // sqrt is somewhat dependent on the exact float it's working
        // on, so we need to test a lot of them so the differences
        // average out and hence the generation needs to be done
        // regularly
        f(random(&mut rng, p))
    })
}

macro_rules! benches {
    (with_prec $p: expr, $n: ident =>
     $($name: ident, $e: expr;)*) => {
        $(#[bench]
          fn $name(b: &mut ::test::Bencher) {
              ::bench(b, $p, |$n| $e);
          })*
    };

    ($($name: ident, $p: expr;)*) => {
        $(
            mod $name {
                #[bench]
                fn noop(b: &mut ::test::Bencher) {
                    ::bench(b, $p, |x| x)
                }

                benches!(with_prec $p, x =>
                         sqrt, x.sqrt(););
            })*
    }
}

benches! {
    p00024, 24;
    p00053, 53;
    p00100, 100;
    p01000, 1000;
//    p10000, 10000; // too slow/unreliable?
}
