#![allow(incomplete_features)]
#![feature(generic_const_exprs)]
#![feature(raw_ref_op)]
#![feature(const_trait_impl)]
#![feature(const_maybe_uninit_as_mut_ptr)]
#![feature(const_mut_refs)]
#![feature(const_refs_to_cell)]
#![feature(const_transmute_copy)]
#![feature(const_ptr_read)]
#![feature(specialization)]
#![no_std]
#![doc = include_str!("../README.md")]

use core::{
    mem::{forget, transmute_copy, MaybeUninit},
    ptr::{copy_nonoverlapping, drop_in_place, read},
};

/// Holds the append methods.
/// Will (probably) get into core when
/// [generic-const-exprs](https://doc.rust-lang.org/beta/unstable-book/language-features/generic-const-exprs.html)
/// becomes complete.
pub trait ArrayAppend<T, const N: usize>: Sized {
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
}

/// Holds the pop methods.
/// Will (probably) get into core when
/// [generic-const-exprs](https://doc.rust-lang.org/beta/unstable-book/language-features/generic-const-exprs.html)
/// becomes complete.
pub trait ArrayPop<T, const N: usize>: Sized {
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

impl<T, const N: usize> const ArrayAppend<T, N> for [T; N] {
    default fn push<const L: usize>(self, array: [T; L]) -> [T; N + L] {
        let mut result: MaybeUninit<[T; N + L]> = MaybeUninit::uninit();
        unsafe {
            copy_nonoverlapping(&raw const self, result.as_mut_ptr().cast(), 1); // copy elements
            copy_nonoverlapping(
                &raw const array,
                result.as_mut_ptr().cast::<T>().add(N).cast(),
                1,
            ); // copy elements

            // avoid drop & deallocation of the copied elements
            forget(self);
            forget(array);

            result.assume_init() // initialized
        }
    }

    default fn push_back<const L: usize>(self, array: [T; L]) -> [T; N + L] {
        let mut result: MaybeUninit<[T; N + L]> = MaybeUninit::uninit();
        unsafe {
            copy_nonoverlapping(&raw const array, result.as_mut_ptr().cast(), 1); // copy elements
            copy_nonoverlapping(
                &raw const self,
                result.as_mut_ptr().cast::<T>().add(L).cast(),
                1,
            ); // copy elements

            // avoid drop & deallocation of the copied elements
            forget(self);
            forget(array);

            result.assume_init() // initialized
        }
    }
}

impl<T: Copy, const N: usize> const ArrayAppend<T, N> for [T; N] {
    fn push<const L: usize>(self, array: [T; L]) -> [T; N + L] {
        let mut result: MaybeUninit<[T; N + L]> = MaybeUninit::uninit();
        unsafe {
            *result.as_mut_ptr().cast() = self;
            *result.as_mut_ptr().cast::<T>().add(N).cast() = array;
            result.assume_init() // initialized
        }
    }

    fn push_back<const L: usize>(self, array: [T; L]) -> [T; N + L] {
        let mut result: MaybeUninit<[T; N + L]> = MaybeUninit::uninit();
        unsafe {
            *result.as_mut_ptr().cast() = array; // copy elements
            *result.as_mut_ptr().cast::<T>().add(L).cast() = self;
            result.assume_init() // initialized
        }
    }
}

impl<T, const N: usize> const ArrayPop<T, N> for [T; N] {
    default fn pop(self) -> [T; N - 1] {
        unsafe {
            let result = transmute_copy(&self); // copy
            forget(self); // avoid drop & deallocation of the copied elements
            result
        }
    }

    default fn pop_back(self) -> [T; N - 1] {
        unsafe {
            let result = read((&raw const self).cast::<T>().add(1).cast()); // copy from offset'ed pointer
            forget(self); // avoid drop & deallocation of the copied elements
            result
        }
    }
}

#[allow(drop_bounds)] // specialization stuff
impl<T: Drop, const N: usize> ArrayPop<T, N> for [T; N] {
    fn pop(mut self) -> [T; N - 1] {
        unsafe {
            drop_in_place(&raw mut self[N - 1]); // drop popped element
            let result = transmute_copy(&self); // copy elements
            forget(self); // avoid drop & deallocation of the copied elements
            result
        }
    }

    fn pop_back(mut self) -> [T; N - 1] {
        unsafe {
            drop_in_place(&raw mut self[0]); // drop popped element
            let result = read((&raw const self).cast::<T>().add(1).cast()); // copy from offset'ed pointer
            forget(self); // avoid drop & deallocation of the copied elements
            result
        }
    }
}

// Won't compile for some reason... hopefully specialization will get better soon
// impl<T: Copy, const N: usize> const ArrayPop<T, N> for [T; N] where Self: Copy {
//     fn pop(self) -> [T; N - 1] {
//         unsafe {
//             *(&raw const self).cast()
//         }
//     }

//     fn pop_back(self) -> [T; N - 1] {
//         unsafe {
//             *(&raw const self).cast::<T>().add(1).cast()
//         }
//     }
// }
