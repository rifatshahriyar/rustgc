#![allow(dead_code)]
use std::sync::atomic::Ordering;

#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;

mod common;
mod objectmodel;
mod heap;

mod exhaust;
mod mark;
mod trace;
mod mt_trace;
mod gcbench;
mod mt_gcbench;
mod obj_init;

fn main() {
    use heap;
    objectmodel::init();
    let heap_size : usize = 150 << 20;      // x << 20 means x megabytes  220
    let n_gcthreads = 8;
    
    let immix_space_size : usize = (heap_size as f64  * heap::IMMIX_SPACE_RATIO) as usize;
    heap::IMMIX_SPACE_SIZE.store(immix_space_size, Ordering::SeqCst);
    
    let lo_space_size : usize = (heap_size as f64 * heap::LO_SPACE_RATIO) as usize;
    heap::LO_SPACE_SIZE.store(lo_space_size, Ordering::SeqCst);
    
    println!("-------------------------------------------------------------------------------");
    println!("heap is {} bytes (immix: {} bytes, lo: {} bytes) . ", heap_size, immix_space_size, lo_space_size);
           
    heap::gc::GC_THREADS.store(n_gcthreads,Ordering::SeqCst);
    
    println!("number of gc threads are {} ", n_gcthreads);


    
    if cfg!(feature = "exhaust") {
        exhaust::exhaust_alloc();
    } else if cfg!(feature = "initobj") {
        obj_init::alloc_init();
    } else if cfg!(feature = "gcbench") {
        gcbench::start();
    } else if cfg!(feature = "mt-gcbench") {
        mt_gcbench::start();
    } else if cfg!(feature = "mark") {
        mark::alloc_mark();
    } else if cfg!(feature = "trace") {
        trace::alloc_trace();
    } else if cfg!(feature = "mt-trace") {
        mt_trace::alloc_trace();
    }
    else {
        println!("unknown features: build with 'cargo build --release --features \"exhaust\"");
    }

}
