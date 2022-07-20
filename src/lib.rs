#![allow(incomplete_features)]
#![feature(generic_const_exprs)]
#![feature(raw_ref_op)]
#![no_std]
#![doc = include_str!("../README.md")]

use core::{
    mem::{forget, zeroed},
    ptr::{copy_nonoverlapping, drop_in_place},
};

/// Holds the methods for manipulating arrays in a Vec-like fashion.
/// Will (probably) get into core when
/// [generic-const-exprs](https://doc.rust-lang.org/beta/unstable-book/language-features/generic-const-exprs.html)
/// becomes complete.
pub trait ArrayManipulation<T, const N: usize>: Sized {
    /// Takes an array of L elements and appends it at the end of Self.
    /// Performs 2 calls to `memcpy()`, so if your code heavily uses it
    /// maybe a linked list is a better fit for your use case.
    /// # Examples
    /// ```
    /// use array_manipulation::ArrayManipulation;
    ///
    /// let array: [u8; 4] = [1, 2, 3, 4];
    /// let expected = [1, 2, 3, 4, 5, 6, 7];
    /// let result = array.push([5, 6, 7]);
    /// assert_eq!(expected, result);
    /// ```
    fn push<const L: usize>(self, array: [T; L]) -> [T; N + L];

    /// Takes an array of L elements and appends it at the start of Self.
    /// Performs 2 calls to `memcpy()`, so if your code heavily uses it
    /// maybe a linked list is a better fit for your use case.
    /// # Examples
    /// ```
    /// use array_manipulation::ArrayManipulation;
    ///
    /// let array: [u8; 4] = [1, 2, 3, 4];
    /// let expected = [0, 1, 2, 3, 4];
    /// let result = array.push_back([0]);
    /// assert_eq!(expected, result);
    /// ```
    fn push_back<const L: usize>(self, array: [T; L]) -> [T; N + L];

    /// `memcpy()`s all the elements on an array except the last one.
    /// Basically it creates a new fixed-size array with all the
    /// elements except the last one. Won't compile if N == 0.
    /// # Examples
    /// ```
    /// use array_manipulation::ArrayManipulation;
    ///
    /// let array: [u8; 4] = [1, 2, 3, 4];
    /// let expected = [1, 2, 3];
    /// let result = array.pop();
    /// assert_eq!(expected, result);
    /// ```
    fn pop(self) -> [T; N - 1];

    /// `memcpy()`s all the elements on an array except the first one.
    /// Basically it creates a new fixed-size array with all the
    /// elements except the first one. Won't compile if N == 0.
    /// # Examples
    /// ```
    /// use array_manipulation::ArrayManipulation;
    ///
    /// let array: [u8; 4] = [1, 2, 3, 4];
    /// let expected = [2, 3, 4];
    /// let result = array.pop_back();
    /// assert_eq!(expected, result);
    /// ```
    fn pop_back(self) -> [T; N - 1];
}

impl<T, const N: usize> ArrayManipulation<T, N> for [T; N] {
    fn push<const L: usize>(self, array: [T; L]) -> [T; N + L] {
        unsafe {
            let mut result: [T; N + L] = zeroed(); // no real need to use MaybeUninit

            let dst = &raw mut result; // get ptr
            copy_nonoverlapping(&raw const self, dst.cast(), 1); // copy elements

            let dst = &raw mut result[N..]; // offset ptr by N
            copy_nonoverlapping(&raw const array, dst.cast(), 1); // copy elements

            // avoid drop & deallocation of the copied elements
            forget(self);
            forget(array);

            result
        }
    }

    fn push_back<const L: usize>(self, array: [T; L]) -> [T; N + L] {
        unsafe {
            let mut result: [T; N + L] = zeroed(); // no real need to use MaybeUninit

            let dst = &raw mut result; // get ptr
            copy_nonoverlapping(&raw const array, dst.cast(), 1); // copy elements

            let dst = &raw mut result[L..]; // offset ptr by L
            copy_nonoverlapping(&raw const self, dst.cast(), 1); // copy elements

            // avoid drop & deallocation of the copied elements
            forget(self);
            forget(array);

            result
        }
    }

    fn pop(mut self) -> [T; N - 1] {
        unsafe {
            let mut result: [T; N - 1] = zeroed(); // no real need to use MaybeUninit

            let src = &raw const self; // get ptr
            copy_nonoverlapping(src.cast(), &raw mut result, 1); // copy elements

            drop_in_place(&raw mut self[N - 1]); // drop popped element
            forget(self); // avoid drop & deallocation of the copied elements

            result
        }
    }

    fn pop_back(mut self) -> [T; N - 1] {
        unsafe {
            let mut result: [T; N - 1] = zeroed(); // no real need to use MaybeUninit

            let src = &raw const self[1..]; // offset ptr by size_of::<T>()
            copy_nonoverlapping(src.cast(), &raw mut result, 1); // copy elements

            drop_in_place(&raw mut self[0]); // drop popped element
            forget(self); // avoid drop & deallocation of the copied elements

            result
        }
    }
}
