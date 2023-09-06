mod leaf;
mod node;

use std::{fmt::Debug, sync::Arc};

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
