use std::collections::BTreeMap;

use rand::seq::SliceRandom;

pub fn main() {
    seek_next();
}

pub fn prev_seek_next() {
    let num = 32;
    let mut datas = (0..num).into_iter().map(|v| v * 2).collect::<Vec<_>>();

    datas.shuffle(&mut rand::thread_rng());

    let mut btree = mem_btree::BTree::new(4);

    for i in datas.iter() {
        btree.put(i.clone(), i.clone());
    }

    let mut iter = btree.iter();

    iter.seek_prev(&10);

    while let Some(item) = iter.prev() {
        println!("k: {:?} v: {:?}", item.0, item.1);
    }

    println!("-------------------");

    iter.seek_prev(&9);

    while let Some(item) = iter.prev() {
        println!("k: {:?} v: {:?}", item.0, item.1);
    }
}

pub fn seek_next() {
    let num = 32;
    let mut datas = (0..num).into_iter().map(|v| v * 2).collect::<Vec<_>>();

    datas.shuffle(&mut rand::thread_rng());

    let mut btree = mem_btree::BTree::new(4);

    for i in datas.iter() {
        btree.put(i.clone(), i.clone());
    }

    let mut iter = btree.iter();

    iter.seek(&10);

    while let Some(item) = iter.next() {
        println!("k: {:?} v: {:?}", item.0, item.1);
    }

    println!("-------------------");

    iter.seek(&9);

    while let Some(item) = iter.next() {
        println!("k: {:?} v: {:?}", item.0, item.1);
    }
}

pub fn prev_iter() {
    let num = 1024;
    let mut datas = (0..num).into_iter().map(|v| v).collect::<Vec<_>>();

    datas.shuffle(&mut rand::thread_rng());

    let mut btree = mem_btree::BTree::new(32);

    for i in datas.iter() {
        btree.put(i.clone(), i.clone());
    }

    let mut iter = btree.iter();

    while let Some(item) = iter.prev() {
        println!("k: {:?} v: {:?}", item.0, item.1);
    }
}

pub fn next_iter() {
    let num = 1024;
    let mut datas = (0..num).into_iter().map(|v| v).collect::<Vec<_>>();

    datas.shuffle(&mut rand::thread_rng());

    let mut btree = mem_btree::BTree::new(32);

    for i in datas.iter() {
        btree.put(i.clone(), i.clone());
    }

    println!("{:#?}", btree);

    let mut iter = btree.iter();

    while let Some(item) = iter.next() {
        println!("k: {:?} v: {:?}", item.0, item.1);
    }
}

pub fn split_off() {
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
