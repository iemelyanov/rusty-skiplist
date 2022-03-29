use rand::prelude::*;
use std::alloc::{alloc, dealloc, Layout};
use std::cmp::Ord;
use std::mem;
use std::ops::Index;
use std::ops::IndexMut;
use std::ptr;

const MAX_LEVEL: usize = 20;

struct Tower<K, V> {
    forward: [*mut Node<K, V>; 0],
}

impl<K, V> Index<usize> for Tower<K, V> {
    type Output = *mut Node<K, V>;

    fn index(&self, index: usize) -> &*mut Node<K, V> {
        unsafe { self.forward.get_unchecked(index) }
    }
}

impl<K, V> IndexMut<usize> for Tower<K, V> {
    fn index_mut(&mut self, index: usize) -> &mut *mut Node<K, V> {
        unsafe { self.forward.get_unchecked_mut(index) }
    }
}

#[repr(C)]
pub struct Node<K, V> {
    key: K,
    val: V,
    layout: Layout,
    tower: Tower<K, V>,
}

impl<K, V> Node<K, V> {
    pub fn alloc(height: usize) -> *mut Node<K, V> {
        let size = mem::size_of::<K>()
            + mem::size_of::<V>()
            + mem::size_of::<Layout>()
            + height * mem::size_of::<*mut Node<K, V>>();
        match Layout::from_size_align(size, 16) {
            Ok(layout) => unsafe {
                let ptr = alloc(layout) as *mut Node<K, V>;
                if ptr.is_null() {
                    return std::ptr::null_mut();
                }
                (*ptr).layout = layout;
                for i in 0..height {
                    (*ptr).tower[i] = ptr::null_mut();
                }
                ptr
            },
            Err(why) => panic!("{}", why),
        }
    }

    pub fn new(key: K, val: V, height: usize) -> *mut Node<K, V> {
        let ptr = Node::alloc(height);
        if ptr.is_null() {
            return ptr::null_mut();
        }
        unsafe {
            (*ptr).key = key;
            (*ptr).val = val;
        }
        ptr
    }

    pub fn new_uninit(height: usize) -> *mut Node<K, V> {
        let ptr = Node::alloc(height);
        if ptr.is_null() {
            return ptr::null_mut();
        }
        ptr
    }
}

fn rand_lvl() -> usize {
    let mut level = 1;
    while level < MAX_LEVEL && (random::<usize>() % 2 == 0) {
        level += 1;
    }
    level
}

pub struct SkipList<K, V> {
    head: *mut Node<K, V>,
    size: usize,
    level: usize,
}

impl<K: Ord, V> SkipList<K, V> {
    pub fn new() -> Self {
        Self {
            head: Node::new_uninit(MAX_LEVEL),
            size: 0,
            level: 1,
        }
    }

    pub fn insert(&mut self, key: K, val: V) {
        let mut update: [*mut Node<K, V>; MAX_LEVEL] = [ptr::null_mut(); MAX_LEVEL];

        unsafe {
            let node_ptr = self.find_gt_or_eq_node(&key, &mut update);
            if !node_ptr.is_null() && (*node_ptr).key == key {
                (*node_ptr).val = val;
                return;
            }
        }

        let level = rand_lvl();
        if level > self.level {
            for i in self.level..level {
                update[i] = self.head;
            }
            self.level = level;
        }

        let x = Node::new(key, val, level);

        for i in 0..level {
            unsafe {
                (*x).tower[i] = (*update[i]).tower[i];
                (*update[i]).tower[i] = x;
            }
        }

        self.size += 1;
    }

    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        let mut update: [*mut Node<K, V>; MAX_LEVEL] = [ptr::null_mut(); MAX_LEVEL];
        unsafe {
            let node_ptr = self.find_gt_or_eq_node(key, &mut update);
            if !node_ptr.is_null() && (*node_ptr).key == *key {
                return Some(&mut (*node_ptr).val);
            }
        }
        None
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        let mut update: [*mut Node<K, V>; MAX_LEVEL] = [ptr::null_mut(); MAX_LEVEL];
        unsafe {
            let node_ptr = self.find_gt_or_eq_node(key, &mut update);
            if !node_ptr.is_null() && (*node_ptr).key == *key {
                return Some(&(*node_ptr).val);
            }
            None
        }
    }

    pub fn len(&self) -> usize {
        self.size
    }

    unsafe fn find_gt_or_eq_node(
        &self,
        key: &K,
        update: &mut [*mut Node<K, V>; MAX_LEVEL],
    ) -> *mut Node<K, V> {
        let mut x = self.head;
        for i in (0..self.level).rev() {
            loop {
                let node_ptr = (*x).tower[i];
                if node_ptr.is_null() {
                    break;
                }
                if (*node_ptr).key < *key {
                    x = (*x).tower[i];
                } else {
                    break;
                }
            }
            update[i] = x;
        }

        return (*x).tower[0];
    }
}

impl<K, V> Drop for SkipList<K, V> {
    fn drop(&mut self) {
        unsafe {
            let mut x = (*self.head).tower[0];
            while !x.is_null() {
                let t = (*x).tower[0];
                dealloc(x as *mut u8, (*x).layout);
                x = t;
            }
            dealloc(self.head as *mut u8, (*self.head).layout);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::SkipList;
    #[test]
    fn it_works() {
        let mut sk = SkipList::new();
        assert_eq!(sk.len(), 0);

        for i in 0..100 {
            sk.insert(i, i);
        }
        assert_eq!(sk.len(), 100);
        for i in 0..100 {
            let k = i;
            let mut v = i;
            assert_eq!(sk.get_mut(&k), Some(&mut v));
        }

        for i in 0..100 {
            sk.insert(i, i + 1);
        }
        assert_eq!(sk.len(), 100);
        for i in 0..100 {
            let k = i;
            let v = i + 1;
            assert_eq!(sk.get(&k), Some(&v));
        }

        for i in 0..20 {
            sk.insert(i, i + 1);
        }
        assert_eq!(sk.len(), 20);
        for i in 0..20 {
            let k = i;
            let v = i + 1;
            assert_eq!(sk.get(&k), Some(&v));
        }
    }
}
