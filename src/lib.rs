#![allow(incomplete_features)]
#![feature(generic_const_exprs)]
#![feature(raw_ref_op)]
#![feature(const_trait_impl)]
#![feature(const_maybe_uninit_as_mut_ptr)]
#![feature(const_mut_refs)]
#![feature(const_refs_to_cell)]
#![feature(const_transmute_copy)]
#![feature(const_ptr_read)]
#![feature(const_slice_index)]
#![feature(specialization)]
#![cfg_attr(not(test), no_std)]
#![doc = include_str!("../README.md")]

use core::{
    mem::{forget, ManuallyDrop},
    ptr::{drop_in_place, read},
};

/// Holds the append methods.
/// Will (probably) get into core when
/// [generic-const-exprs](https://doc.rust-lang.org/beta/unstable-book/language-features/generic-const-exprs.html)
/// becomes complete.
// TODO implement append_at & concat_at when const exprs become usable enough
pub trait ArrayAdd<T, const N: usize>: Sized {
    /// Inserts an element at the end of Self. Use concat for >1 elements.
    /// # Examples
    /// ```
    /// use array_manipulation::ArrayAdd;
    ///
    /// let array: [u8; 4] = [1, 2, 3, 4];
    /// let expected = [1, 2, 3, 4, 5];
    /// let result = array.append(5);
    /// assert_eq!(expected, result);
    /// ```
    fn append(self, e: T) -> [T; N + 1];

    /// Inserts an element at the start of Self. Use concat for >1 elements.
    /// # Examples
    /// ```
    /// use array_manipulation::ArrayAdd;
    ///
    /// let array: [u8; 4] = [1, 2, 3, 4];
    /// let expected = [0, 1, 2, 3, 4];
    /// let result = array.append_back(0);
    /// assert_eq!(expected, result);
    /// ```
    fn append_back(self, e: T) -> [T; N + 1];

    /// Takes an array of L elements and appends it at the end of Self.
    /// # Examples
    /// ```
    /// use array_manipulation::ArrayAdd;
    ///
    /// let array: [u8; 4] = [1, 2, 3, 4];
    /// let expected = [1, 2, 3, 4, 5, 6, 7];
    /// let result = array.concat([5, 6, 7]);
    /// assert_eq!(expected, result);
    /// ```
    fn concat<const L: usize>(self, array: [T; L]) -> [T; N + L];

    /// Takes an array of L elements and appends it at the end of Self.
    /// # Examples
    /// ```
    /// use array_manipulation::ArrayAdd;
    ///
    /// let array: [u8; 4] = [1, 2, 3, 4];
    /// let expected = [254, 255, 0, 1, 2, 3, 4];
    /// let result = array.concat_back([254, 255, 0]);
    /// assert_eq!(expected, result);
    /// ```
    fn concat_back<const L: usize>(self, array: [T; L]) -> [T; N + L];
}

/// Holds the pop methods.
/// Will (probably) get into core when
/// [generic-const-exprs](https://doc.rust-lang.org/beta/unstable-book/language-features/generic-const-exprs.html)
/// becomes complete.
// TODO implement pop_at when const exprs become usable enough
pub trait ArrayRemove<T, const N: usize>: Sized {
    /// `memcpy()`s all the elements on an array except the first L ones.
    /// Basically it creates a new fixed-size array with all the
    /// elements except the first L ones. Won't compile if N == 0.
    /// # Examples
    /// ```
    /// use array_manipulation::ArrayManipulation;
    ///
    /// let array: [u8; 4] = [1, 2, 3, 4];
    /// let expected = [3, 4];
    /// let result = array.truncate_start(2);
    /// assert_eq!(expected, result);
    /// ```
    fn truncate_start<const L: usize>(self) -> [T; N - L];

    /// Creates a new fixed-size array with all the
    /// elements except the L ones.
    /// Won't compile if N == 0.
    /// # Examples
    /// ```
    /// use array_manipulation::ArrayManipulation;
    ///
    /// let array: [u8; 4] = [1, 2, 3, 4];
    /// let expected = [1, 2];
    /// let result = array.truncate_end(2);
    /// assert_eq!(expected, result);
    /// ```
    fn truncate_end<const L: usize>(self) -> [T; N - L];
}

#[repr(C)]
struct Contiguous<A, B>(A, B);

// from https://github.com/Vurich/const-concat/issues/13#issue-1190857331
const unsafe fn transmute_unchecked<From, To>(from: From) -> To {
    union Transmute<From, To> {
        from: ManuallyDrop<From>,
        to: ManuallyDrop<To>,
    }

    ManuallyDrop::into_inner(
        Transmute {
            from: ManuallyDrop::new(from),
        }
        .to,
    )
}

impl<T, const N: usize> const ArrayAdd<T, N> for [T; N] {
    fn concat<const L: usize>(self, array: [T; L]) -> [T; N + L] {
        unsafe {
            // join contiguous memory in a single array
            transmute_unchecked(Contiguous(self, array))
        }
    }

    fn concat_back<const L: usize>(self, array: [T; L]) -> [T; N + L] {
        unsafe {
            // join contiguous memory in a single array
            transmute_unchecked(Contiguous(array, self))
        }
    }

    fn append(self, element: T) -> [T; N + 1] {
        unsafe {
            // join contiguous memory in a single array
            transmute_unchecked(Contiguous(self, element))
        }
    }

    fn append_back(self, element: T) -> [T; N + 1] {
        unsafe {
            // join contiguous memory in a single array
            transmute_unchecked(Contiguous(element, self))
        }
    }
}

impl<T, const N: usize> const ArrayRemove<T, N> for [T; N] {
    default fn truncate_start<const L: usize>(self) -> [T; N - L] {
        unsafe {
            let result = read((&raw const self).cast::<T>().add(L).cast()); // copy from offset'ed pointer
            forget(self); // avoid drop & deallocation of the copied elements
            result
        }
    }

    default fn truncate_end<const L: usize>(self) -> [T; N - L] {
        unsafe {
            transmute_unchecked(self) // resize self
        }
    }
}

#[allow(drop_bounds)] // specialization stuff
impl<T: Drop, const N: usize> ArrayRemove<T, N> for [T; N] {
    fn truncate_start<const L: usize>(mut self) -> [T; N - L] {
        unsafe {
            let result = read((&raw const self).cast::<T>().add(L).cast()); // copy from offset'ed pointer
            drop_in_place(&raw mut self[..L]); // drop popped elements
            forget(self); // avoid drop & deallocation of the copied elements
            result
        }
    }
    fn truncate_end<const L: usize>(mut self) -> [T; N - L] {
        unsafe {
            drop_in_place(&raw mut self[L..]); // drop popped elements
            transmute_unchecked(self) // resize self
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{ArrayAdd, ArrayRemove};

    #[test]
    fn append_noncopy() {
        let input = [vec![1, 2], vec![3, 4]];
        let expected = [vec![1, 2], vec![3, 4], vec![5, 6, 7]];
        let result = input.append(vec![5, 6, 7]);
        assert_eq!(expected, result)
    }

    #[test]
    fn append_back_noncopy() {
        let input = [vec![1, 2], vec![3, 4]];
        let expected = [vec![254, 255, 0], vec![1, 2], vec![3, 4]];
        let result = input.append_back(vec![254, 255, 0]);
        assert_eq!(expected, result)
    }

    #[test]
    fn concat_noncopy() {
        let input = [vec![1, 2], vec![3, 4]];
        let expected = [
            vec![1, 2],
            vec![3, 4],
            vec![5, 6, 7],
            vec![8, 9],
            vec![10, 11, 12],
        ];
        let result = input.concat([vec![5, 6, 7], vec![8, 9], vec![10, 11, 12]]);
        assert_eq!(expected, result)
    }

    #[test]
    fn concat_back_noncopy() {
        let input = [vec![1, 2], vec![3, 4]];
        let expected = [
            vec![249, 250, 251],
            vec![252, 253],
            vec![254, 255, 0],
            vec![1, 2],
            vec![3, 4],
        ];
        let result = input.concat_back([vec![249, 250, 251], vec![252, 253], vec![254, 255, 0]]);
        assert_eq!(expected, result)
    }

    #[test]
    fn truncate_start_noncopy() {
        let input = [vec![1, 2], vec![3, 4], vec![5, 6], vec![7, 8]];
        let expected = [vec![5, 6], vec![7, 8]];
        let result = input.truncate_start::<2>();
        assert_eq!(expected, result)
    }

    #[test]
    fn truncate_end_noncopy() {
        let input = [vec![1, 2], vec![3, 4], vec![5, 6], vec![7, 8]];
        let expected = [vec![1, 2], vec![3, 4]];
        let result = input.truncate_end::<2>();
        assert_eq!(expected, result)
    }

    #[test]
    fn append_copy() {
        let input = [1, 2, 3, 4];
        let expected = [1, 2, 3, 4, 5];
        let result = input.append(5);
        assert_eq!(expected, result)
    }

    #[test]
    fn append_back_copy() {
        let input = [1, 2, 3, 4];
        let expected = [0, 1, 2, 3, 4];
        let result = input.append_back(0);
        assert_eq!(expected, result)
    }

    #[test]
    fn concat_copy() {
        let input = [1, 2, 3, 4];
        let expected = [1, 2, 3, 4, 5, 6, 7];
        let result = input.concat([5, 6, 7]);
        assert_eq!(expected, result)
    }

    #[test]
    fn concat_back_copy() {
        let input = [1, 2, 3, 4];
        let expected = [254, 255, 0, 1, 2, 3, 4];
        let result = input.concat_back([254, 255, 0]);
        assert_eq!(expected, result)
    }

    #[test]
    fn truncate_start_copy() {
        let input = [1, 2, 3, 4];
        let expected = [3, 4];
        let result = input.truncate_start::<2>();
        assert_eq!(expected, result)
    }

    #[test]
    fn truncate_end_copy() {
        let input = [1, 2, 3, 4];
        let expected = [1, 2];
        let result = input.truncate_end::<2>();
        assert_eq!(expected, result)
    }
}
