use rand::prelude::*;
use std::cell::UnsafeCell;
use std::cmp::Ord;
use std::mem::MaybeUninit;
use std::ptr::NonNull;

const MAX_LEVEL: usize = 24;

struct Node<K, V> {
    forward: [Option<NonNull<Node<K, V>>>; MAX_LEVEL],
    key: K,
    val: V,
}

impl<K, V> Node<K, V> {
    fn new(key: K, val: V) -> Self {
        Self {
            key,
            val,
            forward: [None; MAX_LEVEL],
        }
    }

    fn new_uninit() -> Self {
        unsafe {
            Self {
                key: MaybeUninit::zeroed().assume_init(),
                val: MaybeUninit::zeroed().assume_init(),
                forward: [None; MAX_LEVEL],
            }
        }
    }
}

fn rand_lvl() -> usize {
    let mut level = 1;
    let branching = 2;
    while level < MAX_LEVEL && (random::<usize>() % branching == 0) {
        level += 1;
    }
    level
}

pub struct SkipList<K, V> {
    head: UnsafeCell<Node<K, V>>,
    size: usize,
    level: usize,
}

impl<K: Ord, V> SkipList<K, V> {
    pub fn new() -> Self {
        Self {
            head: UnsafeCell::new(Node::new_uninit()),
            size: 0,
            level: 1,
        }
    }

    pub fn insert(&mut self, key: K, val: V) {
        let mut update: [Option<NonNull<Node<K, V>>>; MAX_LEVEL] = [None; MAX_LEVEL];
        let mut x = NonNull::new(self.head.get_mut());

        for i in (0..self.level).rev() {
            unsafe {
                while let Some(node_ptr) = x.unwrap().as_ref().forward[i] {
                    if node_ptr.as_ref().key < key {
                        x = x.unwrap().as_ref().forward[i];
                    } else {
                        break;
                    }
                }
                update[i] = x;
            }
        }

        x = unsafe { x.unwrap().as_ref().forward[0] };
        if let Some(mut node_ptr) = x {
            let node = unsafe { node_ptr.as_mut() };
            if node.key == key {
                node.val = val;
                return;
            }
        }

        let level = rand_lvl();
        if level > self.level {
            for i in self.level..level {
                update[i] = NonNull::new(self.head.get_mut());
            }
            self.level = level;
        }

        x = NonNull::new(Box::into_raw(Box::new(Node::new(key, val))));

        for i in 0..level {
            unsafe {
                x.unwrap().as_mut().forward[i] = update[i].unwrap().as_ref().forward[i];
                update[i].unwrap().as_mut().forward[i] = x;
            }
        }

        self.size += 1;
    }

    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        unsafe {
            if let Some(mut node_ptr) = self.find_node(key) {
                Some(&mut node_ptr.as_mut().val)
            } else {
                None
            }
        }
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        unsafe {
            if let Some(mut node_ptr) = self.find_node(key) {
                Some(&node_ptr.as_mut().val)
            } else {
                None
            }
        }
    }

    pub fn len(&self) -> usize {
        self.size
    }

    unsafe fn find_node(&self, key: &K) -> Option<NonNull<Node<K, V>>> {
        let mut x = NonNull::new(self.head.get());
        for i in (0..self.level).rev() {
            while let Some(node_ptr) = x.unwrap().as_ref().forward[i] {
                if node_ptr.as_ref().key < *key {
                    x = x.unwrap().as_ref().forward[i];
                } else {
                    break;
                }
            }
        }

        x = x.unwrap().as_ref().forward[0];
        if let Some(mut node_ptr) = x {
            let node = node_ptr.as_mut();
            if node.key == *key {
                return x;
            }
        }

        None
    }
}

impl<K, V> Drop for SkipList<K, V> {
    fn drop(&mut self) {
        let mut x = self.head.get_mut().forward[0];
        while let Some(node_ptr) = x {
            unsafe {
                let t = node_ptr.as_ref().forward[0];
                Box::from_raw(node_ptr.as_ptr());
                x = t;
            }
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

        for i in 0..10 {
            sk.insert(i, i);
        }
        assert_eq!(sk.len(), 10);
        for i in 0..10 {
            let k = i;
            let mut v = i;
            assert_eq!(sk.get_mut(&k), Some(&mut v));
        }

        for i in 0..10 {
            sk.insert(i, i + 1);
        }
        assert_eq!(sk.len(), 10);
        for i in 0..10 {
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
