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

#[derive(Debug)]
pub enum BTreeType<K, V>
where
    K: Ord + Clone,
{
    Leaf(Leaf<K, V>),
    Node(Node<K, V>),
}

impl<K, V> BTreeType<K, V>
where
    K: Ord + Clone,
{
    fn key(&self) -> Option<&Item<K, V>> {
        match self {
            BTreeType::Leaf(l) => Some(&l.items[0]),
            BTreeType::Node(n) => n.key.as_ref(),
        }
    }

    fn put(&self, m: usize, k: K, v: V) -> (Vec<N<K, V>>, Option<Item<K, V>>) {
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
    K: Ord + Debug + Clone,
    V: Debug,
{
    inner: BTree<K, V>,
    stack: LinkedList<(N<K, V>, i32)>,
}

impl<K, V> Iterator<K, V>
where
    K: Ord + Debug + Clone,
    V: Debug,
{
    fn new(inner: BTree<K, V>) -> Self {
        let mut stack = LinkedList::new();
        stack.push_back((inner.root.clone(), -1));
        Self { inner, stack }
    }
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

    pub fn reset(&mut self) {
        self.stack.clear();
        self.stack.push_back((self.inner.root.clone(), -1));
    }

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
pub struct BTree<K, V>
where
    K: Ord + Clone,
{
    m: usize,
    root: N<K, V>,
}

impl<K, V> BTree<K, V>
where
    K: Ord + Debug + Clone,
    V: Debug,
{
    pub fn new(m: usize) -> Self {
        Self {
            m,
            root: Arc::new(BTreeType::Leaf(Leaf { items: Vec::new() })),
        }
    }

    pub fn put(&mut self, k: K, v: V) -> Option<Item<K, V>> {
        let (values, v) = self.root.put(self.m, k, v);

        if values.len() > 1 {
            self.root = Node::new(values);
        } else {
            self.root = values[0].clone();
        }

        v
    }

    pub fn remove(&mut self, k: &K) -> Option<Item<K, V>> {
        let (node, item) = self.root.remove(k)?;

        self.root = node;

        Some(item)
    }

    pub fn write(&mut self, batch_write: BatchWrite<K, V>) {
        let mut nodes = self.root.write(self.m, batch_write.to_map());

        while nodes.len() > self.m {
            nodes = nodes
                .chunks(self.m)
                .filter_map(|c| {
                    if c.len() > 0 {
                        Some(Node::new(c.to_vec()))
                    } else {
                        None
                    }
                })
                .collect();
        }

        if nodes.len() > 1 {
            self.root = Node::new(nodes);
        } else {
            match nodes.into_iter().next() {
                Some(v) => self.root = v,
                None => self.root = Node::new(vec![]),
            };
        }
    }

    pub fn split_off(&mut self, k: &K) -> BTree<K, V> {
        let (left, right) = self.root.split_off(k);
        self.root = left;

        BTree {
            m: self.m,
            root: right,
        }
    }

    pub fn get(&self, k: &K) -> Option<&V> {
        if self.root.len() == 0 {
            return None;
        }
        self.root.get(k)
    }

    pub fn len(&self) -> usize {
        self.root.len()
    }

    /// make a iterator for this btree
    /// default is seek_first
    pub fn iter(&self) -> Iterator<K, V> {
        Iterator::new(Self {
            m: self.m,
            root: self.root.clone(),
        })
    }
}

fn cmp<K, V>(k1: Option<&Item<K, V>>, k2: Option<&K>) -> std::cmp::Ordering
where
    K: Ord + Clone,
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
            let mut bw = BatchWrite::new();
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
}
