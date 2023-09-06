use crate::*;

#[derive(Debug)]
pub struct Leaf<K: Debug, V: Debug> {
    pub items: Vec<Item<K, V>>,
}

impl<K, V> Leaf<K, V>
where
    K: Ord + Debug + Clone,
    V: Debug,
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

    pub fn split_off(&self, _k: &K) -> (N<K, V>, N<K, V>) {
        todo!()
    }
}
