use std::{collections::HashSet, env};

type SnekVal = u64;

#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(i64)]
pub enum ErrCode {
    InvalidArgument = 1,
    Overflow = 2,
    IndexOutOfBounds = 3,
    InvalidVecSize = 4,
    OutOfMemory = 5,
}

const TRUE: u64 = 7;
const FALSE: u64 = 3;

static mut HEAP_START: *const u64 = std::ptr::null();
static mut HEAP_END: *const u64 = std::ptr::null();

#[link(name = "our_code")]
extern "C" {
    // The \x01 here is an undocumented feature of LLVM that ensures
    // it does not add an underscore in front of the name.
    // Courtesy of Max New (https://maxsnew.com/teaching/eecs-483-fa22/hw_adder_assignment.html)
    #[link_name = "\x01our_code_starts_here"]
    fn our_code_starts_here(input: u64, heap_start: *const u64, heap_end: *const u64) -> u64;
}

#[export_name = "\x01snek_error"]
pub extern "C" fn snek_error(errcode: i64) {
    if errcode == ErrCode::InvalidArgument as i64 {
        eprintln!("invalid argument");
    } else if errcode == ErrCode::Overflow as i64 {
        eprintln!("overflow");
    } else if errcode == ErrCode::IndexOutOfBounds as i64 {
        eprintln!("index out of bounds");
    } else if errcode == ErrCode::InvalidVecSize as i64 {
        eprintln!("vector size must be non-negative");
    } else {
        eprintln!("an error ocurred {}", errcode);
    }
    std::process::exit(errcode as i32);
}

#[export_name = "\x01snek_print"]
pub unsafe extern "C" fn snek_print(val: SnekVal) -> SnekVal {
    println!("{}", snek_str(val, &mut HashSet::new()));
    val
}

/// This function is called when the program needs to allocate `count` words of memory and there's no
/// space left. The function should try to clean up space by triggering a garbage collection. If there's
/// not enough space to hold `count` words after running the garbage collector, the program should terminate
/// with an `out of memory` error.
///
/// Args:
///     * `count`: The number of words the program is trying to allocate, including an extra word for
///       the size of the vector and an extra word to store metadata for the garbage collector, e.g.,
///       to allocate a vector of size 5, `count` will be 7.
///     * `heap_ptr`: The current position of the heap pointer (i.e., the value stored in `%r15`). It
///       is guaranteed that `heap_ptr + 8 * count > HEAP_END`, i.e., this function is only called if
///       there's not enough space to allocate `count` words.
///     * `stack_base`: A pointer to the "base" of the stack.
///     * `curr_rbp`: The value of `%rbp` in the stack frame that triggered the allocation.
///     * `curr_rsp`: The value of `%rsp` in the stack frame that triggered the allocation.
///
/// Returns:
///
/// The new heap pointer where the program should allocate the vector (i.e., the new value of `%r15`)
///
#[export_name = "\x01snek_try_gc"]
pub unsafe fn snek_try_gc(
    count: isize,
    heap_ptr: *const u64,
    stack_base: *const u64,
    curr_rbp: *const u64,
    curr_rsp: *const u64,
) -> *const u64 {
    // print the heap
    //snek_print_stack(stack_base, curr_rbp, curr_rsp);
    //print_heap(heap_ptr);
    // iterate the stack and find roots
    let mut stack_ptr = stack_base.sub(1);
    let mut to_visit:Vec<*mut u64> = Vec::new();
    let mut root_set:HashSet<*mut u64> = HashSet::new();
    let mut free_space =(((HEAP_END as u64) - (HEAP_START as u64))/8) as i64;
    while stack_ptr >= curr_rsp {
        let val = *stack_ptr;
        //println!("{}", val);
        if val != TRUE && val != FALSE && val != 1 && val & 1 == 1  &&
            (val < (HEAP_END as u64) && val >= (HEAP_START as u64)){ // make sure its not an instruction pointer
            //println!("active");
            let addr = (val - 1) as *mut u64;
            if root_set.insert(addr) {
                to_visit.push(addr);
                let active_size = addr.add(1).read() as i64;
                free_space = free_space - active_size - 2;
            }
        }
        stack_ptr = stack_ptr.sub(1);
    }
    // traverse tree from roots to find all live data and mark it
    while to_visit.len() > 0 {
        let curr_ptr = to_visit.remove(0);
        // mark
        *curr_ptr = 1;
        //println!("mark");
        let size = curr_ptr.add(1).read() as usize;
        for i in 0..size { // queue up any members of this vec not already queued up
            let elem = curr_ptr.add(2+i).read();
            if elem != TRUE && elem != FALSE && elem != 1 && elem & 1 == 1 {
                let addr = (elem - 1) as *mut u64;
                if root_set.insert(addr) {
                    to_visit.push(addr);
                    let active_size = addr.add(1).read() as i64;
                    free_space = free_space - active_size-2;
                }
            }
        }
    }
    //print_heap(heap_ptr);
    //println!("{}",free_space);
    if (free_space as isize) < count {
        eprintln!("out of memory");
        std::process::exit(ErrCode::OutOfMemory as i32)
    }
    // compacting 1: compute forwarding addresses
    let mut heap_cursor = HEAP_START as *mut u64;
    let mut free_heap_cursor = HEAP_START as *mut u64;
    // move both cursors until finding something that can be cleared
    while (free_heap_cursor as *const u64)< heap_ptr {
        let gc_tag = free_heap_cursor.read();
        if gc_tag & 1 != 0 { // move both cursors up by one vec
            let size = free_heap_cursor.add(1).read() as usize;
            heap_cursor = heap_cursor.add(2+size);
            free_heap_cursor = free_heap_cursor.add(2+size);
        } else {
            let size = heap_cursor.add(1).read() as usize;
            if size == 0 { // have hit the end of heap, cannot compact anymore
                return heap_ptr;
            } else {
                heap_cursor = heap_cursor.add(2+size); // advance the scanning cursor to next
            }
            break;
        }
    }
    // start assigning fowarding addresses
    while (heap_cursor as *const u64) < heap_ptr {
        let gc_tag = heap_cursor.read();
        let heap_cursor_size = heap_cursor.add(1).read() as usize;
        if gc_tag != 0 { 
            *heap_cursor = (free_heap_cursor as u64) + 1; // assign free space + tagged
            // move free heap cursor up
            let free_size = heap_cursor_size;
            free_heap_cursor = free_heap_cursor.add(2+free_size);
        }
        let heap_cursor_size = heap_cursor.add(1).read() as usize;
        heap_cursor = heap_cursor.add(2+heap_cursor_size);
    }
    //print_heap(heap_ptr);

    // compacting 2: update references
    // linear scan of stack
    stack_ptr = stack_base.sub(1);
    while stack_ptr >= curr_rsp {
        let val = *stack_ptr;
        if val != TRUE && val != FALSE && val != 1 && val & 1 == 1 && (val < (HEAP_END as u64) && val >= (HEAP_START as u64)){
            let addr = (val - 1) as *mut u64;
            // check if forwarding addr has been set for this addr
            let gc_tag = addr.read();
            if gc_tag & 1 == 1 && gc_tag != 1{
                *(stack_ptr as *mut u64) = gc_tag;
            }
        }
        stack_ptr = stack_ptr.sub(1);
    }
    // linear scan of heap
    heap_cursor = HEAP_START as *mut u64;
    while (heap_cursor as *const u64) < heap_ptr {
        let gc_tag = heap_cursor.read();
        let heap_cursor_size = heap_cursor.add(1).read() as usize;
        if gc_tag != 0 { 
            // if gc_tag == 1 { // marked, but not moving
            //     *heap_cursor = 0;
            // } else {
                for i in 2..2+heap_cursor_size {
                    let heap_val = heap_cursor.add(i).read();
                    if heap_val != TRUE && heap_val != FALSE && heap_val != 1 && heap_val & 1 == 1 {
                        let fwd_tag = ((heap_val-1) as *const u64).read();
                        if fwd_tag != 0  && fwd_tag != 1 {
                            let heap_val_ptr = heap_cursor.add(i) as *mut u64;
                            *heap_val_ptr = fwd_tag;
                        }
                    }
                    //println!("{}", heap_val);
                }
           // }
        }
        heap_cursor = heap_cursor.add(2+heap_cursor_size);
    }
    //snek_print_stack(stack_base, curr_rbp, curr_rsp);
    //print_heap(heap_ptr);
    // compacting 3: move the objects
    heap_cursor = HEAP_START as *mut u64;
    while (heap_cursor as *const u64) < heap_ptr {
        let gc_tag = heap_cursor.read();
        let heap_cursor_size = heap_cursor.add(1).read() as usize;
        if gc_tag != 0 { 
            if gc_tag == 1 {// marked, but not moving
                    *heap_cursor = 0;
            } else{
                let new_addr = (gc_tag - 1) as *mut u64;
                *new_addr = 0; // zero out gc metainfo
                *(new_addr.add(1)) = heap_cursor_size as u64; // set size field
                for i in 2..2+heap_cursor_size {
                    let heap_val = heap_cursor.add(i).read();
                    *(new_addr.add(i)) = heap_val;
                }
            }

        }
        heap_cursor = heap_cursor.add(2+heap_cursor_size);
    }
    let new_heap_ptr = free_heap_cursor;
    while (free_heap_cursor as *const u64) < heap_ptr {
        *free_heap_cursor = 0;
        free_heap_cursor = free_heap_cursor.add(1);
    }
    // print the heap
    //snek_print_stack(stack_base, curr_rbp, curr_rsp);
    //print_heap(heap_ptr);
    return new_heap_ptr;
    // eprintln!("out of memory");
    // std::process::exit(ErrCode::OutOfMemory as i32)
}

/// This function should trigger garbage collection and return the updated heap pointer (i.e., the new
/// value of `%r15`). See [`snek_try_gc`] for a description of the meaning of the arguments.
#[export_name = "\x01snek_gc"]
pub unsafe fn snek_gc(
    heap_ptr: *const u64,
    stack_base: *const u64,
    curr_rbp: *const u64,
    curr_rsp: *const u64,
) -> *const u64 {
    return snek_try_gc(0, heap_ptr, stack_base, curr_rbp, curr_rsp);
}

/// A helper function that can called with the `(snek-printstack)` snek function. It prints the stack
/// See [`snek_try_gc`] for a description of the meaning of the arguments.
#[export_name = "\x01snek_print_stack"]
pub unsafe fn snek_print_stack(stack_base: *const u64, curr_rbp: *const u64, curr_rsp: *const u64) {
    let mut ptr = stack_base;
    println!("-----------------------------------------");
    while ptr >= curr_rsp {
        let val = *ptr;
        println!("{ptr:?}: {:#0x}", val);
        ptr = ptr.sub(1);
    }
    println!("-----------------------------------------");
}

unsafe fn snek_str(val: SnekVal, seen: &mut HashSet<SnekVal>) -> String {
    if val == TRUE {
        format!("true")
    } else if val == FALSE {
        format!("false")
    } else if val & 1 == 0 {
        format!("{}", (val as i64) >> 1)
    } else if val == 1 {
        format!("nil")
    } else if val & 1 == 1 {
        if !seen.insert(val) {
            return "[...]".to_string();
        }
        let addr = (val - 1) as *const u64;
        let size = addr.add(1).read() as usize;
        let mut res = "[".to_string();
        for i in 0..size {
            let elem = addr.add(2 + i).read();
            res = res + &snek_str(elem, seen);
            if i < size - 1 {
                res = res + ", ";
            }
        }
        seen.remove(&val);
        res + "]"
    } else {
        format!("unknown value: {val}")
    }
}

unsafe fn print_heap(heap_ptr: *const u64) {
    let mut cursor = HEAP_START;
    println!("-----------------------------------------");
    println!("{cursor:?} to {heap_ptr:?}");
    while cursor < heap_ptr {
        let val = *cursor;
        println!("{cursor:?}: {:#0x}", val);
        cursor = cursor.add(1);
    }
    println!("-----------------------------------------");

}

fn parse_input(input: &str) -> u64 {
    match input {
        "true" => TRUE,
        "false" => FALSE,
        _ => (input.parse::<i64>().unwrap() << 1) as u64,
    }
}

fn parse_heap_size(input: &str) -> usize {
    input.parse::<usize>().unwrap()
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let input = if args.len() >= 2 { &args[1] } else { "false" };
    let heap_size = if args.len() >= 3 { &args[2] } else { "10000" };
    let input = parse_input(&input);
    let heap_size = parse_heap_size(&heap_size);

    // Initialize heap
    let mut heap: Vec<u64> = Vec::with_capacity(heap_size);
    unsafe {
        HEAP_START = heap.as_mut_ptr();
        HEAP_END = HEAP_START.add(heap_size);
    }

    let i: u64 = unsafe { our_code_starts_here(input, HEAP_START, HEAP_END) };
    unsafe { snek_print(i) };
}
