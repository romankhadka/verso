use lru::LruCache;
use std::num::NonZeroUsize;

use super::page::Page;

type Key = (u32, u16, String); // (spine_idx, column_width, theme)

pub struct PageCache { inner: LruCache<Key, Vec<Page>> }

impl PageCache {
    pub fn new(cap: usize) -> Self {
        Self { inner: LruCache::new(NonZeroUsize::new(cap.max(1)).unwrap()) }
    }
    pub fn get(&mut self, spine_idx: u32, width: u16, theme: &str) -> Option<&Vec<Page>> {
        self.inner.get(&(spine_idx, width, theme.to_string()))
    }
    pub fn put(&mut self, spine_idx: u32, width: u16, theme: &str, pages: Vec<Page>) {
        self.inner.put((spine_idx, width, theme.to_string()), pages);
    }
}
