use std::sync::Arc;

use crate::*;

#[derive(Debug)]
pub struct Node<K, V>
where
    K: Ord + Debug + Clone,
    V: Debug,
{
    pub key: Option<Arc<K>>,
    length: usize,
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
        let length = children.iter().map(|v| v.len()).sum();
        Arc::new(BTreeType::Node(Self {
            key,
            length,
            children,
        }))
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

    pub fn remove(&self, k: &K) -> Option<(N<K, V>, Item<K, V>)> {
        let index = self.search_index(&k);

        let (child, item) = self.children[index].remove(k)?;

        let mut children = Vec::with_capacity(self.children.len() - 1);
        children.extend(self.children[..index].iter().cloned());
        if child.len() > 0 {
            children.push(child);
        }
        children.extend(self.children[index + 1..].iter().cloned());

        Some((Self::new(children), item))
    }

    pub fn len(&self) -> usize {
        self.length
    }

    pub fn split_off(&self, k: &K) -> (N<K, V>, N<K, V>) {
        let aaa = match self.children.binary_search_by(|c| cmp(c.key(), Some(k))) {
            Ok(index) => {
                println!("ok index: {}", index);
                let (left, right) = self.children.split_at(index);
                (Self::new(left.to_vec()), Self::new(right.to_vec()))
            }
            Err(index) => {
                println!("err index: {}----{}", index, self.children.len());
                if index == 0 {
                    (Self::new(Vec::new()), Self::new(self.children.clone()))
                } else if index == self.children.len() {
                    (Self::new(self.children.clone()), Self::new(Vec::new()))
                } else {
                    self.children[index].split_off(k)
                }
            }
        };

        println!(
            "{:?}++++++=xxx=======: {:#?}================right:{:?}",
            k, aaa.0, aaa.1
        );
        aaa
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
}
