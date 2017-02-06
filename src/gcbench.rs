#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

#![allow(unused_variables)]

use heap;
use heap::immix::ImmixMutatorLocal;
use heap::immix::ImmixSpace;
use heap::immix::ImmixBlock;
use heap::immix::myHashMap;
use heap::immix::myHashMapForLine;
use heap::immix::LineMark;
use heap::immix::BYTES_IN_BLOCK;
use heap::immix::LOG_BYTES_IN_LINE;
use heap::immix::BYTES_IN_LINE;
use heap::immix::LINES_IN_BLOCK;
use heap::freelist;
use heap::freelist::FreeListSpace;
use std::mem::size_of as size_of;
use std::collections::HashSet;
use std::sync::RwLock;
use common::Address;
use std::sync::atomic;
use std::sync::Arc;
use std::collections::HashMap;

extern crate time;

const kStretchTreeDepth   : i32 = 18;
const kLongLivedTreeDepth : i32 = 16;
const kArraySize          : i32 = 500000;
const kMinTreeDepth       : i32 = 4;
const kMaxTreeDepth       : i32 = 16;


lazy_static! {
    pub static ref myHashSet : RwLock<HashSet<Address>> = {
        let mut ret :HashSet<Address> = HashSet::new();
        RwLock::new(ret)
    };
 //   pub static ref myHashMap : RwLock<HashMap<Address,bool>> = {
 //       let mut ret :HashMap<Address,bool> = HashMap::new();
 //       RwLock::new(ret)
 //   };
    
}

pub static mut OBJ_COUNT : usize = 0;



struct Node {
    left : *mut Node,
    right : *mut Node,
    i : i32,
    j : i32
}

struct Array {
    value : [f64; kArraySize as usize]
}

fn init_Node(me: *mut Node, l: *mut Node, r: *mut Node) {
    unsafe {
        (*me).left = l;
        (*me).right = r;
    }
}

fn TreeSize(i: i32) -> i32{
    (1 << (i + 1)) - 1
}

fn NumIters(i: i32) -> i32 {
    2 * TreeSize(kStretchTreeDepth) / TreeSize(i)
}

fn Populate(iDepth: i32, thisNode: *mut Node, mutator: &mut ImmixMutatorLocal) {
    if iDepth <= 0 {
        return;
    } else {
        unsafe {
            (*thisNode).left = alloc(mutator);
            (*thisNode).right = alloc(mutator);
            Populate(iDepth - 1, (*thisNode).left, mutator);
            Populate(iDepth - 1, (*thisNode).right, mutator);
        }
    }
}

fn MakeTree(iDepth: i32, mutator: &mut ImmixMutatorLocal) -> *mut Node {
    if iDepth <= 0 {
        alloc(mutator)
    } else {
        let left = MakeTree(iDepth - 1, mutator);
        let right = MakeTree(iDepth - 1, mutator);
        let result = alloc(mutator);
        init_Node(result, left, right);
        
        result
    }
}

fn PrintDiagnostics() {
    
}

fn TimeConstruction(depth: i32, mutator: &mut ImmixMutatorLocal) {
    let iNumIters = NumIters(depth);
    println!("creating {} trees of depth {}", iNumIters, depth);
    
    let tStart = time::now_utc();
    for _ in 0..iNumIters {
        let tempTree = alloc(mutator);
        Populate(depth, tempTree, mutator);
        
        // destroy tempTree
    }
    let tFinish = time::now_utc();
    println!("\tTop down construction took {} msec", (tFinish - tStart).num_milliseconds());
    
    let tStart = time::now_utc();
    for _ in 0..iNumIters {
        let tempTree = MakeTree(depth, mutator);
    }
    let tFinish = time::now_utc();
    println!("\tButtom up construction took {} msec", (tFinish - tStart).num_milliseconds());
}

#[inline(always)]
fn alloc(mutator: &mut ImmixMutatorLocal) -> *mut Node {
    let addr = mutator.alloc(size_of::<Node>(), 8);
    mutator.init_object(addr, 0b1100_0011);
//    objectmodel::init_header(unsafe{addr.to_object_reference()}, HEADER_INIT_U64);
//    let mut myhash = myHashMap.write().unwrap();
//    myhash.insert(addr,false);
    unsafe { OBJ_COUNT  = OBJ_COUNT + 1; }
    addr.to_ptr_mut::<Node>()
}

pub fn start() {
    use std::sync::{Arc, RwLock};
    use std::sync::atomic::Ordering;
    unsafe { OBJ_COUNT = 0; }
    unsafe {heap::gc::set_low_water_mark();}
    
    let immix_space : Arc<ImmixSpace> = {
        let space : ImmixSpace = ImmixSpace::new(heap::IMMIX_SPACE_SIZE.load(Ordering::SeqCst));
        Arc::new(space)
    };
    let lo_space : Arc<RwLock<FreeListSpace>> = {
        let space : FreeListSpace = FreeListSpace::new(heap::LO_SPACE_SIZE.load(Ordering::SeqCst));
        Arc::new(RwLock::new(space))
    };
    heap::gc::init(immix_space.clone(), lo_space.clone());
    let mut mutator = ImmixMutatorLocal::new(immix_space.clone());
    
    println!("Garbage Collector Test");
    println!(" Live storage will peak at {} bytes.\n", 
        2 * (size_of::<Node>() as i32) * TreeSize(kLongLivedTreeDepth) + 
        (size_of::<Array>() as i32));
    
    println!(" Stretching memory with a binary tree or depth {}", kStretchTreeDepth);
    PrintDiagnostics();
    
    let tStart = time::now_utc();
    // Stretch the memory space quickly
    let tempTree = MakeTree(kStretchTreeDepth, &mut mutator);
    // destroy tree
    
    // Create a long lived object
    println!(" Creating a long-lived binary tree of depth {}", kLongLivedTreeDepth);
    let longLivedTree = alloc(&mut mutator);
    Populate(kLongLivedTreeDepth, longLivedTree, &mut mutator);
    
    println!(" Creating a long-lived array of {} doubles", kArraySize);
    freelist::alloc_large(size_of::<Array>(), 8, &mut mutator, lo_space.clone());
    
    PrintDiagnostics();
    
    let mut d = kMinTreeDepth;
    while d <= kMaxTreeDepth {
        TimeConstruction(d, &mut mutator);
        d += 2;
    }
    
    if longLivedTree.is_null() {
        println!("Failed(long lived tree wrong)");
    }

    
    let tFinish = time::now_utc();
    let tElapsed = (tFinish - tStart).num_milliseconds();
    
    PrintDiagnostics();
    println!("Completed in {} msec", tElapsed);
    println!("Finished with {} collections", heap::gc::GC_COUNT.load(Ordering::SeqCst));

    let mut myhash = myHashMap.write().unwrap();



    let mut myhashLine = myHashMapForLine.write().unwrap();

  //  let mut myhashLineSanity = myHashMapForLine.write().unwrap();



    let mut usedBlocks = immix_space.used_blocks.lock().unwrap();
    let mut usableBlocks = immix_space.usable_blocks.lock().unwrap();
    let mut freeList = lo_space.write().unwrap();

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
                let line_table_index1 = key.diff(block.start()) >> LOG_BYTES_IN_LINE;
                let end =  block.start().plus(BYTES_IN_BLOCK);
                
            //    println!("address check {}", line_table_index1);
                if *key >= block.start() && *key <= end {
                    count8 = count8+1;
                    
                    let len = BYTES_IN_LINE;
                    for i in 0..LINES_IN_BLOCK {
                        let mut address=block.start().plus(i*BYTES_IN_LINE);
                        if *key >= address && *key <= address.plus(BYTES_IN_LINE){
                            //println!("line address check for used block true {}", address);

                            if myhashLine.contains_key(&address) == true {
                                let mut v = myhashLine[&address]+1;
                                myhashLine.remove(&address);
                                myhashLine.insert(address,v);
                            }
                            else{
                                myhashLine.insert(address,1);
                            } 
                        //    let mut v = myhashLineSanity[&address]+1;
                        //    myhashLineSanity.remove(&address);
                        //    myhashLineSanity.insert(address,++1);
                        }
                    }
                    
                    break;
                }
                

            }
        //    println!("");
            for element in usableBlocks.iter() {
            //    let mut address1=0;
                let mut block = element;
                let end =  block.start().plus(BYTES_IN_BLOCK);
                let len = BYTES_IN_LINE;
            //    address1 = address1+len;
                if *key >= block.start() && *key <= end {
                    count6 = count6+1;
             //       println!("line address check for usable block true {}", address1);
                    let len = BYTES_IN_LINE;
                    for i in 0..LINES_IN_BLOCK {
                        let mut address=block.start().plus(i*BYTES_IN_LINE);
                        if *key >= address && *key <= address.plus(BYTES_IN_LINE){
                           // println!("line address check for usable block true {}", address);
                       //     let mut v = myhashLineSanity[&address]+1;
                       //     myhashLineSanity.remove(&address);
                       //     myhashLineSanity.insert(address,v);
                            if myhashLine.contains_key(&address) == true {
                                let mut v = myhashLine[&address]+1;
                                myhashLine.remove(&address);
                                myhashLine.insert(address,v);
                            }
                            else{
                                myhashLine.insert(address,1);
                            }
                        }
                    }
                    break;
                }

            }
        //    println!("");
        }
        else{
            count3 = count3 + 1;
            for element in usedBlocks.iter() {
             //   let mut address2=0;
                let mut block = element;
                let end =  block.start().plus(BYTES_IN_BLOCK);
                let len = BYTES_IN_LINE;
            //    address2 = address2+len;
                if *key >= block.start() && *key <= end {
                    count5 = count5+1;
            //        println!("line address check for used block false {}", address2);
                    let len = BYTES_IN_LINE;
                    for i in 0..LINES_IN_BLOCK {
                        let mut address=block.start().plus(i*BYTES_IN_LINE);
                        if *key >= address && *key <= address.plus(BYTES_IN_LINE){
                         //   println!("line address check for used block false{}", address);
                   //         let mut v = myhashLineSanity[&address]+1;
                   //         myhashLineSanity.remove(&address);
                   //         myhashLineSanity.insert(address,v);
                        }
                    }
                    break;
                }

            }
        //    println!("");
            for element in usableBlocks.iter() {
             //   let mut address3=0;
                let mut block = element;
                let len = BYTES_IN_LINE;
             //   address3 = address3+len;
                let end =  block.start().plus(BYTES_IN_BLOCK);
                if *key >= block.start() && *key <= end {
                    count7 = count7+1;
            //        println!("line address check for usable block false {}", address3);
                    let len = BYTES_IN_LINE;
                    for i in 0..LINES_IN_BLOCK {
                        let mut address=block.start().plus(i*BYTES_IN_LINE);
                        if *key >= address && *key <= address.plus(BYTES_IN_LINE){
                       //     println!("line address check for usable block false {}", address);
                    //        let mut v = myhashLineSanity[&address]+1;
                    //        myhashLineSanity.remove(&address);
                    //        myhashLineSanity.insert(address,v);
                        }
                    }
                    break;
                }

            }
         //   println!("");
            
            
        }
        let line_table_index = key.diff(immix_space.start()) >> LOG_BYTES_IN_LINE;            
        let markValue = immix_space.line_mark_table().get(line_table_index);
    //    let address= immix_space.line_mark_table().take_slice(line_table_index << LOG_BYTES_IN_LINE, 256);

        if markValue != LineMark::Free {
            count4 += 1;
        }

        if markValue == LineMark::Free {
            count9 += 1;
        }
    }
    println!("------------------------------------------");
    println!("alloc in gc called {} ", unsafe { OBJ_COUNT } );
    println!("hash size {} ", count);
    println!("true found size {} ", count2);
    println!("true found in used blocks {} ", count8 );
    println!("true found in usable blocks {} ", count6 );

    println!("false found size {} ", count3);
    println!("false found in used blocks {} ", count5 );
    println!("false found in usable blocks {} ", count7 );

    println!("false not in free lines {} ",count4);
    println!("false in free lines {} ",count9);

    //for (key,val) in myhashLine.iter(){
    //    println!("TRUE OBJECT LINE ADDRESS {} COUNT VALUE {} ", key, val);
    //}

}