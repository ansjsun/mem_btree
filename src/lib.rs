mod leaf;
mod node;

use std::{collections::LinkedList, fmt::Debug, sync::Arc};

use leaf::Leaf;
use node::Node;

type N<K, V> = Arc<BTreeType<K, V>>;

type Item<K, V> = Arc<(K, V)>;

#[derive(Debug)]
pub enum BTreeType<K, V>
where
    K: Ord + Debug + Clone,
    V: Debug,
{
    Leaf(Leaf<K, V>),
    Node(Node<K, V>),
}

impl<K, V> BTreeType<K, V>
where
    K: Ord + Debug + Clone,
    V: Debug,
{
    fn key(&self) -> Option<&K> {
        match self {
            BTreeType::Leaf(l) => Some(&l.items[0].0),
            BTreeType::Node(n) => match n.key.as_ref() {
                Some(k) => Some(&*k),
                None => None,
            },
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
}

pub struct Iterator<K, V>
where
    K: Ord + Debug + Clone,
    V: Debug,
{
    inner: BTree<K, V>,
    queue: LinkedList<(N<K, V>, usize)>,
    direction: bool, // true front false back
}

impl<K, V> Iterator<K, V>
where
    K: Ord + Debug + Clone,
    V: Debug,
{
    fn new(inner: BTree<K, V>, direction: bool) -> Self {
        let mut queue = LinkedList::new();
        queue.push_back((inner.root.clone(), 0));

        Self {
            inner,
            queue,
            direction,
        }
    }
    pub fn next(&'a mut self) -> Option<(&'a K, &'a V)> {
        todo!()
    }

    pub fn prev(&'a mut self) -> Option<(&'a K, &'a V)> {
        todo!()
    }

    pub fn item(&self) -> Option<&Item<K, V>> {
        todo!()
    }
    pub fn seek(&mut self, key: &K) {
        todo!()
    }
}

#[derive(Clone, Debug)]
pub struct BTree<K, V>
where
    K: Ord + Debug + Clone,
    V: Debug,
{
    m: usize,
    length: usize,
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
            length: Default::default(),
            root: Arc::new(BTreeType::Leaf(Leaf { items: Vec::new() })),
        }
    }

    pub fn put(&mut self, k: K, v: V) -> Option<Item<K, V>> {
        let (values, v) = self.root.put(self.m, k, v);

        if v.is_none() {
            self.length += 1;
        }

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

        self.length -= 1;

        Some(item)
    }

    pub fn split_off(&mut self, k: &K) -> BTree<K, V> {
        let (left, right) = self.root.split_off(k);
        self.root = left;
        self.length = self.root.len();

        BTree {
            m: self.m,
            length: right.len(),
            root: right,
        }
    }

    pub fn get(&self, k: &K) -> Option<&V> {
        self.root.get(k)
    }

    pub fn len(&self) -> usize {
        self.length
    }
}

fn cmp<K>(k1: Option<&K>, k2: Option<&K>) -> std::cmp::Ordering
where
    K: Ord + Debug + Clone,
{
    match (k1, k2) {
        (Some(k1), Some(k2)) => k1.cmp(k2),
        (Some(_), None) => std::cmp::Ordering::Greater,
        (None, Some(_)) => std::cmp::Ordering::Less,
        (None, None) => std::cmp::Ordering::Equal,
    }
}
