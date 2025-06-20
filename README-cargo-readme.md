# Note on `cargo-readme` usage
Bart Massey 2025-06

I chose to use
[`cargo-readme`](https://crates.io/crates/cargo-readme) to
generate the `README.md` here. Unfortunately, this program
has not been maintained for a couple of years, and is
missing a hard but highly-desirable feature…

It is customary in Rust projects to include an example of
library usage in both the README and Rustdoc for the
library. I prefer to keep that example in my `examples/`
directory so that it can be easily compiled and run.

`cargo-readme` seems entirely suited to handling this task,
but it currently does not. One can include the example in
Rustdoc with

    #![doc = include_str!("../examples/client.rs")]

but `cargo-readme` does not recognize this syntax: it would
have to duplicate `rustdoc`'s include processing to do so.

The arguably *right* answer would be to have `rustdoc` first
process the source code, and then have `cargo-readme`
operate on the result. After some playing around I could
find no reasonable way to do this; `rustdoc` does have the
ability to output JSON, but this exists only on nightly Rust
and the output is… special.

So… two separate mechanisms now. I include the example in
the Rustdoc as described above. In my `README.tpl` for
`cargo-readme` I put the line

    @include example/client.rs

I wrote a Python filter (should have been Rust, LOL) called
`at-include` to expand this. I can then build the
`README.md` with

    cargo readme | at-include >README.md

The current very simple source code to `@include` is below

    #!/usr/bin/python

    from sys import stdin

    for line in stdin:
        if line.startswith("@include"):
            fields = line.split()
            assert len(fields) == 2
            with open(fields[1], "r") as inc:
                for sline in inc:
                    print(sline, end="")
        else:
            print(line, end="")
