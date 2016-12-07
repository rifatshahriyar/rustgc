#![allow(dead_code)]
use std::sync::atomic::Ordering;

#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
extern crate simple_logger;
extern crate libc;

pub mod common;
pub mod objectmodel;
pub mod heap;

pub use heap::immix::ImmixMutatorLocal as Mutator;
use common::ObjectReference;
use std::sync::Arc;
use std::sync::RwLock;
use heap::immix::ImmixSpace;
use heap::immix::ImmixMutatorLocal;
use heap::freelist;
use heap::freelist::FreeListSpace;
use std::boxed::Box;

use heap::immix::myHashMap;
use heap::immix::BYTES_IN_BLOCK;
use heap::immix::LOG_BYTES_IN_LINE;
use std::collections::HashMap;
use heap::immix::LineMark;

#[repr(C)]
pub struct GC {
    immix_space: Arc<ImmixSpace>,
    lo_space   : Arc<RwLock<FreeListSpace>>
}

lazy_static! {
    pub static ref MY_GC : RwLock<Option<GC>> = RwLock::new(None);
}

#[no_mangle]
pub extern fn gc_init(immix_size: usize, lo_size: usize, n_gcthreads: usize) {
    // set this line to turn on certain level of debugging info
//    simple_logger::init_with_level(log::LogLevel::Trace).ok();
    
    // init space size
    heap::IMMIX_SPACE_SIZE.store(immix_size, Ordering::SeqCst);
    heap::LO_SPACE_SIZE.store(lo_size, Ordering::SeqCst);
    
    let (immix_space, lo_space) = {
        let immix_space = Arc::new(ImmixSpace::new(immix_size));
        let lo_space    = Arc::new(RwLock::new(FreeListSpace::new(lo_size)));

        heap::gc::init(immix_space.clone(), lo_space.clone());        
        
        (immix_space, lo_space)
    };
    
    *MY_GC.write().unwrap() = Some(GC {immix_space: immix_space, lo_space: lo_space});
    println!("heap is {} bytes (immix: {} bytes, lo: {} bytes) . ", immix_size + lo_size, immix_size, lo_size);
    
    // gc threads
    heap::gc::GC_THREADS.store(n_gcthreads, Ordering::SeqCst);
    println!("{} gc threads", n_gcthreads);
    
    // init object model
    objectmodel::init();
}

#[no_mangle]
pub extern fn new_mutator() -> Box<ImmixMutatorLocal> {
    Box::new(ImmixMutatorLocal::new(MY_GC.read().unwrap().as_ref().unwrap().immix_space.clone()))
}

#[no_mangle]
#[allow(unused_variables)]
pub extern fn drop_mutator(mutator: Box<ImmixMutatorLocal>) {
    // rust will reclaim the boxed mutator
}

#[cfg(target_arch = "x86_64")]
#[link(name = "gc_clib_x64")]
extern "C" {
    pub fn set_low_water_mark();
}

#[no_mangle]
#[inline(always)]
pub extern fn yieldpoint(mutator: &mut Box<ImmixMutatorLocal>) {
    mutator.yieldpoint();
}

#[no_mangle]
#[inline(never)]
pub extern fn yieldpoint_slow(mutator: &mut Box<ImmixMutatorLocal>) {
    mutator.yieldpoint_slow()
}

#[no_mangle]
#[inline(always)]
pub extern fn alloc(mutator: &mut Box<ImmixMutatorLocal>, size: usize, align: usize) -> ObjectReference {
    let addr = mutator.alloc(size, align);
    unsafe {addr.to_object_reference()}
}

#[no_mangle]
pub extern fn alloc_slow(mutator: &mut Box<ImmixMutatorLocal>, size: usize, align: usize) -> ObjectReference {
    let ret = mutator.try_alloc_from_local(size, align);
    unsafe {ret.to_object_reference()}
}

#[no_mangle]
pub extern fn alloc_large(mutator: &mut Box<ImmixMutatorLocal>, size: usize) -> ObjectReference {
    let ret = freelist::alloc_large(size, 8, mutator, MY_GC.read().unwrap().as_ref().unwrap().lo_space.clone());
    unsafe {ret.to_object_reference()}
}

#[no_mangle]
pub extern fn myStat(){
    let mut myhash = myHashMap.write().unwrap();

    let mut immixSpace = MY_GC.read().unwrap().as_ref().unwrap().immix_space.clone();
    let mut usedBlocks = immixSpace.used_blocks.lock().unwrap();
    let mut usableBlocks = immixSpace.usable_blocks.lock().unwrap();

    let mut count = 0;
    let mut count2 = 0;
    let mut count3 = 0;
    let mut count4 = 0;
    let mut count5 = 0;
    let mut count6 = 0;
    let mut count7 = 0;
    let mut count8 = 0;
    let mut count9 = 0;


    for (key, val) in myhash.iter() {
        count = count + 1;
        //println!("iter {}",count2);
        if *val {
            count2 += 1;
            for element in usedBlocks.iter() {
                let mut block = element;
                let end =  block.start().plus(BYTES_IN_BLOCK);
                if *key >= block.start() && *key <= end {
                    count8 = count8+1;
                }

            }
            for element in usableBlocks.iter() {
                let mut block = element;
                let end =  block.start().plus(BYTES_IN_BLOCK);
                if *key >= block.start() && *key <= end {
                    count6 = count6+1;
                }

            }
        }
        else{
            count3 = count3 + 1;
            for element in usedBlocks.iter() {
                let mut block = element;
                let end =  block.start().plus(BYTES_IN_BLOCK);
                if *key >= block.start() && *key <= end {
                    count5 = count5+1;
                }

            }
            for element in usableBlocks.iter() {
                let mut block = element;
                let end =  block.start().plus(BYTES_IN_BLOCK);
                if *key >= block.start() && *key <= end {
                    count7 = count7+1;
                }

            }
            let line_table_index = key.diff(immixSpace.start()) >> LOG_BYTES_IN_LINE;            
            let markValue = immixSpace.line_mark_table().get(line_table_index);

            if markValue != LineMark::Free {
                count4 += 1;
            }

            if markValue == LineMark::Free {
                count9 += 1;
            }
            
        }
    }
    println!("------------------------------------------");
    println!("hash size {} ", count);
    println!("true found size {} ", count2);
    println!("true found in used blocks {} ", count8 );
    println!("true found in usable blocks {} ", count6 );

    println!("false found size {} ", count3);
    println!("false found in used blocks {} ", count5 );
    println!("false found in usable blocks {} ", count7 );

    println!("false not in free lines {} ",count4);
    println!("false in free lines {} ",count9);
    
}