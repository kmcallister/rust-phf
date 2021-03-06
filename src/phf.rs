//! Compile time optimized maps
#![crate_id="github.com/sfackler/rust-phf/phf"]
#![doc(html_root_url="http://sfackler.github.io/rust-phf/doc")]
#![crate_type="rlib"]
#![crate_type="dylib"]
#![warn(missing_doc)]

use std::fmt;
use std::hash::Hasher;
use std::hash::sip::SipHasher;
use std::slice;

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
/// static MY_MAP: PhfMap<int> = phf_map! {
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
pub struct PhfMap<T> {
    #[doc(hidden)]
    pub k1: u64,
    #[doc(hidden)]
    pub k2: u64,
    #[doc(hidden)]
    pub disps: &'static [(uint, uint)],
    #[doc(hidden)]
    pub entries: &'static [(&'static str, T)],
}

static LOG_MAX_SIZE: uint = 21;

#[doc(hidden)]
pub static MAX_SIZE: uint = 1 << LOG_MAX_SIZE;

#[doc(hidden)]
#[inline]
pub fn hash(s: &str, k1: u64, k2: u64) -> (uint, uint, uint) {
    let hash = SipHasher::new_with_keys(k1, k2).hash(&s);
    let mask = (MAX_SIZE - 1) as u64;

    ((hash & mask) as uint,
     ((hash >> LOG_MAX_SIZE) & mask) as uint,
     ((hash >> (2 * LOG_MAX_SIZE)) & mask) as uint)
}

#[doc(hidden)]
#[inline]
pub fn displace(f1: uint, f2: uint, d1: uint, d2: uint) -> uint {
    d2 + f1 * d1 + f2
}

impl<T> Container for PhfMap<T> {
    fn len(&self) -> uint {
        self.entries.len()
    }
}

impl<'a, T> Map<&'a str, T> for PhfMap<T> {
    fn find<'a>(&'a self, key: & &str) -> Option<&'a T> {
        self.find_entry(key).map(|&(_, ref v)| v)
    }
}

impl<T: fmt::Show> fmt::Show for PhfMap<T> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        try!(write!(fmt, r"\{"));
        let mut first = true;
        for (k, v) in self.entries() {
            if !first {
                try!(write!(fmt, ", "));
            }
            try!(write!(fmt, "{}: {}", k, v))
            first = false;
        }
        write!(fmt, r"\}")
    }
}

impl<T> PhfMap<T> {
    fn find_entry(&self, key: & &str) -> Option<&'static (&'static str, T)> {
        let (g, f1, f2) = hash(*key, self.k1, self.k2);
        let (d1, d2) = self.disps[g % self.disps.len()];
        let entry @ &(s, _) = &self.entries[displace(f1, f2, d1, d2) %
                                            self.entries.len()];
        if s == *key {
            Some(entry)
        } else {
            None
        }
    }

    /// Returns a reference to the map's internal static instance of the given
    /// key.
    ///
    /// This can be useful for interning schemes.
    pub fn find_key(&self, key: & &str) -> Option<&'static str> {
        self.find_entry(key).map(|&(s, _)| s)
    }

    /// Returns an iterator over the key/value pairs in the map.
    ///
    /// Entries are retuned in an arbitrary but fixed order.
    pub fn entries<'a>(&'a self) -> PhfMapEntries<'a, T> {
        PhfMapEntries { iter: self.entries.iter() }
    }

    /// Returns an iterator over the keys in the map.
    ///
    /// Keys are returned in an arbitrary but fixed order.
    pub fn keys<'a>(&'a self) -> PhfMapKeys<'a, T> {
        PhfMapKeys { iter: self.entries() }
    }

    /// Returns an iterator over the values in the map.
    ///
    /// Values are returned in an arbitrary but fixed order.
    pub fn values<'a>(&'a self) -> PhfMapValues<'a, T> {
        PhfMapValues { iter: self.entries() }
    }
}

/// An iterator over the key/value pairs in a `PhfMap`.
pub struct PhfMapEntries<'a, T> {
    iter: slice::Items<'a, (&'static str, T)>,
}

impl<'a, T> Iterator<(&'static str, &'a T)> for PhfMapEntries<'a, T> {
    fn next(&mut self) -> Option<(&'static str, &'a T)> {
        self.iter.next().map(|&(key, ref value)| (key, value))
    }

    fn size_hint(&self) -> (uint, Option<uint>) {
        self.iter.size_hint()
    }
}

/// An iterator over the keys in a `PhfMap`.
pub struct PhfMapKeys<'a, T> {
    iter: PhfMapEntries<'a, T>,
}

impl<'a, T> Iterator<&'static str> for PhfMapKeys<'a, T> {
    fn next(&mut self) -> Option<&'static str> {
        self.iter.next().map(|(key, _)| key)
    }

    fn size_hint(&self) -> (uint, Option<uint>) {
        self.iter.size_hint()
    }
}

/// An iterator over the values in a `PhfMap`.
pub struct PhfMapValues<'a, T> {
    iter: PhfMapEntries<'a, T>,
}

impl<'a, T> Iterator<&'a T> for PhfMapValues<'a, T> {
    fn next(&mut self) -> Option<&'a T> {
        self.iter.next().map(|(_, value)| value)
    }

    fn size_hint(&self) -> (uint, Option<uint>) {
        self.iter.size_hint()
    }
}

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
/// static MY_SET: PhfSet = phf_set! {
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
pub struct PhfSet {
    #[doc(hidden)]
    pub map: PhfMap<()>
}

impl fmt::Show for PhfSet {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        try!(write!(fmt, r"\{"));
        let mut first = true;
        for entry in self.iter() {
            if !first {
                try!(write!(fmt, ", "));
            }
            try!(write!(fmt, "{}", entry));
            first = false;
        }
        write!(fmt, r"\}")
    }
}

impl Container for PhfSet {
    #[inline]
    fn len(&self) -> uint {
        self.map.len()
    }
}

impl<'a> Set<&'a str> for PhfSet {
    #[inline]
    fn contains(&self, value: & &'a str) -> bool {
        self.map.contains_key(value)
    }

    #[inline]
    fn is_disjoint(&self, other: &PhfSet) -> bool {
        !self.iter().any(|value| other.contains(&value))
    }

    #[inline]
    fn is_subset(&self, other: &PhfSet) -> bool {
        self.iter().all(|value| other.contains(&value))
    }
}

impl PhfSet {
    /// Returns a reference to the set's internal static instance of the given
    /// key.
    ///
    /// This can be useful for interning schemes.
    #[inline]
    pub fn find_key(&self, key: & &str) -> Option<&'static str> {
        self.map.find_key(key)
    }

    /// Returns an iterator over the values in the set.
    ///
    /// Values are returned in an arbitrary but fixed order.
    #[inline]
    pub fn iter<'a>(&'a self) -> PhfSetValues<'a> {
        PhfSetValues { iter: self.map.keys() }
    }
}

/// An iterator over the values in a `PhfSet`.
pub struct PhfSetValues<'a> {
    iter: PhfMapKeys<'a, ()>,
}

impl<'a> Iterator<&'static str> for PhfSetValues<'a> {
    #[inline]
    fn next(&mut self) -> Option<&'static str> {
        self.iter.next()
    }

    #[inline]
    fn size_hint(&self) -> (uint, Option<uint>) {
        self.iter.size_hint()
    }
}

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
/// static MY_MAP: PhfOrderedMap<int> = phf_ordered_map! {
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
pub struct PhfOrderedMap<T> {
    #[doc(hidden)]
    pub k1: u64,
    #[doc(hidden)]
    pub k2: u64,
    #[doc(hidden)]
    pub disps: &'static [(uint, uint)],
    #[doc(hidden)]
    pub idxs: &'static [uint],
    #[doc(hidden)]
    pub entries: &'static [(&'static str, T)],
}

impl<T: fmt::Show> fmt::Show for PhfOrderedMap<T> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        try!(write!(fmt, r"\{"));
        let mut first = true;
        for (k, v) in self.entries() {
            if !first {
                try!(write!(fmt, ", "));
            }
            try!(write!(fmt, "{}: {}", k, v))
            first = false;
        }
        write!(fmt, r"\}")
    }
}

impl<T> Container for PhfOrderedMap<T> {
    fn len(&self) -> uint {
        self.entries.len()
    }
}

impl<'a, T> Map<&'a str, T> for PhfOrderedMap<T> {
    fn find<'a>(&'a self, key: & &str) -> Option<&'a T> {
        self.find_entry(key).map(|&(_, ref v)| v)
    }
}

impl<T> PhfOrderedMap<T> {
    fn find_entry(&self, key: & &str) -> Option<&'static (&'static str, T)> {
        let (g, f1, f2) = hash(*key, self.k1, self.k2);
        let (d1, d2) = self.disps[g % self.disps.len()];
        let idx = self.idxs[displace(f1, f2, d1, d2) % self.idxs.len()];
        let entry @ &(s, _) = &self.entries[idx];

        if s == *key {
            Some(entry)
        } else {
            None
        }
    }

    /// Returns a reference to the map's internal static instance of the given
    /// key.
    ///
    /// This can be useful for interning schemes.
    pub fn find_key(&self, key: & &str) -> Option<&'static str> {
        self.find_entry(key).map(|&(s, _)| s)
    }

    /// Returns an iterator over the key/value pairs in the map.
    ///
    /// Entries are retuned in the same order in which they were defined.
    pub fn entries<'a>(&'a self) -> PhfOrderedMapEntries<'a, T> {
        PhfOrderedMapEntries { iter: self.entries.iter() }
    }

    /// Returns an iterator over the keys in the map.
    ///
    /// Keys are returned in the same order in which they were defined.
    pub fn keys<'a>(&'a self) -> PhfOrderedMapKeys<'a, T> {
        PhfOrderedMapKeys { iter: self.entries() }
    }

    /// Returns an iterator over the values in the map.
    ///
    /// Values are returned in the same order in which they were defined.
    pub fn values<'a>(&'a self) -> PhfOrderedMapValues<'a, T> {
        PhfOrderedMapValues { iter: self.entries() }
    }
}

/// An iterator over the entries in a `PhfOrderedMap`.
pub struct PhfOrderedMapEntries<'a, T> {
    iter: slice::Items<'a, (&'static str, T)>,
}

impl<'a, T> Iterator<(&'static str, &'a T)> for PhfOrderedMapEntries<'a, T> {
    fn next(&mut self) -> Option<(&'static str, &'a T)> {
        self.iter.next().map(|&(key, ref value)| (key, value))
    }

    fn size_hint(&self) -> (uint, Option<uint>) {
        self.iter.size_hint()
    }
}

impl<'a, T> DoubleEndedIterator<(&'static str, &'a T)>
        for PhfOrderedMapEntries<'a, T> {
    fn next_back(&mut self) -> Option<(&'static str, &'a T)> {
        self.iter.next_back().map(|&(key, ref value)| (key, value))
    }
}

impl<'a, T> RandomAccessIterator<(&'static str, &'a T)>
        for PhfOrderedMapEntries<'a, T> {
    fn indexable(&self) -> uint {
        self.iter.indexable()
    }

    fn idx(&mut self, index: uint) -> Option<(&'static str, &'a T)> {
        // FIXME: mozilla/rust#13167
        self.iter.idx(index).map(|pair| {
            let &(key, ref value) = pair;
            (key, value)
        })
    }
}

impl<'a, T> ExactSize<(&'static str, &'a T)> for PhfOrderedMapEntries<'a, T> {}

/// An iterator over the keys in a `PhfOrderedMap`.
pub struct PhfOrderedMapKeys<'a, T> {
    iter: PhfOrderedMapEntries<'a, T>,
}

impl<'a, T> Iterator<&'static str> for PhfOrderedMapKeys<'a, T> {
    fn next(&mut self) -> Option<&'static str> {
        self.iter.next().map(|(key, _)| key)
    }

    fn size_hint(&self) -> (uint, Option<uint>) {
        self.iter.size_hint()
    }
}

impl<'a, T> DoubleEndedIterator<&'static str> for PhfOrderedMapKeys<'a, T> {
    fn next_back(&mut self) -> Option<&'static str> {
        self.iter.next_back().map(|(key, _)| key)
    }
}

impl<'a, T> RandomAccessIterator<&'static str> for PhfOrderedMapKeys<'a, T> {
    fn indexable(&self) -> uint {
        self.iter.indexable()
    }

    fn idx(&mut self, index: uint) -> Option<&'static str> {
        self.iter.idx(index).map(|(key, _)| key)
    }
}

impl<'a, T> ExactSize<&'static str> for PhfOrderedMapKeys<'a, T> {}

/// An iterator over the values in a `PhfOrderedMap`.
pub struct PhfOrderedMapValues<'a, T> {
    iter: PhfOrderedMapEntries<'a, T>,
}

impl<'a, T> Iterator<&'a T> for PhfOrderedMapValues<'a, T> {
    fn next(&mut self) -> Option<&'a T> {
        self.iter.next().map(|(_, value)| value)
    }

    fn size_hint(&self) -> (uint, Option<uint>) {
        self.iter.size_hint()
    }
}

impl<'a, T> DoubleEndedIterator<&'a T> for PhfOrderedMapValues<'a, T> {
    fn next_back(&mut self) -> Option<&'a T> {
        self.iter.next_back().map(|(_, value)| value)
    }
}

impl<'a, T> RandomAccessIterator<&'a T> for PhfOrderedMapValues<'a, T> {
    fn indexable(&self) -> uint {
        self.iter.indexable()
    }

    fn idx(&mut self, index: uint) -> Option<&'a T> {
        self.iter.idx(index).map(|(_, value)| value)
    }
}

impl<'a, T> ExactSize<&'a T> for PhfOrderedMapValues<'a, T> {}

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
/// static MY_SET: PhfOrderedSet = phf_ordered_set! {
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
pub struct PhfOrderedSet {
    #[doc(hidden)]
    pub map: PhfOrderedMap<()>,
}

impl fmt::Show for PhfOrderedSet {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        try!(write!(fmt, r"\{"));
        let mut first = true;
        for entry in self.iter() {
            if !first {
                try!(write!(fmt, ", "));
            }
            try!(write!(fmt, "{}", entry));
            first = false;
        }
        write!(fmt, r"\}")
    }
}

impl Container for PhfOrderedSet {
    #[inline]
    fn len(&self) -> uint {
        self.map.len()
    }
}

impl<'a> Set<&'a str> for PhfOrderedSet {
    #[inline]
    fn contains(&self, value: & &'a str) -> bool {
        self.map.contains_key(value)
    }

    #[inline]
    fn is_disjoint(&self, other: &PhfOrderedSet) -> bool {
        !self.iter().any(|value| other.contains(&value))
    }

    #[inline]
    fn is_subset(&self, other: &PhfOrderedSet) -> bool {
        self.iter().all(|value| other.contains(&value))
    }
}

impl PhfOrderedSet {
    /// Returns a reference to the set's internal static instance of the given
    /// key.
    ///
    /// This can be useful for interning schemes.
    #[inline]
    pub fn find_key(&self, key: & &str) -> Option<&'static str> {
        self.map.find_key(key)
    }

    /// Returns an iterator over the values in the set.
    ///
    /// Values are returned in the same order in which they were defined.
    #[inline]
    pub fn iter<'a>(&'a self) -> PhfOrderedSetValues<'a> {
        PhfOrderedSetValues { iter: self.map.keys() }
    }
}

/// An iterator over the values in a `PhfOrderedSet`.
pub struct PhfOrderedSetValues<'a> {
    iter: PhfOrderedMapKeys<'a, ()>,
}

impl<'a> Iterator<&'static str> for PhfOrderedSetValues<'a> {
    #[inline]
    fn next(&mut self) -> Option<&'static str> {
        self.iter.next()
    }

    #[inline]
    fn size_hint(&self) -> (uint, Option<uint>) {
        self.iter.size_hint()
    }
}

impl<'a> DoubleEndedIterator<&'static str> for PhfOrderedSetValues<'a> {
    #[inline]
    fn next_back(&mut self) -> Option<&'static str> {
        self.iter.next_back()
    }
}

impl<'a> RandomAccessIterator<&'static str> for PhfOrderedSetValues<'a> {
    #[inline]
    fn indexable(&self) -> uint {
        self.iter.indexable()
    }

    #[inline]
    fn idx(&mut self, index: uint) -> Option<&'static str> {
        self.iter.idx(index)
    }
}

impl<'a> ExactSize<&'static str> for PhfOrderedSetValues<'a> {}
