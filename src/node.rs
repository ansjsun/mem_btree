use std::sync::Arc;

use crate::*;

#[derive(Debug)]
pub struct Node<K, V>
where
    K: Ord + Debug + Clone,
    V: Debug,
{
    pub key: Option<Arc<K>>,
    children: Vec<N<K, V>>,
}

impl<K, V> Node<K, V>
where
    K: Ord + Debug + Clone,
    V: Debug,
{
    pub fn new(children: Vec<N<K, V>>) -> N<K, V>
    where
        K: Ord + Debug + Clone,
        V: Debug,
    {
        let key = if children.len() == 0 {
            None
        } else {
            children[0].key().cloned().map(|v| Arc::new(v))
        };
        Arc::new(BTreeType::Node(Self { key, children }))
    }

    pub fn put(&self, m: usize, k: K, v: V) -> (Vec<N<K, V>>, Option<Item<K, V>>)
    where
        K: Ord,
    {
        let index = self.search_index(&k);

        let (values, old) = self.children[index].put(m, k, v);

        let mut children = Vec::with_capacity(self.children.len() + values.len());

        children.extend(self.children[..index].iter().cloned());
        children.extend(values);
        children.extend(self.children[index + 1..].iter().cloned());

        if children.len() < m {
            return (vec![Self::new(children)], old);
        }

        let mid = m / 2;

        let left = children[..mid].to_vec();
        let right = children[mid..].to_vec();

        return (vec![Self::new(left), Self::new(right)], old);
    }

    pub fn get(&self, k: &K) -> Option<&V> {
        self.children[self.search_index(k)].get(k)
    }

    fn search_index(&self, k: &K) -> usize {
        match self.children.binary_search_by(|c| cmp(c.key(), Some(&k))) {
            Ok(i) => i,
            Err(i) => {
                if i == 0 {
                    i
                } else {
                    i - 1
                }
            }
        }
    }

    pub fn split_off(&mut self, _k: &K) -> (N<K, V>, N<K, V>) {
        // match self.children.binary_search_by(|c| cmp(c.key(), Some(&k))) {
        //     Ok(i) => {
        //         let mut left = self.children[..i].to_vec();
        //         let mut right = self.children[i..].to_vec();

        //         return (Self::new(left), Self::new(right));
        //     }
        //     Err(_) => todo!(),
        // }

        todo!()
    }
}
