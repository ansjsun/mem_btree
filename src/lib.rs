mod leaf;
mod node;

use std::{collections::LinkedList, fmt::Debug, sync::Arc};

use leaf::Leaf;
use node::Node;

type N<K, V> = Arc<BTreeType<K, V>>;

pub type Item<K, V> = Arc<(K, V)>;

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

    fn is_leaf(&self) -> bool {
        match self {
            BTreeType::Leaf(_) => true,
            BTreeType::Node(_) => false,
        }
    }

    fn get_node_by_index(&self, index: usize) -> N<K, V> {
        if let BTreeType::Node(node) = self {
            node.children[index].clone()
        } else {
            panic!("not a node")
        }
    }

    fn get_leaf_by_index(&self, index: usize) -> Item<K, V> {
        if let BTreeType::Leaf(node) = self {
            node.items[index].clone()
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
    fn new(inner: BTree<K, V>, direction: bool) -> Self {
        Self {
            inner,
            stack: LinkedList::new(),
            direction,
        }
    }
    pub fn next<'a>(&'a mut self) -> Option<&Item<K, V>> {
        let (n, index) = self.stack.back()?;

        match &**n {
            BTreeType::Leaf(l) => {
                index += 1;
                if index < l.len() {
                    return Some(&l.items[index as usize]);
                }
                self.stack.pop_back();
            }
            BTreeType::Node(_) => todo!(),
        }
    }

    pub fn prev<'a>(&'a mut self) -> Option<&Item<K, V>> {
        todo!()
    }

    pub fn seek(&mut self, key: &K) {
        todo!()
    }

    pub fn seek_first(&mut self) {
        self.stack.clear();

        let mut node = self.inner.root.clone();

        loop {
            match &*node {
                BTreeType::Leaf(l) => {
                    self.stack.push_back((node.clone(), -1));
                    break;
                }
                BTreeType::Node(n) => {
                    self.stack.push_back((node.clone(), 0));
                    node = n.children[0].clone();
                }
            }
        }
    }

    pub fn seek_last(&mut self) {
        self.stack.clear();

        let mut node = self.inner.root.clone();

        loop {
            self.stack.push_back((node.clone(), node.len() as i32));
            match &*node {
                BTreeType::Leaf(_l) => {
                    break;
                }
                BTreeType::Node(n) => {
                    node = n.children[0].clone();
                }
            }
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
