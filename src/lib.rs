//! This crate provides the [`SmallOrdSet`](struct.SmallOrdSet.html) type, a set data-structure
//! represented by a sorted `SmallVec`.

#![deny(
    missing_debug_implementations,
    missing_copy_implementations,
    missing_docs
)]

mod entry;
mod map;

pub use self::entry::*;
pub use self::map::*;

use std::borrow::Borrow;
use std::cmp::Ordering;
use std::fmt::{self, Debug};
use std::hash::{Hash, Hasher};
use std::iter::FromIterator;
use std::mem::replace;
use std::ops::{Deref, Index, RangeBounds};
use std::slice::{self, SliceIndex};

use smallvec::{self, Array, SmallVec};

/// A set represented by a sorted `SmallVec`.
pub struct SmallOrdSet<A: Array> {
    vec: SmallVec<A>,
}

impl<A: Array> SmallOrdSet<A> {
    /// Make a new, empty, `SmallOrdSet`.
    pub fn new() -> Self {
        SmallOrdSet::default()
    }

    /// Get a slice containing the whole set in sorted order.
    pub fn as_slice(&self) -> &[A::Item] {
        self.vec.as_slice()
    }

    /// The number of elements the set can hold without reallocating
    pub fn capacity(&self) -> usize {
        self.vec.capacity()
    }

    /// Remove all elements from the set.
    pub fn clear(&mut self) {
        self.vec.clear();
    }

    /// Creates a draining iterator that removes the specified range in the set
    /// and yields the removed items.
    ///
    /// Note 1: The element range is removed even if the iterator is only
    /// partially consumed or not consumed at all.
    ///
    /// Note 2: It is unspecified how many elements are removed from the set
    /// if the `Drain` value is leaked.
    ///
    /// # Panics
    ///
    /// Panics if the starting point is greater than the end point or if
    /// the end point is greater than the length of the set.
    pub fn drain<R>(&mut self, range: R) -> smallvec::Drain<A>
    where
        R: RangeBounds<usize>,
    {
        self.vec.drain(range)
    }

    /// Re-allocate to set the capacity to `max(new_cap, inline_size())`.
    ///
    /// Panics if `new_cap` is less than the set's length.
    pub fn grow(&mut self, new_cap: usize) {
        self.vec.grow(new_cap)
    }

    /// The maximum number of elements this set can hold inline
    pub fn inline_size(&self) -> usize {
        self.vec.inline_size()
    }

    /// Convert the set into the inner `SmallVec`.
    pub fn into_vec(self) -> SmallVec<A> {
        self.vec
    }

    /// The number of elements in the set.
    pub fn len(&self) -> usize {
        self.vec.len()
    }

    /// Returns `true` if the vector is empty.
    pub fn is_empty(&self) -> bool {
        self.vec.is_empty()
    }

    /// Reserve capacity for `additional` more elements to be inserted.
    ///
    /// May reserve more space to avoid frequent reallocations.
    pub fn reserve(&mut self, additional: usize) {
        self.vec.reserve(additional)
    }

    /// Reserve the minimum capacity for `additional` more elements to be inserted.
    ///
    /// Panics if the new capacity overflows `usize`.
    pub fn reserve_exact(&mut self, additional: usize) {
        self.vec.reserve_exact(additional)
    }

    /// Retains only the elements specified by the predicate.
    ///
    /// In other words, remove all elements `e` such that `f(&e)` returns `false`.
    /// This method operates in place and preserves the order of the retained
    /// elements.
    pub fn retain<F>(&mut self, f: F)
    where
        F: FnMut(&mut A::Item) -> bool,
    {
        self.vec.retain(f)
    }

    /// Construct a new [`SmallOrdSet`](struct.SmallOrdSet.html) from a sorted `SmallVec`. `vec` must
    /// be sorted and may not contain duplicate elements.
    ///
    /// # Safety
    ///
    /// Failure to uphold the restrictions on the `vec` parameter will not cause memory unsafety,
    /// however the result of any operations on the resulting `SmallOrdSet` is unspecified.
    pub fn from_vec_unchecked(vec: SmallVec<A>) -> Self {
        SmallOrdSet { vec }
    }

    /// Construct an iterator over the set, in ascending order.
    pub fn iter(&self) -> slice::Iter<A::Item> {
        self.vec.iter()
    }

    /// Returns a reference to the first element in the set, if any. This element is always the minimum
    /// of all elements in the set.
    pub fn first(&self) -> Option<&A::Item> {
        self.vec.first()
    }

    /// Returns a reference to the first element in the set, if any. This element is always the maximum
    /// of all elements in the set.
    pub fn last(&self) -> Option<&A::Item> {
        self.vec.last()
    }
}

impl<A> SmallOrdSet<A>
where
    A: Array,
    A::Item: Ord,
{
    /// Moves all elements from `other` into `Self`, leaving other `empty`.
    pub fn append(&mut self, other: &mut Self) {
        self.extend(other.drain(..))
    }

    /// Construct a new [`SmallOrdSet`](struct.SmallOrdSet.html) from a `SmallVec`. The vector will be
    /// sorted and duplicate elements removed.
    pub fn from_vec(vec: SmallVec<A>) -> Self {
        let mut set = SmallOrdSet::from_vec_unchecked(vec);
        set.sort_and_dedup();
        set
    }

    /// Constructs a new [`SmallOrdSet`](struct.SmallOrdSet.html) on the stack from an `A` without
    /// copying elements.
    pub fn from_buf(buf: A) -> Self {
        SmallOrdSet::from_vec(buf.into())
    }

    /// Adds an element to the set.
    ///
    /// If the set did not have this element present, `true` is returned.
    ///
    /// If the set did have this element present, `false` is returned, and the
    /// entry is not updated.
    ///
    /// # Examples
    ///
    /// ```
    /// use small_ord_set::SmallOrdSet;
    ///
    /// let mut set = SmallOrdSet::<[u32; 4]>::new();
    ///
    /// assert_eq!(set.insert(2), true);
    /// assert_eq!(set.insert(2), false);
    /// assert_eq!(set.len(), 1);
    /// ```
    pub fn insert(&mut self, element: A::Item) -> bool {
        match self.find(&element) {
            Ok(_) => false,
            Err(idx) => {
                self.vec.insert(idx, element);
                true
            }
        }
    }

    /// Adds a element to the set, replacing the existing element, if any, that is equal to the given
    /// one. Returns the replaced element.
    ///
    /// # Examples
    ///
    /// ```
    /// use small_ord_set::SmallOrdSet;
    ///
    /// let mut set = SmallOrdSet::<[u32; 4]>::new();
    ///
    /// assert_eq!(set.replace(2), None);
    /// assert_eq!(set.replace(2), Some(2));
    /// assert_eq!(set.len(), 1);
    /// ```
    pub fn replace(&mut self, element: A::Item) -> Option<A::Item> {
        match self.find(&element) {
            Ok(idx) => Some(replace(&mut self.vec[idx], element)),
            Err(idx) => {
                self.vec.insert(idx, element);
                None
            }
        }
    }

    /// Removes and returns the element in the set, if any, that is equal to the given one.
    ///
    /// The element may be any borrowed form of the set's element type,
    /// but the ordering on the borrowed form *must* match the
    /// ordering on the element type.
    ///
    /// # Examples
    ///
    /// ```
    /// use small_ord_set::SmallOrdSet;
    ///
    /// let mut set = SmallOrdSet::<[u32; 4]>::new();
    ///
    /// set.insert(2);
    /// assert_eq!(set.remove(&2), Some(2));
    /// assert_eq!(set.remove(&2), None);
    /// ```
    pub fn remove<Q>(&mut self, element: &Q) -> Option<A::Item>
    where
        A::Item: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        match self.find(element) {
            Ok(idx) => Some(self.vec.remove(idx)),
            Err(_) => None,
        }
    }

    /// Returns `true` if the set contains an element.
    ///
    /// The value may be any borrowed form of the set's element type,
    /// but the ordering on the borrowed form *must* match the
    /// ordering on the element type.
    ///
    /// # Examples
    ///
    /// ```
    /// use small_ord_set::SmallOrdSet;
    ///
    /// let set = SmallOrdSet::from_buf([1, 2, 3]);
    /// assert_eq!(set.contains(&1), true);
    /// assert_eq!(set.contains(&4), false);
    /// ```
    pub fn contains<Q>(&self, element: &Q) -> bool
    where
        A::Item: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        self.find(element).is_ok()
    }

    /// Returns a reference to the element in the set, if any, that is equal to the given value.
    ///
    /// The value may be any borrowed form of the set's element type,
    /// but the ordering on the borrowed form *must* match the
    /// ordering on the element type.
    pub fn get<Q>(&self, element: &Q) -> Option<&A::Item>
    where
        A::Item: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        match self.find(element) {
            Ok(idx) => Some(&self.vec[idx]),
            Err(_) => None,
        }
    }

    /// Returns a mutable reference to the element in the set, if any, that is equal to the given
    /// value. It is an error to mutate the element such that its ordering changes.
    ///
    /// The value may be any borrowed form of the set's element type,
    /// but the ordering on the borrowed form *must* match the
    /// ordering on the element type.
    pub fn get_mut<Q>(&mut self, element: &Q) -> Option<&mut A::Item>
    where
        A::Item: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        match self.find(element) {
            Ok(idx) => Some(&mut self.vec[idx]),
            Err(_) => None,
        }
    }

    /// Gets the given key's corresponding entry in the map for in-place manipulation.
    ///
    /// # Examples
    ///
    /// ```
    /// use small_ord_set::{SmallOrdSet, KeyValuePair};
    ///
    /// let mut letters = SmallOrdSet::<[KeyValuePair<char, u32>; 8]>::new();
    ///
    /// for ch in "a short treatise on fungi".chars() {
    ///     let counter = letters.entry(ch).or_insert(0);
    ///     *counter += 1;
    /// }
    ///
    /// assert_eq!(letters.get_value(&'s'), Some(&2));
    /// assert_eq!(letters.get_value(&'t'), Some(&3));
    /// assert_eq!(letters.get_value(&'u'), Some(&1));
    /// assert_eq!(letters.get_value(&'y'), None);
    /// ```
    pub fn entry<Q>(&mut self, key: Q) -> Entry<A, Q>
    where
        A::Item: Borrow<Q>,
        Q: Ord,
    {
        match self.find(&key) {
            Ok(idx) => Entry::occupied(self, idx),
            Err(idx) => Entry::vacant(self, idx, key),
        }
    }

    fn find<Q>(&self, element: &Q) -> Result<usize, usize>
    where
        A::Item: Borrow<Q>,
        Q: Ord + ?Sized,
    {
        self.vec
            .binary_search_by(|probe| Ord::cmp(probe.borrow(), element))
    }

    fn sort_and_dedup(&mut self) {
        self.vec.sort();
        self.vec.dedup();
    }
}

impl<A: Array> AsRef<[A::Item]> for SmallOrdSet<A> {
    fn as_ref(&self) -> &[A::Item] {
        self.as_slice()
    }
}

impl<A: Array> Borrow<[A::Item]> for SmallOrdSet<A> {
    fn borrow(&self) -> &[A::Item] {
        self.as_slice()
    }
}

impl<A> Clone for SmallOrdSet<A>
where
    A: Array,
    A::Item: Clone,
{
    fn clone(&self) -> Self {
        SmallOrdSet::from_vec_unchecked(self.vec.clone())
    }

    fn clone_from(&mut self, source: &Self) {
        self.vec.clone_from(&source.vec)
    }
}

impl<A> Debug for SmallOrdSet<A>
where
    A: Array,
    A::Item: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_set().entries(self.iter()).finish()
    }
}

impl<A: Array> Default for SmallOrdSet<A> {
    fn default() -> Self {
        SmallOrdSet::from_vec_unchecked(Default::default())
    }
}

impl<A: Array> Deref for SmallOrdSet<A> {
    type Target = [A::Item];

    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl<A> Eq for SmallOrdSet<A>
where
    A: Array,
    A::Item: Eq,
{
}

impl<A> Extend<A::Item> for SmallOrdSet<A>
where
    A: Array,
    A::Item: Ord,
{
    fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = A::Item>,
    {
        self.vec.extend(iter);
        self.sort_and_dedup();
    }
}

impl<A> From<A> for SmallOrdSet<A>
where
    A: Array,
    A::Item: Ord,
{
    fn from(buf: A) -> Self {
        SmallOrdSet::from_buf(buf)
    }
}

impl<A> From<SmallVec<A>> for SmallOrdSet<A>
where
    A: Array,
    A::Item: Ord,
{
    fn from(vec: SmallVec<A>) -> Self {
        SmallOrdSet::from_vec(vec)
    }
}

impl<A> FromIterator<A::Item> for SmallOrdSet<A>
where
    A: Array,
    A::Item: Ord,
{
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = A::Item>,
    {
        SmallOrdSet::from_vec(FromIterator::from_iter(iter))
    }
}

impl<A> Hash for SmallOrdSet<A>
where
    A: Array,
    A::Item: Hash,
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.vec.hash(state)
    }
}

impl<A: Array, I: SliceIndex<[A::Item]>> Index<I> for SmallOrdSet<A> {
    type Output = I::Output;

    fn index(&self, index: I) -> &Self::Output {
        self.vec.index(index)
    }
}

impl<A: Array> IntoIterator for SmallOrdSet<A> {
    type IntoIter = smallvec::IntoIter<A>;
    type Item = A::Item;

    fn into_iter(self) -> Self::IntoIter {
        self.vec.into_iter()
    }
}

impl<'a, A: Array> IntoIterator for &'a SmallOrdSet<A> {
    type IntoIter = slice::Iter<'a, A::Item>;
    type Item = &'a A::Item;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<A> Ord for SmallOrdSet<A>
where
    A: Array,
    A::Item: Ord,
{
    fn cmp(&self, other: &Self) -> Ordering {
        Ord::cmp(&self.vec, &other.vec)
    }
}

impl<A> PartialEq for SmallOrdSet<A>
where
    A: Array,
    A::Item: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        PartialEq::eq(&self.vec, &other.vec)
    }
}

impl<A> PartialOrd for SmallOrdSet<A>
where
    A: Array,
    A::Item: Ord,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        PartialOrd::partial_cmp(&self.vec, &other.vec)
    }
}
