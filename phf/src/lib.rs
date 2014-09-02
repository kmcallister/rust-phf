//! Compile time optimized maps
//!
//! Keys can be string literals, byte string literals, byte literals, char
//! literals, or any of the fixed-size integral types.
#![doc(html_root_url="http://www.rust-ci.org/sfackler")]
#![warn(missing_doc)]
#![feature(macro_rules)]
#![crate_name="phf"]

use std::fmt;
use std::iter;
use std::slice;
use std::collections::Collection;

pub use shared::PhfHash;

#[path="../../shared/mod.rs"]
mod shared;

/// An immutable map constructed at compile time.
///
/// `PhfMap`s may be created with the `phf_map` macro:
///
/// ```rust
/// # #![feature(phase)]
/// extern crate phf;
/// #[phase(syntax)]
/// extern crate phf_mac;
///
/// use phf::PhfMap;
///
/// static MY_MAP: PhfMap<&'static str, int> = phf_map! {
///    "hello" => 10,
///    "world" => 11,
/// };
///
/// # fn main() {}
/// ```
///
/// # Note
///
/// The fields of this struct are public so that they may be initialized by the
/// `phf_map` macro. They are subject to change at any time and should never
/// be accessed directly.
pub struct PhfMap<K, V> {
    #[doc(hidden)]
    pub key: u64,
    #[doc(hidden)]
    pub disps: &'static [(u32, u32)],
    #[doc(hidden)]
    pub entries: &'static [(K, V)],
}

impl<K, V> Collection for PhfMap<K, V> {
    fn len(&self) -> uint {
        self.entries.len()
    }
}

impl<'a, K: PhfHash+Eq, V> Map<K, V> for PhfMap<K, V> {
    fn find(&self, key: &K) -> Option<&V> {
        self.get_entry(key, |k| key == k).map(|e| {
            let &(_, ref v) = e;
            v
        })
    }
}

impl<K: fmt::Show, V: fmt::Show> fmt::Show for PhfMap<K, V> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        try!(write!(fmt, "{{"));
        let mut first = true;
        for &(ref k, ref v) in self.entries() {
            if !first {
                try!(write!(fmt, ", "));
            }
            try!(write!(fmt, "{}: {}", k, v))
            first = false;
        }
        write!(fmt, "}}")
    }
}

impl<K: PhfHash+Eq, V> Index<K, V> for PhfMap<K, V> {
    fn index(&self, k: &K) -> &V {
        self.find(k).expect("invalid key")
    }
}

impl<K: PhfHash+Eq, V> PhfMap<K, V> {
    /// Returns a reference to the map's internal static instance of the given
    /// key.
    ///
    /// This can be useful for interning schemes.
    pub fn find_key(&self, key: &K) -> Option<&K> {
        self.get_entry(key, |k| key == k).map(|e| {
            let &(ref k, _) = e;
            k
        })
    }
}

impl<K, V> PhfMap<K, V> {
    fn get_entry<T: PhfHash>(&self, key: &T, check: |&K| -> bool) -> Option<&(K, V)> {
        let (g, f1, f2) = key.phf_hash(self.key);
        let (d1, d2) = self.disps[(g % (self.disps.len() as u32)) as uint];
        let entry = &self.entries[(shared::displace(f1, f2, d1, d2) % (self.entries.len() as u32))
                                  as uint];
        let &(ref s, _) = entry;
        if check(s) {
            Some(entry)
        } else {
            None
        }
    }

    /// Like `find`, but can operate on any type that is equivalent to a key.
    pub fn find_equiv<T: PhfHash+Equiv<K>>(&self, key: &T) -> Option<&V> {
        self.get_entry(key, |k| key.equiv(k)).map(|e| {
            let &(_, ref v) = e;
            v
        })
    }

    /// Like `find_key`, but can operate on any type that is equivalent to a
    /// key.
    pub fn find_key_equiv<T: PhfHash+Equiv<K>>(&self, key: &T) -> Option<&K> {
        self.get_entry(key, |k| key.equiv(k)).map(|e| {
            let &(ref k, _) = e;
            k
        })
    }
}

impl<K, V> PhfMap<K, V> {
    /// Returns an iterator over the key/value pairs in the map.
    ///
    /// Entries are retuned in an arbitrary but fixed order.
    pub fn entries<'a>(&'a self) -> PhfMapEntries<'a, K, V> {
        PhfMapEntries { iter: self.entries.iter() }
    }

    /// Returns an iterator over the keys in the map.
    ///
    /// Keys are returned in an arbitrary but fixed order.
    pub fn keys<'a>(&'a self) -> PhfMapKeys<'a, K, V> {
        PhfMapKeys { iter: self.entries().map(|&(ref k, _)| k) }
    }

    /// Returns an iterator over the values in the map.
    ///
    /// Values are returned in an arbitrary but fixed order.
    pub fn values<'a>(&'a self) -> PhfMapValues<'a, K, V> {
        PhfMapValues { iter: self.entries().map(|&(_, ref v)| v) }
    }
}

/// An iterator over the key/value pairs in a `PhfMap`.
pub struct PhfMapEntries<'a, K, V> {
    iter: slice::Items<'a, (K, V)>,
}

impl<'a, K, V> Iterator<&'a (K, V)> for PhfMapEntries<'a, K, V> {
    fn next(&mut self) -> Option<&'a (K, V)> {
        self.iter.next()
    }

    fn size_hint(&self) -> (uint, Option<uint>) {
        self.iter.size_hint()
    }
}

impl<'a, K, V> DoubleEndedIterator<&'a (K, V)> for PhfMapEntries<'a, K, V> {
    fn next_back(&mut self) -> Option<&'a (K, V)> {
        self.iter.next_back()
    }
}

impl<'a, K, V> ExactSize<&'a (K, V)> for PhfMapEntries<'a, K, V> {}

/// An iterator over the keys in a `PhfMap`.
pub struct PhfMapKeys<'a, K, V> {
    iter: iter::Map<'a, &'a (K, V), &'a K, PhfMapEntries<'a, K, V>>,
}

impl<'a, K, V> Iterator<&'a K> for PhfMapKeys<'a, K, V> {
    fn next(&mut self) -> Option<&'a K> {
        self.iter.next()
    }

    fn size_hint(&self) -> (uint, Option<uint>) {
        self.iter.size_hint()
    }
}

impl<'a, K, V> DoubleEndedIterator<&'a K> for PhfMapKeys<'a, K, V> {
    fn next_back(&mut self) -> Option<&'a K> {
        self.iter.next_back()
    }
}

impl<'a, K, V> ExactSize<&'a K> for PhfMapKeys<'a, K, V> {}

/// An iterator over the values in a `PhfMap`.
pub struct PhfMapValues<'a, K, V> {
    iter: iter::Map<'a, &'a (K, V), &'a V, PhfMapEntries<'a, K, V>>,
}

impl<'a, K, V> Iterator<&'a V> for PhfMapValues<'a, K, V> {
    fn next(&mut self) -> Option<&'a V> {
        self.iter.next()
    }

    fn size_hint(&self) -> (uint, Option<uint>) {
        self.iter.size_hint()
    }
}

impl<'a, K, V> DoubleEndedIterator<&'a V> for PhfMapValues<'a, K, V> {
    fn next_back(&mut self) -> Option<&'a V> {
        self.iter.next_back()
    }
}

impl<'a, K, V> ExactSize<&'a V> for PhfMapValues<'a, K, V> {}

/// An immutable set constructed at compile time.
///
/// `PhfSet`s may be created with the `phf_set` macro:
///
/// ```rust
/// # #![feature(phase)]
/// extern crate phf;
/// #[phase(syntax)]
/// extern crate phf_mac;
///
/// use phf::PhfSet;
///
/// static MY_SET: PhfSet<&'static str> = phf_set! {
///    "hello",
///    "world",
/// };
///
/// # fn main() {}
/// ```
///
/// # Note
///
/// The fields of this struct are public so that they may be initialized by the
/// `phf_set` macro. They are subject to change at any time and should never be
/// accessed directly.
pub struct PhfSet<T> {
    #[doc(hidden)]
    pub map: PhfMap<T, ()>
}

impl<T: fmt::Show> fmt::Show for PhfSet<T> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        try!(write!(fmt, "{{"));
        let mut first = true;
        for entry in self.iter() {
            if !first {
                try!(write!(fmt, ", "));
            }
            try!(write!(fmt, "{}", entry));
            first = false;
        }
        write!(fmt, "}}")
    }
}

impl<T> Collection for PhfSet<T> {
    #[inline]
    fn len(&self) -> uint {
        self.map.len()
    }
}

impl<'a, T: PhfHash+Eq> Set<T> for PhfSet<T> {
    #[inline]
    fn contains(&self, value: &T) -> bool {
        self.map.contains_key(value)
    }

    #[inline]
    fn is_disjoint(&self, other: &PhfSet<T>) -> bool {
        !self.iter().any(|value| other.contains(value))
    }

    #[inline]
    fn is_subset(&self, other: &PhfSet<T>) -> bool {
        self.iter().all(|value| other.contains(value))
    }
}

impl<T: PhfHash+Eq> PhfSet<T> {
    /// Returns a reference to the set's internal static instance of the given
    /// key.
    ///
    /// This can be useful for interning schemes.
    #[inline]
    pub fn find_key(&self, key: &T) -> Option<&T> {
        self.map.find_key(key)
    }
}

impl<T> PhfSet<T> {
    /// Like `contains`, but can operate on any type that is equivalent to a
    /// value
    #[inline]
    pub fn contains_equiv<U: PhfHash+Equiv<T>>(&self, key: &U) -> bool {
        self.map.find_equiv(key).is_some()
    }

    /// Like `find_key`, but can operate on any type that is equivalent to a
    /// value
    #[inline]
    pub fn find_key_equiv<U: PhfHash+Equiv<T>>(&self, key: &U) -> Option<&T> {
        self.map.find_key_equiv(key)
    }
}

impl<T> PhfSet<T> {
    /// Returns an iterator over the values in the set.
    ///
    /// Values are returned in an arbitrary but fixed order.
    #[inline]
    pub fn iter<'a>(&'a self) -> PhfSetValues<'a, T> {
        PhfSetValues { iter: self.map.keys() }
    }
}

/// An iterator over the values in a `PhfSet`.
pub struct PhfSetValues<'a, T> {
    iter: PhfMapKeys<'a, T, ()>,
}

impl<'a, T> Iterator<&'a T> for PhfSetValues<'a, T> {
    fn next(&mut self) -> Option<&'a T> {
        self.iter.next()
    }

    fn size_hint(&self) -> (uint, Option<uint>) {
        self.iter.size_hint()
    }
}

impl<'a, T> DoubleEndedIterator<&'a T> for PhfSetValues<'a, T> {
    fn next_back(&mut self) -> Option<&'a T> {
        self.iter.next_back()
    }
}

impl<'a, T> ExactSize<&'a T> for PhfSetValues<'a, T> {}

/// An order-preserving immutable map constructed at compile time.
///
/// Unlike a `PhfMap`, the order of entries in a `PhfOrderedMap` is guaranteed
/// to be the order the entries were listed in.
///
/// `PhfOrderedMap`s may be created with the `phf_ordered_map` macro:
///
/// ```rust
/// # #![feature(phase)]
/// extern crate phf;
/// #[phase(syntax)]
/// extern crate phf_mac;
///
/// use phf::PhfOrderedMap;
///
/// static MY_MAP: PhfOrderedMap<&'static str, int> = phf_ordered_map! {
///    "hello" => 10,
///    "world" => 11,
/// };
///
/// # fn main() {}
/// ```
///
/// # Note
///
/// The fields of this struct are public so that they may be initialized by the
/// `phf_ordered_map` macro. They are subject to change at any time and should
/// never be accessed directly.
pub struct PhfOrderedMap<K, V> {
    #[doc(hidden)]
    pub key: u64,
    #[doc(hidden)]
    pub disps: &'static [(u32, u32)],
    #[doc(hidden)]
    pub idxs: &'static [uint],
    #[doc(hidden)]
    pub entries: &'static [(K, V)],
}

impl<K:fmt::Show, V: fmt::Show> fmt::Show for PhfOrderedMap<K, V> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        try!(write!(fmt, "{{"));
        let mut first = true;
        for &(ref k, ref v) in self.entries() {
            if !first {
                try!(write!(fmt, ", "));
            }
            try!(write!(fmt, "{}: {}", k, v))
            first = false;
        }
        write!(fmt, "}}")
    }
}

impl<K, V> Collection for PhfOrderedMap<K, V> {
    fn len(&self) -> uint {
        self.entries.len()
    }
}

impl<K: PhfHash+Eq, V> Map<K, V> for PhfOrderedMap<K, V> {
    fn find(&self, key: &K) -> Option<&V> {
        self.find_entry(key, |k| k == key).map(|e| {
            let &(_, ref v) = e;
            v
        })
    }
}

impl<K: PhfHash+Eq, V> Index<K, V> for PhfOrderedMap<K, V> {
    fn index(&self, k: &K) -> &V {
        self.find(k).expect("invalid key")
    }
}

impl<K: PhfHash+Eq, V> PhfOrderedMap<K, V> {
    /// Returns a reference to the map's internal static instance of the given
    /// key.
    ///
    /// This can be useful for interning schemes.
    pub fn find_key(&self, key: &K) -> Option<&K> {
        self.find_entry(key, |k| k == key).map(|e| {
            let &(ref k, _) = e;
            k
        })
    }
}

impl<K, V> PhfOrderedMap<K, V> {
    fn find_entry<T: PhfHash>(&self, key: &T, check: |&K| -> bool) -> Option<&(K, V)> {
        let (g, f1, f2) = key.phf_hash(self.key);
        let (d1, d2) = self.disps[(g % (self.disps.len() as u32)) as uint];
        let idx = self.idxs[(shared::displace(f1, f2, d1, d2) % (self.idxs.len() as u32)) as uint];
        let entry = &self.entries[idx];
        let &(ref s, _) = entry;

        if check(s) {
            Some(entry)
        } else {
            None
        }
    }

    /// Like `find`, but can operate on any type that is equivalent to a key.
    pub fn find_equiv<T: PhfHash+Equiv<K>>(&self, key: &T) -> Option<&V> {
        self.find_entry(key, |k| key.equiv(k)).map(|e| {
            let &(_, ref v) = e;
            v
        })
    }

    /// Like `find_key`, but can operate on any type that is equivalent to a
    /// key.
    pub fn find_key_equiv<T: PhfHash+Equiv<K>>(&self, key: &T) -> Option<&K> {
        self.find_entry(key, |k| key.equiv(k)).map(|e| {
            let &(ref k, _) = e;
            k
        })
    }
}

impl<K, V> PhfOrderedMap<K, V> {
    /// Returns an iterator over the key/value pairs in the map.
    ///
    /// Entries are retuned in the same order in which they were defined.
    pub fn entries<'a>(&'a self) -> PhfOrderedMapEntries<'a, K, V> {
        PhfOrderedMapEntries { iter: self.entries.iter() }
    }

    /// Returns an iterator over the keys in the map.
    ///
    /// Keys are returned in the same order in which they were defined.
    pub fn keys<'a>(&'a self) -> PhfOrderedMapKeys<'a, K, V> {
        PhfOrderedMapKeys { iter: self.entries().map(|&(ref k, _)| k) }
    }

    /// Returns an iterator over the values in the map.
    ///
    /// Values are returned in the same order in which they were defined.
    pub fn values<'a>(&'a self) -> PhfOrderedMapValues<'a, K, V> {
        PhfOrderedMapValues { iter: self.entries().map(|&(_, ref v)| v) }
    }
}

/// An iterator over the entries in a `PhfOrderedMap`.
pub struct PhfOrderedMapEntries<'a, K, V> {
    iter: slice::Items<'a, (K, V)>,
}

impl<'a, K, V> Iterator<&'a (K, V)> for PhfOrderedMapEntries<'a, K, V> {
    fn next(&mut self) -> Option<&'a (K, V)> {
        self.iter.next()
    }

    fn size_hint(&self) -> (uint, Option<uint>) {
        self.iter.size_hint()
    }
}

impl<'a, K, V> DoubleEndedIterator<&'a (K, V)>
        for PhfOrderedMapEntries<'a, K, V> {
    fn next_back(&mut self) -> Option<&'a (K, V)> {
        self.iter.next_back()
    }
}

impl<'a, K, V> RandomAccessIterator<&'a (K, V)>
        for PhfOrderedMapEntries<'a, K, V> {
    fn indexable(&self) -> uint {
        self.iter.indexable()
    }

    fn idx(&mut self, index: uint) -> Option<&'a (K, V)> {
        self.iter.idx(index)
    }
}

impl<'a, K, V> ExactSize<&'a (K, V)> for PhfOrderedMapEntries<'a, K, V> {}

/// An iterator over the keys in a `PhfOrderedMap`.
pub struct PhfOrderedMapKeys<'a, K, V> {
    iter: iter::Map<'a, &'a (K, V), &'a K, PhfOrderedMapEntries<'a, K, V>>,
}

impl<'a, K, V> Iterator<&'a K> for PhfOrderedMapKeys<'a, K, V> {
    fn next(&mut self) -> Option<&'a K> {
        self.iter.next()
    }

    fn size_hint(&self) -> (uint, Option<uint>) {
        self.iter.size_hint()
    }
}

impl<'a, K, V> DoubleEndedIterator<&'a K> for PhfOrderedMapKeys<'a, K, V> {
    fn next_back(&mut self) -> Option<&'a K> {
        self.iter.next_back()
    }
}

impl<'a, K, V> RandomAccessIterator<&'a K> for PhfOrderedMapKeys<'a, K, V> {
    fn indexable(&self) -> uint {
        self.iter.indexable()
    }

    fn idx(&mut self, index: uint) -> Option<&'a K> {
        self.iter.idx(index)
    }
}

impl<'a, K, V> ExactSize<&'a K> for PhfOrderedMapKeys<'a, K, V> {}

/// An iterator over the values in a `PhfOrderedMap`.
pub struct PhfOrderedMapValues<'a, K, V> {
    iter: iter::Map<'a, &'a (K, V), &'a V, PhfOrderedMapEntries<'a, K, V>>,
}

impl<'a, K, V> Iterator<&'a V> for PhfOrderedMapValues<'a, K, V> {
    fn next(&mut self) -> Option<&'a V> {
        self.iter.next()
    }

    fn size_hint(&self) -> (uint, Option<uint>) {
        self.iter.size_hint()
    }
}

impl<'a, K, V> DoubleEndedIterator<&'a V> for PhfOrderedMapValues<'a, K, V> {
    fn next_back(&mut self) -> Option<&'a V> {
        self.iter.next_back()
    }
}

impl<'a, K, V> RandomAccessIterator<&'a V> for PhfOrderedMapValues<'a, K, V> {
    fn indexable(&self) -> uint {
        self.iter.indexable()
    }

    fn idx(&mut self, index: uint) -> Option<&'a V> {
        self.iter.idx(index)
    }
}

impl<'a, K, V> ExactSize<&'a V> for PhfOrderedMapValues<'a, K, V> {}

/// An order-preserving immutable set constructed at compile time.
///
/// Unlike a `PhfSet`, the order of entries in a `PhfOrderedSet` is guaranteed
/// to be the order the entries were listed in.
///
/// `PhfOrderedSet`s may be created with the `phf_ordered_set` macro:
///
/// ```rust
/// # #![feature(phase)]
/// extern crate phf;
/// #[phase(syntax)]
/// extern crate phf_mac;
///
/// use phf::PhfOrderedSet;
///
/// static MY_SET: PhfOrderedSet<&'static str> = phf_ordered_set! {
///    "hello",
///    "world",
/// };
///
/// # fn main() {}
/// ```
///
/// # Note
///
/// The fields of this struct are public so that they may be initialized by the
/// `phf_ordered_set` macro. They are subject to change at any time and should
/// never be accessed directly.
pub struct PhfOrderedSet<T> {
    #[doc(hidden)]
    pub map: PhfOrderedMap<T, ()>,
}

impl<T: fmt::Show> fmt::Show for PhfOrderedSet<T> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        try!(write!(fmt, "{{"));
        let mut first = true;
        for entry in self.iter() {
            if !first {
                try!(write!(fmt, ", "));
            }
            try!(write!(fmt, "{}", entry));
            first = false;
        }
        write!(fmt, "}}")
    }
}

impl<T> Collection for PhfOrderedSet<T> {
    #[inline]
    fn len(&self) -> uint {
        self.map.len()
    }
}

impl<T: PhfHash+Eq> Set<T> for PhfOrderedSet<T> {
    #[inline]
    fn contains(&self, value: &T) -> bool {
        self.map.contains_key(value)
    }

    #[inline]
    fn is_disjoint(&self, other: &PhfOrderedSet<T>) -> bool {
        !self.iter().any(|value| other.contains(value))
    }

    #[inline]
    fn is_subset(&self, other: &PhfOrderedSet<T>) -> bool {
        self.iter().all(|value| other.contains(value))
    }
}

impl<T: PhfHash+Eq> PhfOrderedSet<T> {
    /// Returns a reference to the set's internal static instance of the given
    /// key.
    ///
    /// This can be useful for interning schemes.
    #[inline]
    pub fn find_key(&self, key: &T) -> Option<&T> {
        self.map.find_key(key)
    }
}

impl<T> PhfOrderedSet<T> {
    /// Like `contains`, but can operate on any type that is equivalent to a
    /// value
    #[inline]
    pub fn contains_equiv<U: PhfHash+Equiv<T>>(&self, key: &U) -> bool {
        self.map.find_equiv(key).is_some()
    }

    /// Like `find_key`, but can operate on any type that is equivalent to a
    /// value
    #[inline]
    pub fn find_key_equiv<U: PhfHash+Equiv<T>>(&self, key: &U) -> Option<&T> {
        self.map.find_key_equiv(key)
    }

    /// Returns an iterator over the values in the set.
    ///
    /// Values are returned in the same order in which they were defined.
    #[inline]
    pub fn iter<'a>(&'a self) -> PhfOrderedSetValues<'a, T> {
        PhfOrderedSetValues { iter: self.map.keys() }
    }
}

/// An iterator over the values in a `PhfOrderedSet`.
pub struct PhfOrderedSetValues<'a, T> {
    iter: PhfOrderedMapKeys<'a, T, ()>,
}

impl<'a, T> Iterator<&'a T> for PhfOrderedSetValues<'a, T> {
    #[inline]
    fn next(&mut self) -> Option<&'a T> {
        self.iter.next()
    }

    #[inline]
    fn size_hint(&self) -> (uint, Option<uint>) {
        self.iter.size_hint()
    }
}

impl<'a, T> DoubleEndedIterator<&'a T> for PhfOrderedSetValues<'a, T> {
    #[inline]
    fn next_back(&mut self) -> Option<&'a T> {
        self.iter.next_back()
    }
}

impl<'a, T> RandomAccessIterator<&'a T> for PhfOrderedSetValues<'a, T> {
    #[inline]
    fn indexable(&self) -> uint {
        self.iter.indexable()
    }

    #[inline]
    fn idx(&mut self, index: uint) -> Option<&'a T> {
        self.iter.idx(index)
    }
}

impl<'a, T> ExactSize<&'a T> for PhfOrderedSetValues<'a, T> {}
