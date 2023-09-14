use crate::*;

#[derive(Debug)]
pub struct Leaf<K, V> {
    pub items: Vec<Item<K, V>>,
}

impl<K, V> Leaf<K, V>
where
    K: Ord + Clone,
{
    pub fn new(items: Vec<Item<K, V>>) -> N<K, V> {
        Arc::new(BTreeType::Leaf(Self { items }))
    }

    fn sort_insert(items: &mut Vec<Item<K, V>>, mut item: Item<K, V>) -> Option<Item<K, V>>
    where
        K: Ord,
    {
        match items.binary_search_by(|i| i.0.cmp(&item.0)) {
            Ok(i) => {
                std::mem::swap(&mut items[i], &mut item);
                Some(item)
            }
            Err(i) => {
                items.insert(i, item);
                None
            }
        }
    }

    pub fn put(&self, m: usize, k: K, v: V) -> (Vec<N<K, V>>, Option<Item<K, V>>)
    where
        K: Ord,
    {
        let mut item = Arc::new((k, v));

        if self.items.len() < m {
            let mut items: Vec<Arc<(K, V)>> = self.items.clone();
            let old = Self::sort_insert(&mut items, item);
            return (vec![Self::new(items)], old);
        }

        let mid = m / 2;

        let mut left = self.items[..mid].to_vec();
        let mut right = self.items[mid..].to_vec();

        let old = match item.0.cmp(&self.items[mid].0) {
            std::cmp::Ordering::Less => Self::sort_insert(&mut left, item),
            std::cmp::Ordering::Equal => {
                std::mem::swap(&mut right[0], &mut item);
                Some(item)
            }
            std::cmp::Ordering::Greater => Self::sort_insert(&mut right, item),
        };

        return (vec![Self::new(left), Self::new(right)], old);
    }

    pub fn get(&self, k: &K) -> Option<&V> {
        if let Ok(i) = self.items.binary_search_by(|v| v.0.cmp(k)) {
            return Some(&self.items[i].1);
        }
        None
    }

    pub fn search_index(&self, k: &K) -> Result<usize, usize> {
        self.items.binary_search_by(|v| v.0.cmp(k))
    }

    pub fn remove(&self, k: &K) -> Option<(N<K, V>, Item<K, V>)> {
        if let Ok(i) = self.items.binary_search_by(|v| v.0.cmp(k)) {
            let mut items = Vec::with_capacity(self.items.len() - 1);
            items.extend_from_slice(&self.items[..i]);
            items.extend_from_slice(&self.items[i + 1..]);
            return Some((Self::new(items), self.items[i].clone()));
        }
        None
    }

    pub fn write(&self, m: usize, bw: BTreeMap<K, Action<V>>) -> Vec<N<K, V>> {
        let items = Self::merge_sort_arr(
            self.items.len() + bw.len(),
            self.items.iter(),
            bw.into_iter(),
        );

        items
            .chunks(m)
            .filter_map(|c| {
                if c.len() > 0 {
                    Some(Self::new(c.to_vec()))
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn split_off(&self, k: &K) -> (N<K, V>, N<K, V>) {
        let index = self
            .items
            .binary_search_by(|v| v.0.cmp(k))
            .unwrap_or_else(|i| i);

        let (left, right) = self.items.split_at(index);
        (Self::new(left.to_vec()), Self::new(right.to_vec()))
    }

    fn merge_sort_arr(
        new_len: usize,
        mut iter1: std::slice::Iter<'_, Arc<(K, V)>>,
        mut iter2: std::collections::btree_map::IntoIter<K, Action<V>>,
    ) -> Vec<Item<K, V>> {
        let mut result = Vec::with_capacity(new_len);
        let mut v1 = iter1.next().cloned();
        let mut v2 = iter2.next();
        loop {
            match (&v1, &v2) {
                (None, None) => break,
                (None, Some(_)) => match v2 {
                    Some((_, Action::Delete)) => {
                        v2 = iter2.next();
                    }
                    Some((k, Action::Put(v))) => {
                        result.push(Arc::new((k, v)));
                        v2 = iter2.next();
                    }
                    None => unreachable!(),
                },
                (Some(i), None) => {
                    result.push(i.clone());
                    v1 = iter1.next().cloned();
                }
                (Some(i), Some((k, i2))) => match i.0.cmp(k) {
                    std::cmp::Ordering::Less => {
                        result.push(i.clone());
                        v1 = iter1.next().cloned();
                    }
                    std::cmp::Ordering::Equal => match i2 {
                        Action::Delete => {
                            v1 = iter1.next().cloned();
                            v2 = iter2.next();
                        }
                        Action::Put(_) => {
                            let (k, v) = v2.unwrap();
                            result.push(Arc::new((k, v.value())));
                            v1 = iter1.next().cloned();
                            v2 = iter2.next();
                        }
                    },
                    std::cmp::Ordering::Greater => match i2 {
                        Action::Delete => {
                            v2 = iter2.next();
                        }
                        Action::Put(_) => {
                            let (k, v) = v2.unwrap();
                            result.push(Arc::new((k, v.value())));
                            v2 = iter2.next();
                        }
                    },
                },
            }
        }

        result
    }
}
