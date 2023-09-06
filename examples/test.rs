use rand::seq::SliceRandom;

pub fn main() {
    let num = 1024 * 1024;
    let mut datas = (100..num).into_iter().map(|v| v * 2).collect::<Vec<_>>();

    datas.shuffle(&mut rand::thread_rng());

    let mut btree = mem_btree::BTree::new(32);

    let now = std::time::Instant::now();

    for i in datas.iter() {
        btree.put(i.clone(), i.clone());
    }

    for i in datas.iter() {
        assert_eq!(btree.get(i), Some(i));
    }

    for i in 0..100 {
        assert_eq!(btree.get(&i), None);
    }

    println!("btree test usetime {:?}", now.elapsed());
}
