#![feature(plugin)]
#![plugin(quickcheck_macros)]

extern crate quickcheck;
extern crate float;

use float::Float;
use quickcheck::TestResult;

use std::fmt::Display;

fn assert_eq<T: Copy + PartialEq + Display + From<Float>>(a: Float, b: T) -> TestResult
    where Float: From<T>
{
    let f = a.clone().into();
    assert!(b == f,
            "{} != {}, ({:?} vs. {:?})", f, b, a, Float::from(b));
    TestResult::from_bool(true)
}

macro_rules! tests {
    ($t: ident) => {
        mod $t {
            use assert_eq;
            use float::Float;
            use quickcheck::TestResult;

            #[quickcheck]
            fn to_from(x: $t) {
                let f = Float::from(x);
                assert_eq(f, x);
            }

            #[quickcheck]
            fn to_f32_prec(x: $t) {
                let f = Float::from(x);
                let g = f.with_precision(24);
                assert_eq(g, x as f32);
            }

            #[quickcheck]
            fn to_f64_prec(x: $t) {
                let f = Float::from(x);
                let g = f.with_precision(53);
                assert_eq(g, x as f64);
            }

            #[quickcheck]
            fn add(x: $t, y: $t) {
                let (f, g) = (Float::from(x), Float::from(y));
                assert_eq(f + g, x + y);
            }

            #[quickcheck]
            fn sub(x: $t, y: $t) {
                let (f, g) = (Float::from(x), Float::from(y));
                assert_eq(f - g, x - y);
            }

            #[quickcheck]
            fn mul(x: $t, y: $t) {
                let (f, g) = (Float::from(x), Float::from(y));
                assert_eq(f + g, x + y);
            }

            #[quickcheck]
            fn div(x: $t, y: $t) -> TestResult {
                if y == 0.0 { return TestResult::discard() }
                let (f, g) = (Float::from(x), Float::from(y));
                assert_eq(f / g, x / y)
            }
            #[quickcheck]
            fn sqrt(x: $t) -> TestResult {
                if x < 0.0 { return TestResult::discard() }

                let f = Float::from(x);
                assert_eq(f.sqrt(), x.sqrt())
            }

            #[quickcheck]
            fn eq(x: $t, y: $t) {
                let f = Float::from(x);
                let g = Float::from(y);

                let arr = [(&f, x), (&g, y)];
                for &(b1, p1) in &arr {
                    for &(b2, p2) in &arr {
                        assert_eq!(*b1 == *b1, p1 == p1);
                        assert_eq!(*b1 == *b2, p1 == p2);
                        assert_eq!(*b2 == *b1, p2 == p1);
                        assert_eq!(*b2 == *b2, p2 == p2);

                        assert_eq!(*b1 == p1, p1 == p1);
                        assert_eq!(*b1 == p2, p1 == p2);
                        assert_eq!(*b2 == p1, p2 == p1);
                        assert_eq!(*b2 == p2, p2 == p2);

                        assert_eq!(p1 == *b1, p1 == p1);
                        assert_eq!(p1 == *b2, p1 == p2);
                        assert_eq!(p2 == *b1, p2 == p1);
                        assert_eq!(p2 == *b2, p2 == p2);
                    }
                }
            }
            #[quickcheck]
            fn ne(x: $t, y: $t) {
                let f = Float::from(x);
                let g = Float::from(y);

                let arr = [(&f, x), (&g, y)];
                for &(b1, p1) in &arr {
                    for &(b2, p2) in &arr {
                        assert_eq!(*b1 != *b1, p1 != p1);
                        assert_eq!(*b1 != *b2, p1 != p2);
                        assert_eq!(*b2 != *b1, p2 != p1);
                        assert_eq!(*b2 != *b2, p2 != p2);

                        assert_eq!(*b1 != p1, p1 != p1);
                        assert_eq!(*b1 != p2, p1 != p2);
                        assert_eq!(*b2 != p1, p2 != p1);
                        assert_eq!(*b2 != p2, p2 != p2);

                        assert_eq!(p1 != *b1, p1 != p1);
                        assert_eq!(p1 != *b2, p1 != p2);
                        assert_eq!(p2 != *b1, p2 != p1);
                        assert_eq!(p2 != *b2, p2 != p2);
                    }
                }
            }
            #[quickcheck]
            fn partial_cmp(x: $t, y: $t) {
                let f = Float::from(x);
                let g = Float::from(y);

                let arr = [(&f, x), (&g, y)];
                for &(b1, p1) in &arr {
                    for &(b2, p2) in &arr {
                        assert_eq!(b1.partial_cmp(&*b1), p1.partial_cmp(&p1));
                        assert_eq!(b1.partial_cmp(&*b2), p1.partial_cmp(&p2));
                        assert_eq!(b2.partial_cmp(&*b1), p2.partial_cmp(&p1));
                        assert_eq!(b2.partial_cmp(&*b2), p2.partial_cmp(&p2));

                        assert_eq!(b1.partial_cmp(&p1), p1.partial_cmp(&p1));
                        assert_eq!(b1.partial_cmp(&p2), p1.partial_cmp(&p2));
                        assert_eq!(b2.partial_cmp(&p1), p2.partial_cmp(&p1));
                        assert_eq!(b2.partial_cmp(&p2), p2.partial_cmp(&p2));

                        assert_eq!(p1.partial_cmp(b1), p1.partial_cmp(&p1));
                        assert_eq!(p1.partial_cmp(b2), p1.partial_cmp(&p2));
                        assert_eq!(p2.partial_cmp(b1), p2.partial_cmp(&p1));
                        assert_eq!(p2.partial_cmp(b2), p2.partial_cmp(&p2));
                    }
                }
            }
            #[quickcheck]
            fn lt(x: $t, y: $t) {
                let f = Float::from(x);
                let g = Float::from(y);

                let arr = [(&f, x), (&g, y)];
                for &(b1, p1) in &arr {
                    for &(b2, p2) in &arr {
                        assert_eq!(*b1 < *b1, p1 < p1);
                        assert_eq!(*b1 < *b2, p1 < p2);
                        assert_eq!(*b2 < *b1, p2 < p1);
                        assert_eq!(*b2 < *b2, p2 < p2);

                        assert_eq!(*b1 < p1, p1 < p1);
                        assert_eq!(*b1 < p2, p1 < p2);
                        assert_eq!(*b2 < p1, p2 < p1);
                        assert_eq!(*b2 < p2, p2 < p2);

                        assert_eq!(p1 < *b1, p1 < p1);
                        assert_eq!(p1 < *b2, p1 < p2);
                        assert_eq!(p2 < *b1, p2 < p1);
                        assert_eq!(p2 < *b2, p2 < p2);
                    }
                }
            }
            #[quickcheck]
            fn le(x: $t, y: $t) {
                let f = Float::from(x);
                let g = Float::from(y);

                let arr = [(&f, x), (&g, y)];
                for &(b1, p1) in &arr {
                    for &(b2, p2) in &arr {
                        assert_eq!(*b1 <= *b1, p1 <= p1);
                        assert_eq!(*b1 <= *b2, p1 <= p2);
                        assert_eq!(*b2 <= *b1, p2 <= p1);
                        assert_eq!(*b2 <= *b2, p2 <= p2);

                        assert_eq!(*b1 <= p1, p1 <= p1);
                        assert_eq!(*b1 <= p2, p1 <= p2);
                        assert_eq!(*b2 <= p1, p2 <= p1);
                        assert_eq!(*b2 <= p2, p2 <= p2);

                        assert_eq!(p1 <= *b1, p1 <= p1);
                        assert_eq!(p1 <= *b2, p1 <= p2);
                        assert_eq!(p2 <= *b1, p2 <= p1);
                        assert_eq!(p2 <= *b2, p2 <= p2);
                    }
                }
            }
            #[quickcheck]
            fn gt(x: $t, y: $t) {
                let f = Float::from(x);
                let g = Float::from(y);

                let arr = [(&f, x), (&g, y)];
                for &(b1, p1) in &arr {
                    for &(b2, p2) in &arr {
                        assert_eq!(*b1 > *b1, p1 > p1);
                        assert_eq!(*b1 > *b2, p1 > p2);
                        assert_eq!(*b2 > *b1, p2 > p1);
                        assert_eq!(*b2 > *b2, p2 > p2);

                        assert_eq!(*b1 > p1, p1 > p1);
                        assert_eq!(*b1 > p2, p1 > p2);
                        assert_eq!(*b2 > p1, p2 > p1);
                        assert_eq!(*b2 > p2, p2 > p2);

                        assert_eq!(p1 > *b1, p1 > p1);
                        assert_eq!(p1 > *b2, p1 > p2);
                        assert_eq!(p2 > *b1, p2 > p1);
                        assert_eq!(p2 > *b2, p2 > p2);
                    }
                }
            }
            #[quickcheck]
            fn ge(x: $t, y: $t) {
                let f = Float::from(x);
                let g = Float::from(y);

                let arr = [(&f, x), (&g, y)];
                for &(b1, p1) in &arr {
                    for &(b2, p2) in &arr {
                        assert_eq!(*b1 >= *b1, p1 >= p1);
                        assert_eq!(*b1 >= *b2, p1 >= p2);
                        assert_eq!(*b2 >= *b1, p2 >= p1);
                        assert_eq!(*b2 >= *b2, p2 >= p2);

                        assert_eq!(*b1 >= p1, p1 >= p1);
                        assert_eq!(*b1 >= p2, p1 >= p2);
                        assert_eq!(*b2 >= p1, p2 >= p1);
                        assert_eq!(*b2 >= p2, p2 >= p2);

                        assert_eq!(p1 >= *b1, p1 >= p1);
                        assert_eq!(p1 >= *b2, p1 >= p2);
                        assert_eq!(p2 >= *b1, p2 >= p1);
                        assert_eq!(p2 >= *b2, p2 >= p2);
                    }
                }
            }
        }
    }
}

tests!(f32);
tests!(f64);
