use std::fmt::{self, Debug};

use smallvec::Array;

use crate::{KeyValuePair, SmallOrdSet};

/// A view into a single entry in a set, which may either be vacant or occupied.
///
/// This `enum` is constructed from the [`entry`] method on [`SmallOrdSet`].
///
/// [`SmallOrdSet`]: struct.SmallOrdSet.html
/// [`entry`]: struct.SmallOrdSet.html#method.entry
pub enum Entry<'a, A: Array, K> {
    /// An occupied entry.
    Occupied(OccupiedEntry<'a, A>),
    /// A vacant entry.
    Vacant(VacantEntry<'a, A, K>),
}

/// A view into an occupied entry in a `SmallOrdSet`.
/// It is part of the [`Entry`] enum.
///
/// [`Entry`]: enum.Entry.html
pub struct OccupiedEntry<'a, A: Array> {
    set: &'a mut SmallOrdSet<A>,
    idx: usize,
}

/// A view into a vacant entry in a `HashMap`.
/// It is part of the [`Entry`] enum.
///
/// [`Entry`]: enum.Entry.html
pub struct VacantEntry<'a, A: Array, K> {
    set: &'a mut SmallOrdSet<A>,
    idx: usize,
    key: K,
}

impl<'a, A: Array, K> Entry<'a, A, K> {
    pub(crate) fn occupied(set: &'a mut SmallOrdSet<A>, idx: usize) -> Self {
        Entry::Occupied(OccupiedEntry { set, idx })
    }

    pub(crate) fn vacant(set: &'a mut SmallOrdSet<A>, idx: usize, key: K) -> Self {
        Entry::Vacant(VacantEntry { set, idx, key })
    }
}

impl<'a, A, K, V> Entry<'a, A, K>
where
    A: Array<Item = KeyValuePair<K, V>>,
    K: Ord + 'a,
    V: 'a,
{
    /// Ensures a value is in the entry by inserting the default if empty, and returns
    /// a mutable reference to the value in the entry.
    pub fn or_insert(self, default: V) -> &'a mut V {
        match self {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) => entry.insert(default),
        }
    }

    /// Ensures a value is in the entry by inserting the result of the default function if empty,
    /// and returns a mutable reference to the value in the entry.
    pub fn or_insert_with<F: FnOnce() -> V>(self, default: F) -> &'a mut V {
        match self {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) => entry.insert(default()),
        }
    }

    /// Returns a reference to this entry's key.
    pub fn key(&self) -> &K {
        match self {
            Entry::Occupied(entry) => entry.key(),
            Entry::Vacant(entry) => entry.key(),
        }
    }

    /// Provides in-place mutable access to an occupied entry before any
    /// potential inserts into the map.
    pub fn and_modify<F>(self, f: F) -> Self
    where
        F: FnOnce(&mut V),
    {
        match self {
            Entry::Occupied(mut entry) => {
                f(entry.get_mut());
                Entry::Occupied(entry)
            }
            Entry::Vacant(entry) => Entry::Vacant(entry),
        }
    }
}

impl<'a, A> OccupiedEntry<'a, A>
where
    A: Array,
{
    /// Gets a reference to the the entry.
    pub fn get_entry(&self) -> &A::Item {
        &self.set.vec[self.idx]
    }

    /// Take the ownership of the element from the set.
    pub fn remove_entry(self) -> A::Item {
        self.set.vec.remove(self.idx)
    }
}

impl<'a, A, K, V> OccupiedEntry<'a, A>
where
    A: Array<Item = KeyValuePair<K, V>>,
    K: Ord + 'a,
    V: 'a,
{
    /// Gets a reference to the key in the entry.
    pub fn key(&self) -> &K {
        &self.get_entry().key
    }

    /// Gets a reference to the value in the entry.
    pub fn get(&self) -> &V {
        &self.get_entry().value
    }

    /// Gets a mutable reference to the value in the entry.
    ///
    /// If you need a reference to the `OccupiedEntry` which may outlive the
    /// destruction of the `Entry` value, see [`into_mut`].
    ///
    /// [`into_mut`]: #method.into_mut
    pub fn get_mut(&mut self) -> &mut V {
        &mut self.set.vec[self.idx].value
    }

    /// Converts the OccupiedEntry into a mutable reference to the value in the entry
    /// with a lifetime bound to the map itself.
    ///
    /// If you need multiple references to the `OccupiedEntry`, see [`get_mut`].
    ///
    /// [`get_mut`]: #method.get_mut
    pub fn into_mut(self) -> &'a mut V {
        &mut self.set.vec[self.idx].value
    }
}

impl<'a, A: Array, K> VacantEntry<'a, A, K> {
    /// Gets a reference to the key that would be used when inserting a value through the VacantEntry.
    pub fn key(&self) -> &K {
        &self.key
    }

    /// Take ownership of the key.
    pub fn into_key(self) -> K {
        self.key
    }

    /// Insert an element using the given constructor.
    ///
    /// The ordering of the computed element must match that of the key.
    pub fn insert_with<F>(self, f: F) -> &'a mut A::Item
    where
        F: FnOnce(K) -> A::Item,
    {
        let element = f(self.key);
        self.set.vec.insert(self.idx, element);
        &mut self.set.vec[self.idx]
    }
}

impl<'a, A, K, V> VacantEntry<'a, A, K>
where
    A: Array<Item = KeyValuePair<K, V>>,
    K: Ord + 'a,
    V: 'a,
{
    /// Sets the value of the entry with the VacantEntry's key, and returns a mutable reference to it.
    pub fn insert(self, value: V) -> &'a mut V {
        &mut self.insert_with(|key| KeyValuePair { key, value }).value
    }
}

impl<A, K> Debug for Entry<'_, A, K>
where
    A: Array,
    A::Item: Debug,
    K: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Entry::Vacant(ref v) => f.debug_tuple("Entry").field(v).finish(),
            Entry::Occupied(ref o) => f.debug_tuple("Entry").field(o).finish(),
        }
    }
}

impl<A> Debug for OccupiedEntry<'_, A>
where
    A: Array,
    A::Item: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("OccupiedEntry")
            .field("element", self.get_entry())
            .finish()
    }
}

impl<A, K> Debug for VacantEntry<'_, A, K>
where
    A: Array,
    K: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("VacantEntry").finish()
    }
}
