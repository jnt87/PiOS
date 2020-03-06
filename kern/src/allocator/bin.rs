use core::alloc::Layout;
use core::fmt;
use core::ptr;

use crate::allocator::linked_list::LinkedList;
use crate::allocator::util::*;
use crate::allocator::LocalAlloc;

/// A simple allocator that allocates based on size classes.
///   bin 0 (2^3 bytes)    : handles allocations in (0, 2^3]
///   bin 1 (2^4 bytes)    : handles allocations in (2^3, 2^4]
///   ...
///   bin 29 (2^22 bytes): handles allocations in (2^31, 2^32]
///   
///   map_to_bin(size) -> k
///   
pub struct Allocator {
    free_list: [LinkedList; 30],
    heap_ptr: usize,
    end: usize,
}

impl Allocator {
    /// Creates a new bin allocator that will allocate memory from the region
    /// starting at address `start` and ending at address `end`.
    pub fn new(start: usize, end: usize) -> Allocator {
        let mut free_list = [LinkedList::new(); 30];
        Allocator {
            free_list: free_list,
            heap_ptr: start,
            end: end,
        }
    }
}

impl LocalAlloc for Allocator {
    /// Allocates memory. Returns a pointer meeting the size and alignment
    /// properties of `layout.size()` and `layout.align()`.
    ///
    /// If this method returns an `Ok(addr)`, `addr` will be non-null address
    /// pointing to a block of storage suitable for holding an instance of
    /// `layout`. In particular, the block will be at least `layout.size()`
    /// bytes large and will be aligned to `layout.align()`. The returned block
    /// of storage may or may not have its contents initialized or zeroed.
    ///
    /// # Safety
    ///
    /// The _caller_ must ensure that `layout.size() > 0` and that
    /// `layout.align()` is a power of two. Parameters not meeting these
    /// conditions may result in undefined behavior.
    ///
    /// # Errors
    ///
    /// Returning null pointer (`core::ptr::null_mut`)
    /// indicates that either memory is exhausted
    /// or `layout` does not meet this allocator's
    /// size or alignment constraints.
    unsafe fn alloc(&mut self, layout: Layout) -> *mut u8 {
        let mut size = layout.size();
        if size < layout.align() {
            size = layout.align();
        }
        if size * 2 != size.next_power_of_two() {
            size = size.next_power_of_two();
        }
        let bin_num = (size.trailing_zeros().saturating_sub(3)) as usize;

        if !self.free_list[bin_num].is_empty() {
            return (self.free_list[bin_num].pop().unwrap() as *mut u8);
        }
        //make new space
        let pointer_addr = align_up(self.heap_ptr, layout.align());
        if self.end.saturating_sub(size) < pointer_addr {
            return core::ptr::null_mut() as *mut u8;
        } else {
            let padding = pointer_addr - self.heap_ptr;
            //let size = padding + layout.size();
            self.heap_ptr = self.heap_ptr + size + padding;
            return (pointer_addr as *mut u8);
        }

    }

    /// Deallocates the memory referenced by `ptr`.
    ///
    /// # Safety
    ///
    /// The _caller_ must ensure the following:
    ///
    ///   * `ptr` must denote a block of memory currently allocated via this
    ///     allocator
    ///   * `layout` must properly represent the original layout used in the
    ///     allocation call that returned `ptr`
    ///
    /// Parameters not meeting these conditions may result in undefined
    /// behavior.
    unsafe fn dealloc(&mut self, ptr: *mut u8, layout: Layout) {
        let mut size = layout.size();
        if size < layout.align() {
            size = layout.align();
        }
        if size * 2 != size.next_power_of_two() {
            size = size.next_power_of_two();
        }
        let bin_num = size.trailing_zeros().saturating_sub(3) as usize;

        unsafe {
            self.free_list[bin_num].push(ptr as *mut usize);
        }
    }
}

// FIXME: Implement `Debug` for `Allocator`.
impl fmt::Debug for Allocator {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "{:?}", self.heap_ptr);
        writeln!(f, "{:?}", self.end);
        Ok(())
    }
}
