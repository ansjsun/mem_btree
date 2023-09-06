# rust mem_btree 

A Data Structure of BTree Implemented with Rust, not use any unsafe lib.


## Design:
xxxxxxxx

## future:
* snapshot 
* split_off
* put
* delete
* get
* prev iter
* next iter


## bench
5k kv insert
````
btree insert 120.064954ms
btreemap insert 73.882981ms
btreemap_arc insert 79.869725ms
btree get 102.721024ms
btreemap get 100.939223ms
btreemap_arc get 100.255662ms
btree clone 759ns
btreemap clone 453.955778ms
btreemap_arc clone 24.776548ms
````