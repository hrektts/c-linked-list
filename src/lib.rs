//! A linked list for wrapping a C linked list.
//!
//! This crate provides a wrapper structure of a C linked list and its iterator.

#![feature(shared)]

use std::fmt;
use std::ptr::Shared;

/// A linked list for wrapping a C linked list.
///
/// This `struct` is created by the [`from_const_ptr`] method and
/// [`from_mut_ptr`] method from a C linked list that elements are linked
/// by immutable pointers and mutable pointers, respectively.
/// See their documentation for more.
///
/// [`from_const_ptr`]: struct.CLinkedList.html#method.from_const_ptr
/// [`from_mut_ptr`]: struct.CLinkedList.html#method.from_mut_ptr
pub struct CLinkedList<T, P, F: Fn(&T) -> P> {
    element: Shared<T>,
    next: F,
}

/// An iterator over the elements of a `CLinkedList`.
///
/// This `struct` is created by the [`iter`] method on [`CLinkedList`]. See its
/// documentation for more.
///
/// [`iter`]: struct.CLinkedList.html#method.iter
/// [`CLinkedList`]: struct.CLinkedList.html
pub struct Iter<'a, T: 'a, P: 'a, F: Fn(&T) -> P + 'a> {
    list: &'a CLinkedList<T, P, F>,
    prev: Option<&'a T>,
}

impl<'a, T: 'a, F> fmt::Debug for Iter<'a, T, *const T, F>
where
    T: fmt::Debug,
    F: Fn(&T) -> *const T,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("Iter").field(&self.list).finish()
    }
}

impl<'a, T: 'a, F> fmt::Debug for Iter<'a, T, *mut T, F>
where
    T: fmt::Debug,
    F: Fn(&T) -> *mut T,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("Iter").field(&self.list).finish()
    }
}

/// A mutable iterator over the elements of a `CLinkedList`.
///
/// This `struct` is created by the [`iter_mut`] method on [`CLinkedList`]. See its
/// documentation for more.
///
/// [`iter_mut`]: struct.CLinkedList.html#method.iter_mut
/// [`CLinkedList`]: struct.CLinkedList.html
pub struct IterMut<'a, T: 'a, P: 'a, F: Fn(&T) -> P + 'a> {
    list: &'a CLinkedList<T, P, F>,
    prev: Option<&'a mut T>,
}

impl<'a, T: 'a, F> fmt::Debug for IterMut<'a, T, *mut T, F>
where
    T: fmt::Debug,
    F: Fn(&T) -> *mut T,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("IterMut").field(&self.list).finish()
    }
}

impl<T, F> CLinkedList<T, *const T, F>
where
    F: Fn(&T) -> *const T,
{
    /// Creates a `CLinkedList` by wrapping a C linked list. `head` points to
    /// the head element of the list or is NULL for a list of length 0.
    /// `next` is a function that takes an element and returns an immutable raw
    /// pointer to the next element.
    pub fn from_const_ptr(head: *const T, next: F) -> Option<Self> {
        Shared::new(head as *mut T).map(|p| {
            Self {
                element: p,
                next: next,
            }
        })
    }

    /// Returns the length of the `CLinkedList`.
    pub fn len(&self) -> usize {
        let mut e = self.element;
        let mut ret = 1;
        while let Some(p) = Shared::new((self.next)(unsafe { e.as_ref() }) as *mut T) {
            e = p;
            ret += 1;
        }
        ret
    }

    /// Returns `true` if the `CLinkedList` contains an element equal to the
    /// given value.
    pub fn contains(&self, x: &T) -> bool
    where
        T: PartialEq<T>,
    {
        self.iter().any(|e| e == x)
    }
}

impl<T, F> CLinkedList<T, *mut T, F>
where
    F: Fn(&T) -> *mut T,
{
    /// Creates a `CLinkedList` by wrapping a C linked list. `head` points to
    /// the head element of the list or is NULL for a list of length 0.
    /// `next` is a function that takes an element and returns a mutable raw
    /// pointer to the next element.
    pub fn from_mut_ptr(head: *mut T, next: F) -> Option<Self> {
        Shared::new(head).map(|p| {
            Self {
                element: p,
                next: next,
            }
        })
    }

    /// Provides a forward iterator with mutable references.
    pub fn iter_mut(&mut self) -> IterMut<T, *mut T, F> {
        IterMut {
            list: self,
            prev: None,
        }
    }

    /// Returns the length of the `CLinkedList`.
    pub fn len(&self) -> usize {
        let mut e = self.element;
        let mut ret = 1;
        while let Some(p) = Shared::new((self.next)(unsafe { e.as_ref() })) {
            e = p;
            ret += 1;
        }
        ret
    }

    /// Returns `true` if the `CLinkedList` contains an element equal to the
    /// given value.
    pub fn contains(&self, x: &T) -> bool
    where
        T: PartialEq<T>,
    {
        self.iter().any(|e| e == x)
    }

    /// Provides a mutable reference to the front element, or `None` if the list
    /// is empty.
    pub fn front_mut(&mut self) -> Option<&mut T> {
        if self.element.as_ptr().is_null() {
            None
        } else {
            Some(unsafe { self.element.as_mut() })
        }
    }
}

impl<T, P, F> CLinkedList<T, P, F>
where
    F: Fn(&T) -> P,
{
    /// Provides a forward iterator.
    pub fn iter(&self) -> Iter<T, P, F> {
        Iter {
            list: self,
            prev: None,
        }
    }

    /// Returns `true` if the `CLinkedList` is empty.
    pub fn is_empty(&self) -> bool {
        self.element.as_ptr().is_null()
    }

    /// Provides a reference to the front element, or `None` if the list is
    /// empty.
    pub fn front(&self) -> Option<&T> {
        if self.element.as_ptr().is_null() {
            None
        } else {
            Some(unsafe { self.element.as_ref() })
        }
    }
}

impl<'a, T: 'a, F> Iterator for Iter<'a, T, *const T, F>
where
    F: Fn(&T) -> *const T,
{
    type Item = &'a T;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.prev
            .map_or_else(
                || Some(self.list.element.as_ptr()),
                |prev| Some((self.list.next)(prev) as *mut T),
            )
            .and_then(|p_element| {
                if p_element.is_null() {
                    None
                } else {
                    self.prev = unsafe { p_element.as_ref() };
                    self.prev
                }
            })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.list.len();
        (len, Some(len))
    }
}

impl<'a, T, F: Fn(&T) -> *const T> ExactSizeIterator for Iter<'a, T, *const T, F> {}

impl<'a, T: 'a, F> Iterator for Iter<'a, T, *mut T, F>
where
    F: Fn(&T) -> *mut T,
{
    type Item = &'a T;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.prev
            .map_or_else(
                || Some(self.list.element.as_ptr()),
                |prev| Some((self.list.next)(prev)),
            )
            .and_then(|p_element| {
                if p_element.is_null() {
                    None
                } else {
                    self.prev = unsafe { p_element.as_ref() };
                    self.prev
                }
            })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.list.len();
        (len, Some(len))
    }
}

impl<'a, T, F: Fn(&T) -> *mut T> ExactSizeIterator for Iter<'a, T, *mut T, F> {}

impl<'a, T: 'a, F: Fn(&T) -> *mut T> Iterator for IterMut<'a, T, *mut T, F> {
    type Item = &'a mut T;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let p_element = match self.prev {
            None => self.list.element.as_ptr(),
            Some(ref prev) => (self.list.next)(*prev),
        };
        if p_element.is_null() {
            None
        } else {
            self.prev = unsafe { p_element.as_mut() };
            unsafe { p_element.as_mut() }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.list.len();
        (len, Some(len))
    }
}

impl<'a, T, F: Fn(&T) -> *mut T> ExactSizeIterator for IterMut<'a, T, *mut T, F> {}

impl<'a, T: 'a, F> IntoIterator for &'a CLinkedList<T, *const T, F>
where
    F: Fn(&T) -> *const T,
{
    type Item = &'a T;
    type IntoIter = Iter<'a, T, *const T, F>;

    fn into_iter(self) -> Iter<'a, T, *const T, F> {
        self.iter()
    }
}

impl<'a, T: 'a, F> IntoIterator for &'a CLinkedList<T, *mut T, F>
where
    F: Fn(&T) -> *mut T,
{
    type Item = &'a T;
    type IntoIter = Iter<'a, T, *mut T, F>;

    fn into_iter(self) -> Iter<'a, T, *mut T, F> {
        self.iter()
    }
}

impl<'a, T: 'a, F> IntoIterator for &'a mut CLinkedList<T, *mut T, F>
where
    F: Fn(&T) -> *mut T,
{
    type Item = &'a mut T;
    type IntoIter = IterMut<'a, T, *mut T, F>;

    fn into_iter(self) -> IterMut<'a, T, *mut T, F> {
        self.iter_mut()
    }
}

impl<T: fmt::Debug, F> fmt::Debug for CLinkedList<T, *const T, F>
where
    F: Fn(&T) -> *const T,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}

impl<T: fmt::Debug, F> fmt::Debug for CLinkedList<T, *mut T, F>
where
    F: Fn(&T) -> *mut T,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std;

    #[derive(PartialEq, Eq)]
    struct TestNodeConst {
        val: u32,
        next: *const TestNodeConst,
    }

    fn make_list_const() -> *const TestNodeConst {
        fn malloc<T>(t: T) -> *const T {
            Box::into_raw(Box::new(t)) as *const T
        }

        malloc(TestNodeConst {
            val: 1,
            next: malloc(TestNodeConst {
                val: 2,
                next: malloc(TestNodeConst {
                    val: 3,
                    next: std::ptr::null(),
                }),
            }),
        })
    }

    #[test]
    fn test_using_const_ptr() {
        let ptr: *const TestNodeConst = std::ptr::null();
        assert!(CLinkedList::from_const_ptr(ptr, |n| n.next).is_none());

        let ptr = make_list_const();
        let list = CLinkedList::from_const_ptr(ptr, |n| n.next).unwrap();
        let vs = list.iter().map(|n| n.val).collect::<Vec<_>>();
        assert_eq!(vs, &[1, 2, 3]);
        assert_eq!(list.len(), 3);
        assert!(!list.contains(&TestNodeConst {
            val: 0,
            next: std::ptr::null(),
        }));
        assert!(list.contains(&TestNodeConst {
            val: 3,
            next: std::ptr::null(),
        }));
        assert!(!list.is_empty());
        assert_eq!(list.front().unwrap().val, 1);
    }

    #[derive(PartialEq, Eq)]
    struct TestNodeMut {
        val: u32,
        next: *mut TestNodeMut,
    }

    fn make_list_mut() -> *mut TestNodeMut {
        fn malloc<T>(t: T) -> *mut T {
            Box::into_raw(Box::new(t)) as *mut T
        }

        malloc(TestNodeMut {
            val: 1,
            next: malloc(TestNodeMut {
                val: 2,
                next: malloc(TestNodeMut {
                    val: 3,
                    next: std::ptr::null_mut(),
                }),
            }),
        })
    }

    #[test]
    fn test_using_mut_ptr() {
        let ptr: *mut TestNodeMut = std::ptr::null_mut();
        assert!(CLinkedList::from_mut_ptr(ptr, |n| n.next).is_none());

        let ptr = make_list_mut();
        let mut list = CLinkedList::from_mut_ptr(ptr, |n| n.next).unwrap();
        let vs = list.iter().map(|n| n.val).collect::<Vec<_>>();
        assert_eq!(vs, &[1, 2, 3]);
        assert_eq!(list.len(), 3);
        assert!(!list.contains(&TestNodeMut {
            val: 0,
            next: std::ptr::null_mut(),
        }));
        assert!(list.contains(&TestNodeMut {
            val: 3,
            next: std::ptr::null_mut(),
        }));
        assert!(!list.is_empty());
        assert_eq!(list.front().unwrap().val, 1);

        for node in list.iter_mut() {
            node.val += 1;
        }
        let vs = list.iter().map(|n| n.val).collect::<Vec<_>>();
        assert_eq!(vs, &[2, 3, 4]);
        assert_eq!(list.len(), 3);
        assert!(!list.is_empty());
        assert_eq!(list.front().unwrap().val, 2);

        list.front_mut().unwrap().val = 10;
        assert_eq!(list.front().unwrap().val, 10);
    }
}
