# Laertes

An implementation of the [Shakespeare Programming Language](http://shakespearelang.sourceforge.net/report/shakespeare/shakespeare.html#) as a proc macro in Rust.

# Differences from the original

## additions

- allows gotos using scene/act titles inspired by [drsam94/Spl](https://github.com/drsam94/Spl)

## limitations

- doesn't allow `'`, `"`, or other grouping characters as Rust requires them to all match
- hyphenated words like "flirt-gill" don't work as they are interpreted as identifier/punctuation/identifier instead of one identifier
- multi-word operators like "the remainder of the quotient of" and "square root" have been shortened to a single word "modulus" or "root" because this looks better and makes parsing easier

# Usage

Just import this crate and put the program inside the macro, like in the examples.
You may need to modify existing programs to fit the limitations.
