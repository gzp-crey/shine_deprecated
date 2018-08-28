pub trait VectorMerge {
    type Item;

    fn contains(&mut self, idx: usize) -> bool;

    fn lower_bound_index(&mut self, idx: usize) -> Option<usize>;

    fn get_unchecked(&mut self, idx: usize) -> Self::Item;
}

pub trait VectorMergeExt: VectorMerge {
    fn get(&mut self, idx: usize) -> Option<Self::Item> {
        if self.contains(idx) {
            Some(self.get_unchecked(idx))
        } else {
            None
        }
    }

    fn merge(&mut self) -> VectorMergeIter<Self>
    where
        Self: Sized,
    {
        VectorMergeIter {
            idx: self.lower_bound_index(0),
            store: self,
        }
    }

    fn merge_all<F>(&mut self, mut f: F)
    where
        F: FnMut(usize, <Self as VectorMerge>::Item),
        Self: Sized,
    {
        let mut it = self.merge();
        while let Some((id, e)) = it.next() {
            f(id, e);
        }
    }

    fn merge_until<F>(&mut self, mut f: F)
    where
        F: FnMut(usize, <Self as VectorMerge>::Item) -> bool,
        Self: Sized,
    {
        let mut it = self.merge();
        while let Some((id, e)) = it.next() {
            if !f(id, e) {
                break;
            }
        }
    }
}
impl<T: ?Sized> VectorMergeExt for T where T: VectorMerge {}

/// Iterate over the non-zero (non-mutable) elements of a vector
pub struct VectorMergeIter<'a, V>
where
    V: 'a + VectorMerge,
{
    idx: Option<usize>,
    store: &'a mut V,
}

impl<'a, V> VectorMergeIter<'a, V>
where
    V: 'a + VectorMerge,
{
    #[cfg_attr(feature = "cargo-clippy", allow(should_implement_trait))]
    pub fn next(&mut self) -> Option<(usize, V::Item)> {
        self.idx = self.store.lower_bound_index(self.idx.map_or(0, |idx| idx + 1));
        self.idx.map(|idx| (idx, self.store.get_unchecked(idx)))
    }
}
