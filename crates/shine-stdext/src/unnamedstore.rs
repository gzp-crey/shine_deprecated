use crate::arena::PinnedArena;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Mutex, MutexGuard, RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::{fmt, ops};

/// Reference counted indexing of the store items in O(1).
pub struct Index<D>(*mut Entry<D>);

unsafe impl<D> Send for Index<D> {}
unsafe impl<D> Sync for Index<D> {}

impl<D> Index<D> {
    fn new(entry: *mut Entry<D>) -> Index<D> {
        assert!(!entry.is_null());
        unsafe { &(*entry).ref_count.fetch_add(1, Ordering::Relaxed) };
        Index(entry)
    }
}

impl<D> fmt::Debug for Index<D> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        assert!(!self.0.is_null());
        let rc = unsafe { &(*self.0).ref_count.load(Ordering::Relaxed) };
        write!(f, "Index({:p}, rc:{})", self.0, rc)
    }
}

impl<D> PartialEq for Index<D> {
    fn eq(&self, e: &Self) -> bool {
        assert!(!self.0.is_null());
        assert!(!e.0.is_null());
        self.0 == e.0
    }
}

impl<D> Clone for Index<D> {
    fn clone(&self) -> Index<D> {
        assert!(!self.0.is_null());
        unsafe { &(*self.0).ref_count.fetch_add(1, Ordering::Relaxed) };
        Index(self.0)
    }
}

impl<D> Drop for Index<D> {
    fn drop(&mut self) {
        assert!(!self.0.is_null());
        unsafe { &(*self.0).ref_count.fetch_sub(1, Ordering::Relaxed) };
    }
}

/// An entry in the store.
#[derive(Debug)]
struct Entry<D> {
    /// Number of active Index (number of references) to this entry
    ref_count: AtomicUsize,
    /// The stored data
    value: D,
}

// Store data that requires exclusive lock
struct SharedData<D> {
    resources: Vec<*mut Entry<D>>,
}

// D that requires exclusive lock
struct ExclusiveData<D> {
    arena: PinnedArena<Entry<D>>,
    requests: Vec<*mut Entry<D>>,
}

impl<D> ExclusiveData<D> {
    /// Adds a new item to the store
    fn add(&mut self, data: D) -> Index<D> {
        let entry = self.arena.allocate(Entry {
            ref_count: AtomicUsize::new(0),
            value: data,
        });
        let entry = entry as *mut Entry<D>;

        let index = Index::new(entry);
        self.requests.push(entry);
        index
    }
}

/// Thread safe resource store. Simmilar to the HashStore, but items can be aquired only by index,
/// no unique key is present and once all the indices are dropped, item cannot be retreaved from the store.
pub struct Store<D> {
    shared: RwLock<SharedData<D>>,
    exclusive: Mutex<ExclusiveData<D>>,
}

unsafe impl<D> Send for Store<D> {}
unsafe impl<D> Sync for Store<D> {}

impl<D> Store<D> {
    pub fn new() -> Store<D> {
        Store {
            shared: RwLock::new(SharedData { resources: Vec::new() }),
            exclusive: Mutex::new(ExclusiveData {
                arena: PinnedArena::new(),
                requests: Vec::new(),
            }),
        }
    }

    /// Creates a new store with memory allocated for at least capacity items
    pub fn new_with_capacity(_page_size: usize, capacity: usize) -> Store<D> {
        Store {
            shared: RwLock::new(SharedData {
                resources: Vec::with_capacity(capacity),
            }),
            exclusive: Mutex::new(ExclusiveData {
                arena: PinnedArena::new(), /*Arena::_with_capacity(page_size, capacity)*/
                requests: Vec::with_capacity(capacity),
            }),
        }
    }

    /// Aquire read lock.
    pub fn try_read(&self) -> Option<ReadGuard<'_, D>> {
        let shared = self.shared.try_read().ok()?;
        Some(ReadGuard {
            _shared: shared,
            exclusive: &self.exclusive,
        })
    }

    /// Try to aquire write lock.
    pub fn try_write(&self) -> Option<WriteGuard<'_, D>> {
        let shared = self.shared.try_write().ok()?;
        let locked_exclusive = self.exclusive.lock().ok()?;
        Some(WriteGuard {
            shared,
            locked_exclusive,
        })
    }
}

impl<D> Default for Store<D> {
    fn default() -> Self {
        Self::new()
    }
}

impl<D> Drop for Store<D> {
    fn drop(&mut self) {
        let shared = &mut *(self.shared.write().unwrap());
        let exclusive = &mut *(self.exclusive.lock().unwrap());
        let arena = &mut exclusive.arena;
        let requests = &mut exclusive.requests;
        let resources = &mut shared.resources;

        resources.drain_filter(|&mut v| {
            let v = unsafe { &mut *v };
            assert!(v.ref_count.load(Ordering::Relaxed) == 0, "resource leak");
            arena.deallocate(v);
            true
        });

        requests.drain_filter(|&mut v| {
            let v = unsafe { &mut *v };
            assert!(v.ref_count.load(Ordering::Relaxed) == 0, "resource leak");
            arena.deallocate(v);
            true
        });

        assert!(resources.is_empty(), "Leaking resource");
        assert!(requests.is_empty(), "Leaking requests");
        assert!(arena.is_empty(), "Leaking arena, internal store error");
    }
}

/// Guarded read access to a store
pub struct ReadGuard<'a, D> {
    _shared: RwLockReadGuard<'a, SharedData<D>>,
    exclusive: &'a Mutex<ExclusiveData<D>>,
}

impl<'a, D: 'a> ReadGuard<'a, D> {
    pub fn add(&self, data: D) -> Index<D> {
        let mut exclusive = self.exclusive.lock().unwrap();
        exclusive.add(data)
    }

    /// Try to add the item to the store. On success the index is returned.
    /// If operation cannot be carried out immediatelly, data is returned back in the Error.
    pub fn try_add(&self, data: D) -> Result<Index<D>, D> {
        if let Ok(mut exclusive) = self.exclusive.try_lock() {
            Ok(exclusive.add(data))
        } else {
            Err(data)
        }
    }

    pub fn at(&self, index: &Index<D>) -> &D {
        assert!(!index.0.is_null(), "Indexing is invalid");
        let entry = unsafe { &(*index.0) };
        &entry.value
    }
}

impl<'a, 'i, D: 'a> ops::Index<&'i Index<D>> for ReadGuard<'a, D> {
    type Output = D;

    fn index(&self, index: &Index<D>) -> &Self::Output {
        self.at(index)
    }
}

/// Guarded update access to a store
pub struct WriteGuard<'a, D> {
    shared: RwLockWriteGuard<'a, SharedData<D>>,
    locked_exclusive: MutexGuard<'a, ExclusiveData<D>>,
}

impl<'a, D: 'a> WriteGuard<'a, D> {
    pub fn add(&mut self, data: D) -> Index<D> {
        self.locked_exclusive.add(data)
    }

    /// Returns if the store is empty.
    pub fn is_empty(&self) -> bool {
        self.locked_exclusive.requests.is_empty() && self.shared.resources.is_empty()
    }

    /// Merges the requests into the "active" items
    pub fn finalize_requests(&mut self) {
        // Move all resources into the stored resources
        self.shared.resources.append(&mut self.locked_exclusive.requests);
    }

    fn drain_impl<F: FnMut(&mut D) -> bool>(arena: &mut PinnedArena<Entry<D>>, v: &mut Vec<*mut Entry<D>>, filter: &mut F) {
        v.drain_filter(|&mut e| {
            let e = unsafe { &mut *e };
            if e.ref_count.load(Ordering::Relaxed) == 0 {
                let drain = filter(&mut e.value);
                if drain {
                    arena.deallocate(e);
                }
                drain
            } else {
                false
            }
        });
    }

    /// Drain unreferenced elements those specified by the predicate.
    /// In other words, remove all unreferenced resources such that f(&mut data) returns true.
    pub fn drain_unused_filtered<F: FnMut(&mut D) -> bool>(&mut self, mut filter: F) {
        let exclusive = &mut *self.locked_exclusive;
        Self::drain_impl(&mut exclusive.arena, &mut self.shared.resources, &mut filter);
        Self::drain_impl(&mut exclusive.arena, &mut exclusive.requests, &mut filter);
    }

    /// Drain all unreferenced items. Only the referenced items are kept in the store.
    pub fn drain_unused(&mut self) {
        self.drain_unused_filtered(|_| true)
    }

    pub fn at(&self, index: &Index<D>) -> &D {
        assert!(!index.0.is_null(), "Indexing is invalid");
        let entry = unsafe { &(*index.0) };
        &entry.value
    }

    pub fn at_mut(&mut self, index: &Index<D>) -> &mut D {
        assert!(!index.0.is_null(), "Indexing is invalid");
        let entry = unsafe { &mut (*index.0) };
        &mut entry.value
    }
}

impl<'a, 'i, D: 'a> ops::Index<&'i Index<D>> for WriteGuard<'a, D> {
    type Output = D;

    fn index(&self, index: &Index<D>) -> &Self::Output {
        self.at(index)
    }
}

impl<'a, 'i, D: 'a> ops::IndexMut<&'i Index<D>> for WriteGuard<'a, D> {
    fn index_mut(&mut self, index: &Index<D>) -> &mut Self::Output {
        self.at_mut(index)
    }
}
