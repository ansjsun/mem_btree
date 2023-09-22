//! # A Data Structure of BTree Implemented with Rust, support snapshot. not use any unsafe lib.
//!
//! ## Features
//!
//! * snapshot ✅
//! * split_off ✅
//! * put ✅
//! * delete ✅
//! * get ✅
//! * seek ✅
//! * seek_prev ✅
//! * prev iter ✅
//! * next iter ✅
//! * batch_write ✅
//!
//! Licensed under either of
//! * Apache License, Version 2.0,
//! (./LICENSE-APACHE or <http://www.apache.org/licenses/LICENSE-2.0>)
//! * MIT license (./LICENSE-MIT or <http://opensource.org/licenses/MIT>)
//! at your option.
//!
//! ## Examples
//!
//! All examples are in the [sub-repository](https://github.com/async-graphql/examples), located in the examples directory.
//!
//! **Run an example:**
//!
//! ```shell script
//! cd test
//! cargo run --package mem_btree --example example
//! ```
//!

mod batch_write;
mod leaf;
mod node;

use std::{
    collections::{BTreeMap, LinkedList},
    fmt::Debug,
    sync::Arc,
};

use batch_write::Action;
use leaf::Leaf;
use node::Node;

type N<K, V> = Arc<BTreeType<K, V>>;

pub type Item<K, V> = Arc<(K, V)>;

pub type BatchWrite<K, V> = batch_write::BatchWrite<K, V>;

pub type PutResult<K, V> = (Vec<N<K, V>>, Option<Item<K, V>>);

#[derive(Debug)]
pub enum BTreeType<K, V> {
    Leaf(Leaf<K, V>),
    Node(Node<K, V>),
}

impl<K, V> BTreeType<K, V>
where
    K: Ord,
{
    /// return min key for this node
    fn key(&self) -> Option<&Item<K, V>> {
        match self {
            BTreeType::Leaf(l) => Some(&l.items[0]),
            BTreeType::Node(n) => n.key.as_ref(),
        }
    }

    /// return max key for this node
    fn max(&self) -> Option<&Item<K, V>> {
        match self {
            BTreeType::Leaf(l) => l.items.last(),
            BTreeType::Node(n) => n.children.last()?.max(),
        }
    }

    fn put(&self, m: usize, k: K, v: V) -> PutResult<K, V> {
        match self {
            BTreeType::Leaf(leaf) => leaf.put(m, k, v),
            BTreeType::Node(node) => node.put(m, k, v),
        }
    }

    fn get(&self, k: &K) -> Option<&V> {
        match self {
            BTreeType::Leaf(leaf) => leaf.get(k),
            BTreeType::Node(node) => node.get(k),
        }
    }

    fn remove(&self, k: &K) -> Option<(N<K, V>, Item<K, V>)> {
        match self {
            BTreeType::Leaf(leaf) => leaf.remove(k),
            BTreeType::Node(node) => node.remove(k),
        }
    }

    fn write(&self, m: usize, batch_write: BTreeMap<K, Action<V>>) -> Vec<N<K, V>> {
        match self {
            BTreeType::Leaf(leaf) => leaf.write(m, batch_write),
            BTreeType::Node(node) => node.write(m, batch_write),
        }
    }

    fn split_off(&self, k: &K) -> (N<K, V>, N<K, V>) {
        match self {
            BTreeType::Leaf(leaf) => leaf.split_off(k),
            BTreeType::Node(node) => node.split_off(k),
        }
    }

    pub fn len(&self) -> usize {
        match self {
            BTreeType::Leaf(leaf) => leaf.len(),
            BTreeType::Node(node) => node.len(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn children_len(&self) -> usize {
        match self {
            BTreeType::Leaf(leaf) => leaf.items.len(),
            BTreeType::Node(node) => node.children.len(),
        }
    }

    fn get_node_by_index(&self, index: usize) -> N<K, V> {
        if let BTreeType::Node(node) = self {
            node.children[index].clone()
        } else {
            panic!("not a node")
        }
    }
}

pub struct Iterator<K, V>
where
    K: Ord + Debug,
    V: Debug,
{
    inner: BTree<K, V>,
    stack: LinkedList<(N<K, V>, i32)>,
}

impl<K, V> Iterator<K, V>
where
    K: Ord + Debug,
    V: Debug,
{
    fn new(inner: BTree<K, V>) -> Self {
        let mut stack = LinkedList::new();
        stack.push_back((inner.root.clone(), -1));
        Self { inner, stack }
    }

    /// # Example
    /// ```rust
    /// use mem_btree::BTree;
    /// let mut btree = BTree::new(32);
    /// let datas = vec![1,2,3,4,5];
    /// for i in datas.iter() {
    ///   btree.put(i.clone(), i.clone());
    /// }
    /// let mut iter = btree.iter();
    /// while let Some(item) = iter.next() {
    ///  println!("{:?}", item);
    /// }
    /// ```
    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> Option<Item<K, V>> {
        loop {
            let (b, mut index) = self.stack.pop_back()?;
            index += 1;
            if index == b.children_len() as i32 {
                continue;
            }
            self.stack.push_back((b.clone(), index));

            match &*b {
                BTreeType::Leaf(l) => {
                    let result = Some(l.items[index as usize].clone());
                    return result;
                }
                BTreeType::Node(n) => {
                    self.stack
                        .push_back((n.children[index as usize].clone(), -1));
                }
            }
        }
    }

    /// # Example
    /// ```rust
    /// use mem_btree::BTree;
    /// let mut btree = BTree::new(32);
    /// let datas = vec![1,2,3,4,5];
    /// for i in datas.iter() {
    ///  btree.put(i.clone(), i.clone());
    /// }
    /// let mut iter = btree.iter();
    /// while let Some(item) = iter.prev() {
    /// println!("{:?}", item);
    /// }    
    pub fn prev(&mut self) -> Option<Item<K, V>> {
        loop {
            let (b, mut index) = self.stack.pop_back()?;
            if index == -1 {
                index = b.children_len() as i32;
            }

            index -= 1;
            if index < 0 {
                continue;
            }
            self.stack.push_back((b.clone(), index));

            match &*b {
                BTreeType::Leaf(l) => {
                    let result = Some(l.items[index as usize].clone());
                    return result;
                }
                BTreeType::Node(n) => {
                    self.stack
                        .push_back((n.children[index as usize].clone(), -1));
                }
            }
        }
    }

    /// clear stack and push root node
    /// it same as new Iterator
    pub fn reset(&mut self) {
        self.stack.clear();
        self.stack.push_back((self.inner.root.clone(), -1));
    }

    /// seek by the key key
    /// # Example
    /// ```rust
    /// use mem_btree::BTree;
    /// let mut btree = BTree::new(32);
    /// let datas = vec![1,2,3,4,5];
    /// for i in datas.iter() {
    ///     btree.put(i.clone(), i.clone());
    /// }
    /// let mut iter = btree.iter();
    /// iter.seek(&3);
    /// assert_eq!(iter.next(), Some(std::sync::Arc::new((3, 3))));
    /// assert_eq!(iter.next(), Some(std::sync::Arc::new((4, 4))));
    /// assert_eq!(iter.next(), Some(std::sync::Arc::new((5, 5))));
    ///
    /// ```
    pub fn seek(&mut self, key: &K) {
        self.stack.clear();

        let mut node = self.inner.root.clone();
        loop {
            match &*node {
                BTreeType::Leaf(l) => {
                    let index = l.search_index(key).unwrap_or_else(|i| i);
                    self.stack.push_back((node.clone(), index as i32 - 1));
                    break;
                }
                BTreeType::Node(n) => {
                    let index = n.search_index(key);
                    self.stack.push_back((node.clone(), index as i32));
                    node = node.get_node_by_index(index);
                }
            }
        }
    }
    /// seek prev by the key
    /// # Example
    /// ```rust
    /// use mem_btree::BTree;
    /// let mut btree = BTree::new(32);
    /// let datas = vec![1,2,3,4,5];
    /// for i in datas.iter() {
    ///     btree.put(i.clone(), i.clone());
    /// }
    /// let mut iter = btree.iter();
    /// iter.seek_prev(&3);
    /// assert_eq!(iter.prev(), Some(std::sync::Arc::new((3, 3))));
    /// assert_eq!(iter.prev(), Some(std::sync::Arc::new((2, 2))));
    /// assert_eq!(iter.prev(), Some(std::sync::Arc::new((1, 1))));
    ///
    /// ```
    pub fn seek_prev(&mut self, key: &K) {
        self.stack.clear();

        let mut node = self.inner.root.clone();
        loop {
            match &*node {
                BTreeType::Leaf(l) => {
                    match l.search_index(key) {
                        Ok(index) => self.stack.push_back((node.clone(), index as i32 + 1)),
                        Err(index) => self.stack.push_back((node.clone(), index as i32)),
                    }

                    break;
                }
                BTreeType::Node(n) => {
                    let index = n.search_index(key);
                    self.stack.push_back((node.clone(), index as i32));
                    node = node.get_node_by_index(index);
                }
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct BTree<K, V> {
    m: usize,
    root: N<K, V>,
}

impl<K, V> BTree<K, V>
where
    K: Ord + Debug,
    V: Debug,
{
    /// Create a new BTree with a given branching factor
    /// The branching factor is the maximum number of children a node can have
    /// The branching factor must be at least 2
    /// # Examples
    /// ```rust
    /// let mut btree = mem_btree::BTree::new(4);
    /// let datas = vec![1,2,3,4,5] ;
    /// for i in datas.iter() {
    ///    btree.put(i.clone(), i.clone());
    /// }
    /// println!("{:?}", btree.len());
    /// for i in datas.iter() {
    ///   btree.remove(i);
    /// }
    /// println!("{:?}", btree.len());
    /// ```
    ///

    // println!("{:?}", btree);
    pub fn new(m: usize) -> Self {
        Self {
            m,
            root: Arc::new(BTreeType::Leaf(Leaf { items: Vec::new() })),
        }
    }

    /// Insert a key-value pair into the B-tree
    /// If the key already exists, the old value is returned
    /// If the key does not exist, None is returned
    pub fn put(&mut self, k: K, v: V) -> Option<Item<K, V>> {
        let (values, v) = self.root.put(self.m, k, v);

        if values.len() > 1 {
            self.root = Node::instance(values);
        } else {
            self.root = values[0].clone();
        }

        v
    }

    /// Remove a key-value pair from the B-tree
    /// If the key exists, the old value is returned
    /// If the key does not exist, None is returned
    pub fn remove(&mut self, k: &K) -> Option<Item<K, V>> {
        let (node, item) = self.root.remove(k)?;

        self.root = node;

        Some(item)
    }

    /// Write a batch of key-value pairs into the B-tree
    ///
    /// # Examples
    /// ```
    /// use mem_btree::BTree;
    /// use mem_btree::BatchWrite;
    /// let mut btree = BTree::new(32);
    /// let mut bw = BatchWrite::new();
    /// bw.put(1, 1);
    /// bw.put(2, 2);
    /// bw.put(3, 3);
    /// btree.write(bw);
    /// ```
    ///
    pub fn write(&mut self, batch_write: BatchWrite<K, V>) {
        let mut nodes = self.root.write(self.m, batch_write.into_map());

        while nodes.len() > self.m {
            nodes = nodes
                .chunks(self.m)
                .filter_map(|c| {
                    if c.is_empty() {
                        None
                    } else {
                        Some(Node::instance(c.to_vec()))
                    }
                })
                .collect();
        }

        if nodes.len() > 1 {
            self.root = Node::instance(nodes);
        } else {
            match nodes.into_iter().next() {
                Some(v) => self.root = v,
                None => self.root = Node::instance(vec![]),
            };
        }
    }

    /// Split off a part of the B-tree
    /// The key k is the minimum key in the new B-tree
    /// The new B-tree contains all keys greater than or equal to k
    /// The old B-tree contains all keys less than k
    /// # Examples
    /// ```rust
    /// use mem_btree::BTree;
    /// let mut btree = BTree::new(32);
    /// let datas = vec![1,2,3,4,5];
    /// for i in datas.iter() {
    ///  btree.put(i.clone(), i.clone());
    /// }
    /// let right = btree.split_off(&3);
    /// assert_eq!(btree.len(), 2); // 1,2
    /// assert_eq!(right.len(), 3); // 3,4,5
    /// ```
    ///
    pub fn split_off(&mut self, k: &K) -> BTree<K, V> {
        let (left, right) = self.root.split_off(k);
        self.root = left;

        BTree {
            m: self.m,
            root: right,
        }
    }

    /// Get the value for a given key
    /// If the key exists, the value is returned
    /// If the key does not exist, None is returned
    /// # Examples
    /// ```rust
    /// use mem_btree::BTree;
    /// let mut btree = BTree::new(32);
    /// let datas = vec![1,2,3,4,5];
    /// for i in datas.iter() {
    /// btree.put(i.clone(), i.clone());
    /// }
    /// assert_eq!(btree.get(&1), Some(&1));
    /// assert_eq!(btree.get(&6), None);
    /// ```
    pub fn get(&self, k: &K) -> Option<&V> {
        if self.root.len() == 0 {
            return None;
        }
        self.root.get(k)
    }

    /// Get the number of key-value pairs in the B-tree
    /// # Examples
    /// ```rust
    /// use mem_btree::BTree;
    /// let mut btree = BTree::new(32);
    /// let datas = vec![1,2,3,4,5];
    /// for i in datas.iter() {
    /// btree.put(i.clone(), i.clone());
    /// }
    /// assert_eq!(btree.len(), 5);
    /// ```
    pub fn len(&self) -> usize {
        self.root.len()
    }

    pub fn is_empty(&self) -> bool {
        self.root.is_empty()
    }

    /// make a iterator for this btree
    /// default is seek_first
    pub fn iter(&self) -> Iterator<K, V> {
        Iterator::new(Self {
            m: self.m,
            root: self.root.clone(),
        })
    }

    /// Get the minimum key in the B-tree
    pub fn min(&mut self) -> Option<&Item<K, V>> {
        self.root.key()
    }

    /// Get the maximum key in the B-tree
    pub fn max(&mut self) -> Option<&Item<K, V>> {
        self.root.max()
    }
}

fn cmp<K, V>(k1: Option<&Item<K, V>>, k2: Option<&K>) -> std::cmp::Ordering
where
    K: Ord,
{
    match (k1, k2) {
        (Some(k1), Some(k2)) => k1.0.cmp(k2),
        (Some(_), None) => std::cmp::Ordering::Greater,
        (None, Some(_)) => std::cmp::Ordering::Less,
        (None, None) => std::cmp::Ordering::Equal,
    }
}

#[cfg(test)]
mod tests {
    use crate::BatchWrite;

    use super::BTree;
    use rand::rngs::StdRng;
    use rand::{Rng, SeedableRng};
    use std::collections::BTreeMap;

    #[test]
    fn test_insert_and_compare() {
        // Create a new BTree and BTreeMap
        let mut btree = BTree::new(32);
        let mut btree_map = BTreeMap::new();

        // Generate some random key-value pairs
        let mut rng = StdRng::seed_from_u64(42);
        let mut pairs = Vec::new();
        for _ in 0..10000 {
            let key = rng.gen::<u64>();
            let value = rng.gen::<u64>();
            pairs.push((key, value));
        }

        // Insert the key-value pairs into both data structures
        for (key, value) in &pairs {
            btree.put(*key, *value);
            btree_map.insert(*key, *value);
        }

        // Check if the values are the same in both data structures
        for (key, _value) in &pairs {
            assert_eq!(btree.get(key), btree_map.get(key));
        }
    }

    #[test]
    fn test_remove_and_compare() {
        // Create a new BTree and BTreeMap
        let mut btree = BTree::new(32);
        let mut btree_map = BTreeMap::new();

        // Generate some random key-value pairs
        let mut rng = StdRng::seed_from_u64(42);
        let mut pairs = Vec::new();
        for _ in 0..10000 {
            let key = rng.gen::<u64>();
            let value = rng.gen::<u64>();
            pairs.push((key, value));
        }

        // Insert the key-value pairs into both data structures
        for (key, value) in &pairs {
            btree.put(*key, *value);
            btree_map.insert(*key, *value);
        }

        // Remove some key-value pairs from both data structures
        for i in 0..5000 {
            let (key, _) = pairs[i];
            btree.remove(&key);
            btree_map.remove(&key);
        }

        // Check if the values are the same in both data structures
        for (key, _value) in &pairs {
            assert_eq!(btree.get(key), btree_map.get(key));
        }
    }

    #[test]
    fn test_split_index() {
        // Create a new BTree and BTreeMap
        let mut btree = BTree::new(32);
        let mut btree_map = BTreeMap::new();

        // Generate some random key-value pairs
        let mut rng = StdRng::seed_from_u64(42);
        let mut pairs = Vec::new();
        for _ in 0..1000 {
            let key = rng.gen::<u64>();
            let value = rng.gen::<u64>();
            pairs.push((key, value));
        }

        // Insert the key-value pairs into both data structures
        for (key, value) in &pairs {
            btree.put(*key, *value);
            btree_map.insert(*key, *value);
        }

        let temp = btree.clone();
        let temp_map = btree_map.clone();

        for i in 0..pairs.len() {
            let mut btree = temp.clone();
            let mut btree_map = temp_map.clone();
            // Split off a part of the BTree and BTreeMap
            let split_index = i as u64;
            let split_btree = btree.split_off(&split_index);
            let split_btree_map = btree_map.split_off(&split_index);

            // Check if the split off part is correct
            for (key, _value) in &pairs {
                if *key < split_index {
                    assert_eq!(btree.get(key), btree_map.get(key));
                } else {
                    assert_eq!(split_btree.get(key), split_btree_map.get(key));
                }
            }

            // Check if the split off part is empty in the original BTree and BTreeMap
            for (key, _value) in &pairs {
                if *key >= split_index {
                    assert_eq!(btree.get(key), None);
                    assert_eq!(btree_map.get(key), None);
                }
            }
        }
    }

    #[test]
    fn test_iter() {
        // Create a new BTree and BTreeMap
        let mut btree = BTree::new(32);
        let mut btree_map = BTreeMap::new();

        // Generate some random key-value pairs
        let mut rng = StdRng::seed_from_u64(42);
        let mut pairs = Vec::new();
        for _ in 0..10000 {
            let key = rng.gen::<u64>();
            let value = rng.gen::<u64>();
            pairs.push((key, value));
        }

        // Insert the key-value pairs into both data structures
        for (key, value) in &pairs {
            btree.put(*key, *value);
            btree_map.insert(*key, *value);
        }

        // Check if the values are the same in both data structures
        let mut btree_iter = btree.iter();
        let mut btree_map_iter = btree_map.iter();
        loop {
            match (btree_iter.next(), btree_map_iter.next()) {
                (Some(item), Some((btree_map_key, btree_map_value))) => {
                    assert_eq!(&item.0, btree_map_key);
                    assert_eq!(&item.1, btree_map_value);
                }
                (None, None) => break,
                _ => panic!("BTree and BTreeMap have different lengths"),
            }
        }
    }

    #[test]
    fn test_iter_prev() {
        // Create a new BTree and BTreeMap
        let mut btree = BTree::new(32);
        let mut btree_map = BTreeMap::new();

        // Generate some random key-value pairs
        let mut rng = StdRng::seed_from_u64(42);
        let mut pairs = Vec::new();
        for _ in 0..10000 {
            let key = rng.gen::<u64>();
            let value = rng.gen::<u64>();
            pairs.push((key, value));
        }

        // Insert the key-value pairs into both data structures
        for (key, value) in &pairs {
            btree.put(*key, *value);
            btree_map.insert(*key, *value);
        }

        // Check if the values are the same in both data structures
        let mut btree_iter = btree.iter();
        let mut btree_map_iter = btree_map.iter().rev();
        loop {
            match (btree_iter.prev(), btree_map_iter.next()) {
                (Some(item), Some((btree_map_key, btree_map_value))) => {
                    assert_eq!(&item.0, btree_map_key);
                    assert_eq!(&item.1, btree_map_value);
                }
                (None, None) => break,
                _ => panic!("BTree and BTreeMap have different lengths"),
            }
        }
    }

    #[test]
    fn test_seek() {
        // Create a new BTree and BTreeMap
        let mut btree = BTree::new(32);
        let mut btree_map = BTreeMap::new();

        // Generate some random key-value pairs
        let mut rng = StdRng::seed_from_u64(42);
        let mut pairs = Vec::new();
        for _ in 0..10000 {
            let key = rng.gen::<u64>();
            let value = rng.gen::<u64>();
            pairs.push((key, value));
        }

        // Insert the key-value pairs into both data structures
        for (key, value) in &pairs {
            btree.put(*key, *value);
            btree_map.insert(*key, *value);
        }

        // Check if the values are the same in both data structures
        for i in 0..10000 {
            let key = i as u64;
            let mut btree_iter = btree.iter();
            btree_iter.seek(&key);
            let btree_map_iter = btree_map.range(key..).next();
            match (btree_iter.next(), btree_map_iter) {
                (Some(item), Some((btree_map_key, btree_map_value))) => {
                    assert_eq!(&item.0, btree_map_key);
                    assert_eq!(&item.1, btree_map_value);
                }
                (None, None) => {}
                _ => panic!("BTree and BTreeMap have different lengths"),
            }
        }
    }

    #[test]
    fn test_seek_prev() {
        // Create a new BTree and BTreeMap
        let mut btree = BTree::new(32);
        let mut btree_map = BTreeMap::new();

        // Generate some random key-value pairs
        let mut rng = StdRng::seed_from_u64(42);
        let mut pairs = Vec::new();
        for _ in 0..10000 {
            let key = rng.gen::<u64>();
            let value = rng.gen::<u64>();
            pairs.push((key, value));
        }

        // Insert the key-value pairs into both data structures
        for (key, value) in &pairs {
            btree.put(*key, *value);
            btree_map.insert(*key, *value);
        }

        // Check if the values are the same in both data structures
        for i in 0..10000 {
            let key = i as u64;
            let mut btree_iter = btree.iter();
            btree_iter.seek_prev(&key);
            let btree_map_iter = btree_map.range(..=key).next_back();
            match (btree_iter.prev(), btree_map_iter) {
                (Some(item), Some((btree_map_key, btree_map_value))) => {
                    assert_eq!(&item.0, btree_map_key);
                    assert_eq!(&item.1, btree_map_value);
                }
                (None, None) => {}
                _ => panic!("BTree and BTreeMap have different lengths"),
            }
        }
    }

    #[test]
    fn test_batch_write() {
        // Create a new BTree and BTreeMap
        let mut btree = BTree::new(32);
        let mut btree_map = BTreeMap::new();

        // Generate some random key-value pairs
        let mut rng = StdRng::seed_from_u64(42);
        let mut pairs = Vec::new();
        for _ in 0..10240 {
            let key = rng.gen::<u64>();
            let value = rng.gen::<u64>();
            pairs.push((key, value));
        }

        pairs.chunks(256).for_each(|c| {
            let mut bw = BatchWrite::default();
            for v in c {
                bw.put(v.0, v.1);
            }
            btree.write(bw);
        });

        // Insert the key-value pairs into both data structures
        for (key, value) in &pairs {
            btree_map.insert(*key, *value);
        }

        // Check if the values are the same in both data structures
        for (key, _value) in &pairs {
            assert_eq!(btree.get(key), btree_map.get(key));
        }
    }

    #[test]
    fn test_max() {
        let mut btree = BTree::new(4);
        btree.put(1, "a");
        btree.put(2, "b");
        btree.put(3, "c");
        btree.put(4, "d");
        btree.put(5, "e");
        assert_eq!(btree.max(), Some(&std::sync::Arc::new((5, "e"))));

        assert_eq!(btree.min(), Some(&std::sync::Arc::new((1, "a"))));
    }
}
