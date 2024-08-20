# Contributing to Sherlock

Welcome to the Sherlock project! Thank you so much for your interest in 
contributing. :smiley:

This library was initially developed in OCaml, see the original
[repository](https://github.com/miguel-ambrona/sherlock-ocaml). 
I rewrote it entirely in Rust to make it better, faster and more accessible.
I hope this effort captivates many new contributors like you!

## Making contributions

### Reporting issues

If you find a bug, please open an issue on the 
[issue tracker](https://github.com/miguel-ambrona/sherlock/issues). 
Be sure to include a detailed description of the problem.

### Testing the limits of Sherlock

Sherlock is not _complete_ (yet?). There are many illegal positions that are
not captured. Try to find one of them! If you do, please, open an issue and 
label it as https://github.com/miguel-ambrona/sherlock/labels/enhancement.

On the other hand, Sherlock is supposed to be _sound_, i.e. it should never
classify a legal position as illegal. If you find otherwise, please open an
issue and label it as https://github.com/miguel-ambrona/sherlock/labels/bug.

### Creating a new Sherlock rule

Sherlock is made of multiple (usually very simple) deduction 
[rules](https://github.com/miguel-ambrona/sherlock/tree/main/src/rules) that
allow us to derive new information about a position.

A great way to contribute is to add new rules to Sherlock. Every rule must
implement the 
[Rule trait](https://github.com/miguel-ambrona/sherlock/blob/main/src/rules.rs#L5-L24),
have a look at the
[existing rules](https://github.com/miguel-ambrona/sherlock/tree/main/src/rules)
for some examples.

You can pick one of our issues about improving Sherlock's expressivity, labeled
with https://github.com/miguel-ambrona/sherlock/labels/good%20first%20issue and
try to implement a new rule that captures the relevant position.

## Code Style

We use rustfmt, the official Rust code formatter. Before committing your
changes, please run `cargo +nightly fmt` to ensure your code aligns with the
project's style.

## License

By contributing to Sherlock, you agree that your contributions will be licensed
under the MIT License.
See [LICENSE](https://github.com/miguel-ambrona/sherlock/blob/main/LICENSE) for
more details.

Thank you for contributing to Sherlock and helping us make it even better!
