use rand::seq::SliceRandom;

pub fn main() {
    for k in 0..33 {
        let num = 32;
        let mut datas = (0..num).into_iter().map(|v| v).collect::<Vec<_>>();

        datas.shuffle(&mut rand::thread_rng());

        let mut btree = mem_btree::BTree::new(4);

        for i in datas.iter() {
            btree.put(i.clone(), i.clone());
        }

        let right = btree.split_off(&k);

        println!("k:{} left: {:?} right: {:?}", k, btree.len(), right.len());
    }
}

pub fn remove() {
    let num = 1024 * 1024;
    let mut datas = (0..num).into_iter().map(|v| v).collect::<Vec<_>>();

    datas.shuffle(&mut rand::thread_rng());

    let mut btree = mem_btree::BTree::new(4);

    for i in datas.iter() {
        btree.put(i.clone(), i.clone());
    }

    println!("{:?}", btree.len());

    for i in datas.iter() {
        btree.remove(i);
    }

    println!("{:?}", btree);
}
