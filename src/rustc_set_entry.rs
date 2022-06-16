use self::RustcEntry::*;
use crate::raw::{Allocator, Global};
use crate::rustc_entry;
use crate::set::HashSet;
use core::fmt::{self, Debug};
use core::hash::{BuildHasher, Hash};

impl<T, S, A> HashSet<T, S, A>
where
    T: Eq + Hash,
    S: BuildHasher,
    A: Allocator + Clone,
{
    /// Gets the given value's corresponding entry in the set for in-place manipulation.
    ///
    /// # Examples
    ///
    /// ```
    /// use hashbrown::HashSet;
    /// use hashbrown::hash_set::RustcEntry::*;
    ///
    /// let mut singles = HashSet::new();
    /// let mut dupes = HashSet::new();
    ///
    /// for ch in "a short treatise on fungi".chars() {
    ///     if let Vacant(dupe_entry) = dupes.rustc_entry(ch) {
    ///         // We haven't already seen a duplicate, so
    ///         // check if we've at least seen it once.
    ///         match singles.rustc_entry(ch) {
    ///             Vacant(single_entry) => {
    ///                 // We found a new character for the first time.
    ///                 single_entry.insert()
    ///             }
    ///             Occupied(single_entry) => {
    ///                 // We've already seen this once, "move" it to dupes.
    ///                 single_entry.remove();
    ///                 dupe_entry.insert();
    ///             }
    ///         }
    ///     }
    /// }
    ///
    /// assert!(!singles.contains(&'t') && dupes.contains(&'t'));
    /// assert!(singles.contains(&'u') && !dupes.contains(&'u'));
    /// assert!(!singles.contains(&'v') && !dupes.contains(&'v'));
    /// ```
    #[cfg_attr(feature = "inline-more", inline)]
    pub fn rustc_entry(&mut self, value: T) -> RustcEntry<'_, T, A> {
        match self.map.rustc_entry(value) {
            rustc_entry::RustcEntry::Occupied(entry) => {
                RustcEntry::Occupied(RustcOccupiedEntry { inner: entry })
            }
            rustc_entry::RustcEntry::Vacant(entry) => {
                RustcEntry::Vacant(RustcVacantEntry { inner: entry })
            }
        }
    }
}

/// A view into a single entry in a set, which may either be vacant or occupied.
///
/// This `enum` is constructed from the [`rustc_entry`] method on [`HashSet`].
///
/// [`HashSet`]: struct.HashSet.html
/// [`rustc_entry`]: struct.HashSet.html#method.rustc_entry
pub enum RustcEntry<'a, T, A = Global>
where
    A: Allocator + Clone,
{
    /// An occupied entry.
    Occupied(RustcOccupiedEntry<'a, T, A>),

    /// A vacant entry.
    Vacant(RustcVacantEntry<'a, T, A>),
}

impl<T: Debug, A: Allocator + Clone> Debug for RustcEntry<'_, T, A> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Vacant(ref v) => f.debug_tuple("Entry").field(v).finish(),
            Occupied(ref o) => f.debug_tuple("Entry").field(o).finish(),
        }
    }
}

/// A view into an occupied entry in a `HashSet`.
/// It is part of the [`RustcEntry`] enum.
///
/// [`RustcEntry`]: enum.RustcEntry.html
pub struct RustcOccupiedEntry<'a, T, A = Global>
where
    A: Allocator + Clone,
{
    inner: rustc_entry::RustcOccupiedEntry<'a, T, (), A>,
}

impl<T: Debug, A: Allocator + Clone> Debug for RustcOccupiedEntry<'_, T, A> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("OccupiedEntry")
            .field("value", self.get())
            .finish()
    }
}

/// A view into a vacant entry in a `HashSet`.
/// It is part of the [`RustcEntry`] enum.
///
/// [`RustcEntry`]: enum.RustcEntry.html
pub struct RustcVacantEntry<'a, T, A = Global>
where
    A: Allocator + Clone,
{
    inner: rustc_entry::RustcVacantEntry<'a, T, (), A>,
}

impl<T: Debug, A: Allocator + Clone> Debug for RustcVacantEntry<'_, T, A> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("VacantEntry").field(self.get()).finish()
    }
}

impl<'a, T, A: Allocator + Clone> RustcEntry<'a, T, A> {
    /// Sets the value of the entry, and returns a RustcOccupiedEntry.
    ///
    /// # Examples
    ///
    /// ```
    /// use hashbrown::HashSet;
    ///
    /// let mut set: HashSet<&str> = HashSet::new();
    /// let entry = set.rustc_entry("horseyland").insert();
    ///
    /// assert_eq!(entry.get(), &"horseyland");
    /// ```
    pub fn insert(self) -> RustcOccupiedEntry<'a, T, A> {
        match self {
            Occupied(entry) => entry,
            Vacant(entry) => entry.insert_entry(),
        }
    }

    /// Ensures a value is in the entry by inserting if it was vacant.
    ///
    /// # Examples
    ///
    /// ```
    /// use hashbrown::HashSet;
    ///
    /// let mut set: HashSet<&str> = HashSet::new();
    ///
    /// set.rustc_entry("poneyland").or_insert();
    /// assert!(set.contains("poneyland"));
    ///
    /// set.rustc_entry("poneyland").or_insert();
    /// assert!(set.contains("poneyland"));
    /// assert_eq!(set.len(), 1);
    /// ```
    #[cfg_attr(feature = "inline-more", inline)]
    pub fn or_insert(self)
    where
        T: Hash,
    {
        if let Vacant(entry) = self {
            entry.insert();
        }
    }

    /// Returns a reference to this entry's value.
    ///
    /// # Examples
    ///
    /// ```
    /// use hashbrown::HashSet;
    ///
    /// let mut set: HashSet<&str> = HashSet::new();
    /// assert_eq!(set.rustc_entry("poneyland").get(), &"poneyland");
    /// ```
    #[cfg_attr(feature = "inline-more", inline)]
    pub fn get(&self) -> &T {
        match *self {
            Occupied(ref entry) => entry.get(),
            Vacant(ref entry) => entry.get(),
        }
    }
}

impl<'a, T, A: Allocator + Clone> RustcOccupiedEntry<'a, T, A> {
    /// Gets a reference to the value in the entry.
    ///
    /// # Examples
    ///
    /// ```
    /// use hashbrown::HashSet;
    ///
    /// let mut set: HashSet<&str> = HashSet::new();
    /// set.rustc_entry("poneyland").or_insert();
    /// assert_eq!(set.rustc_entry("poneyland").get(), &"poneyland");
    /// ```
    #[cfg_attr(feature = "inline-more", inline)]
    pub fn get(&self) -> &T {
        self.inner.key()
    }

    /// Takes the value out of the entry, and returns it.
    ///
    /// # Examples
    ///
    /// ```
    /// use hashbrown::HashSet;
    /// use hashbrown::hash_set::RustcEntry;
    ///
    /// let mut set: HashSet<&str> = HashSet::new();
    /// set.rustc_entry("poneyland").or_insert();
    ///
    /// if let RustcEntry::Occupied(o) = set.rustc_entry("poneyland") {
    ///     assert_eq!(o.remove(), "poneyland");
    /// }
    ///
    /// assert_eq!(set.contains("poneyland"), false);
    /// ```
    #[cfg_attr(feature = "inline-more", inline)]
    pub fn remove(self) -> T {
        self.inner.remove_entry().0
    }

    /// Replaces the entry, returning the old value. The new value in the hash map will be
    /// the value used to create this entry.
    ///
    /// # Examples
    ///
    /// ```
    /// use hashbrown::hash_set::{RustcEntry, HashSet};
    /// use std::rc::Rc;
    ///
    /// let mut set: HashSet<Rc<String>> = HashSet::new();
    /// set.insert(Rc::new("Stringthing".to_string()));
    ///
    /// let my_key = Rc::new("Stringthing".to_string());
    ///
    /// if let RustcEntry::Occupied(entry) = set.rustc_entry(my_key) {
    ///     // Also replace the value with a handle to our other value.
    ///     let old_key: Rc<String> = entry.replace();
    /// }
    ///
    /// ```
    #[cfg_attr(feature = "inline-more", inline)]
    pub fn replace(self) -> T {
        self.inner.replace_key()
    }
}

impl<'a, T, A: Allocator + Clone> RustcVacantEntry<'a, T, A> {
    /// Gets a reference to the value that would be used when inserting
    /// through the `RustcVacantEntry`.
    ///
    /// # Examples
    ///
    /// ```
    /// use hashbrown::HashSet;
    ///
    /// let mut set: HashSet<&str> = HashSet::new();
    /// assert_eq!(set.rustc_entry("poneyland").get(), &"poneyland");
    /// ```
    #[cfg_attr(feature = "inline-more", inline)]
    pub fn get(&self) -> &T {
        self.inner.key()
    }

    /// Take ownership of the value.
    ///
    /// # Examples
    ///
    /// ```
    /// use hashbrown::HashSet;
    /// use hashbrown::hash_set::RustcEntry;
    ///
    /// let mut set: HashSet<&str> = HashSet::new();
    ///
    /// if let RustcEntry::Vacant(v) = set.rustc_entry("poneyland") {
    ///     v.into_value();
    /// }
    /// ```
    #[cfg_attr(feature = "inline-more", inline)]
    pub fn into_value(self) -> T {
        self.inner.into_key()
    }

    /// Sets the value of the entry with the RustcVacantEntry's value.
    ///
    /// # Examples
    ///
    /// ```
    /// use hashbrown::HashSet;
    /// use hashbrown::hash_set::RustcEntry;
    ///
    /// let mut set: HashSet<&str> = HashSet::new();
    ///
    /// if let RustcEntry::Vacant(o) = set.rustc_entry("poneyland") {
    ///     o.insert();
    /// }
    /// assert!(set.contains("poneyland"));
    /// ```
    #[cfg_attr(feature = "inline-more", inline)]
    pub fn insert(self) {
        self.inner.insert(());
    }

    /// Sets the value of the entry with the RustcVacantEntry's value,
    /// and returns a RustcOccupiedEntry.
    ///
    /// # Examples
    ///
    /// ```
    /// use hashbrown::HashSet;
    /// use hashbrown::hash_set::RustcEntry;
    ///
    /// let mut set: HashSet<&str> = HashSet::new();
    ///
    /// if let RustcEntry::Vacant(v) = set.rustc_entry("poneyland") {
    ///     let o = v.insert_entry();
    ///     assert_eq!(o.get(), &"poneyland");
    /// }
    /// ```
    #[cfg_attr(feature = "inline-more", inline)]
    pub fn insert_entry(self) -> RustcOccupiedEntry<'a, T, A> {
        RustcOccupiedEntry {
            inner: self.inner.insert_entry(()),
        }
    }
}
