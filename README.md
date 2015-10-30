# Arbitrary precision floats

[![Build Status](https://travis-ci.org/huonw/float.svg?branch=master)](https://travis-ci.org/huonw/float) [![Coverage Status](https://coveralls.io/repos/huonw/float/badge.svg?branch=master&service=github)](https://coveralls.io/github/huonw/float?branch=master)

The `Float` type is an arbitrary precision float. It supports
correctly rounded computation of arithmetic (`+`, `-`, `*`, `/`),
square roots and precision conversions. It makes no claims to high
performance, but does try to keep it in mind.

Built on [`ramp`](https://github.com/Aatch/ramp), and implemented with
a lot of reference to [1].

[1]: Muller, Jean-Michel; Brisebarre, Nicolas; de Dinechin, Florent; Jeannerod, Claude-Pierre; Lefèvre, Vincent; Melquiond, Guillaume; Revol, Nathalie; Stehlé, Damien; Torres, Serge (2010). *Handbook of Floating-Point Arithmetic.* Birkhäuser. ISBN 978-0-8176-4705-6.
