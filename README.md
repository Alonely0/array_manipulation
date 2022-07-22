# Manipulate Arrays as if they were vectors!

This crate exposes 2 traits that allow manipulating arrays in a vec-like fashion.

Alternatives like [ArrayVec](https://docs.rs/arrayvec/latest/arrayvec/struct.ArrayVec.html) operate over a `[MaybeUninit<T>; N]`-like data structure and panic if the size is overflown. The point of this crate is allowing to "resize" arrays.

If you heavily depend on this crate, probably a linked list will be a much better fit, but this crate is still very useful for one-time operations, operations where using a linked list will give more problems than solutions, coercing arrays or devices where you can't allocate.

This crate works & performs exceptionally well when you have an array that you need to manipulate but you still need to use a fixed array later and not a Vec. In that situation you just avoided 1 `malloc()` & 3 `memcpy()`s best-case scenario and *n* `malloc()`s, 3 `memcpy()`s and a conditional worst-case scenario.


# Merging into core

This crate depends on the experimental (not complete) feature [generic-const-exprs](https://doc.rust-lang.org/beta/unstable-book/language-features/generic-const-exprs.html), so [the Pre-RFC is postponed](https://internals.rust-lang.org/t/add-push-push-back-pop-pop-back-methods-to-fixed-size-arrays/17049) until it doesn't get to a more mature state.
