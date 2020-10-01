use std::borrow::Borrow;
use std::cmp::Ordering;
use std::fmt::{self, Debug};
use std::hash::{Hash, Hasher};

use smallvec::Array;

use crate::SmallOrdSet;

/// A key-value pair. When used as the element type of a `SmallOrdSet`, it
/// acts as a map.
///
/// Comparisons of this type only look at the key. This property also applies
/// to `SmallOrdSet<KeyValuePair>`.
#[derive(Copy, Clone, Default)]
pub struct KeyValuePair<K, V> {
    /// The key, used for checking ordering and equality.
    pub key: K,
    /// The value.
    pub value: V,
}

impl<A, K, V> SmallOrdSet<A>
where
    A: Array<Item = KeyValuePair<K, V>>,
    K: Ord,
{
    /// Inserts a key-value pair into the map.
    ///
    /// This function is a convenience wrapper around [`insert`](struct.SmallOrdSet.html#method.insert)
    pub fn insert_value(&mut self, key: K, value: V) -> bool {
        self.insert(KeyValuePair { key, value })
    }

    /// Replaces a key-value pair in the map.
    ///
    /// This function is a convenience wrapper around [`replace`](struct.SmallOrdSet.html#method.replace)
    pub fn replace_value(&mut self, key: K, value: V) -> Option<V> {
        self.replace(KeyValuePair { key, value })
            .map(|kvp| kvp.value)
    }

    /// Removes a key-value pair from the map.
    ///
    /// This function is a convenience wrapper around [`remove`](struct.SmallOrdSet.html#method.remove)
    pub fn remove_value(&mut self, key: &K) -> Option<V> {
        self.remove(key).map(|kvp| kvp.value)
    }

    /// Gets a reference to the value for a key in the map.
    ///
    /// This function is a convenience wrapper around [`get`](struct.SmallOrdSet.html#method.get)
    pub fn get_value<'a>(&'a self, key: &K) -> Option<&'a V>
    where
        K: 'a,
    {
        self.get(key).map(|kvp| &kvp.value)
    }

    /// Gets a mutable reference to the value for a key in the map.
    ///
    /// This function is a convenience wrapper around [`get_mut`](struct.SmallOrdSet.html#method.get_mut).
    /// Unlike `get_mut`, it prevents changing the order of elements by only returning the value part of
    /// the pair.
    pub fn get_value_mut<'a>(&'a mut self, key: &K) -> Option<&'a mut V>
    where
        K: 'a,
    {
        self.get_mut(key).map(|kvp| &mut kvp.value)
    }

    /// Get an iterator over all keys in the map.
    pub fn keys<'a>(&'a self) -> impl Iterator<Item = &'a K> + Clone
    where
        KeyValuePair<K, V>: 'a,
    {
        self.iter().map(|kvp| &kvp.key)
    }

    /// Get an iterator over all values in the map.
    pub fn values<'a>(&'a self) -> impl Iterator<Item = &'a V> + Clone
    where
        KeyValuePair<K, V>: 'a,
    {
        self.iter().map(|kvp| &kvp.value)
    }
}

impl<K: Hash, V> Hash for KeyValuePair<K, V> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.key.hash(state)
    }
}

impl<K: PartialEq, V> PartialEq for KeyValuePair<K, V> {
    fn eq(&self, other: &Self) -> bool {
        PartialEq::eq(&self.key, &other.key)
    }
}

impl<K: Eq, V> Eq for KeyValuePair<K, V> {}

impl<K: PartialOrd, V> PartialOrd for KeyValuePair<K, V> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        PartialOrd::partial_cmp(&self.key, &other.key)
    }
}

impl<K: Ord, V> Ord for KeyValuePair<K, V> {
    fn cmp(&self, other: &Self) -> Ordering {
        Ord::cmp(&self.key, &other.key)
    }
}

impl<K: Debug, V: Debug> Debug for KeyValuePair<K, V> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}: {:?}", self.key, self.value)
    }
}

impl<K, V> Borrow<K> for KeyValuePair<K, V> {
    fn borrow(&self) -> &K {
        &self.key
    }
}
