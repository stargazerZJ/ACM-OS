use alloc::vec::Vec;
use lazy_static::lazy_static;
use crate::config::{MEMORY_END, PAGE_SIZE, PAGE_SIZE_BITS};
use crate::mem::address::{PhysAddr, PhysPageNum};
use crate::println;
use crate::sync::UPSafeCell;

trait FrameAllocator {
    fn new() -> Self;
    fn alloc(&mut self) -> Option<PhysPageNum>;
    fn dealloc(&mut self, ppn: PhysPageNum);
}

static VANITY_MAGIC_NUMBER: usize = 0xdeadbeef;

pub struct LinkedListFrameAllocator {
    range: (PhysPageNum, PhysPageNum),
    head: usize
}

type FrameAllocatorImpl = StackFrameAllocator;
lazy_static! {
    pub static ref FRAME_ALLOCATOR: UPSafeCell<FrameAllocatorImpl> = unsafe {
        UPSafeCell::new(FrameAllocatorImpl::new())
    };
}

pub fn init_frame_allocator() {
    extern "C" {
        fn ekernel();
    }
    FRAME_ALLOCATOR
        .exclusive_access()
        .init(PhysAddr::from(ekernel as usize).ceil(), PhysAddr::from(MEMORY_END).floor());
}

pub fn frame_alloc() -> Option<FrameTracker> {
    FRAME_ALLOCATOR
        .exclusive_access()
        .alloc()
        .map(|ppn| FrameTracker::new(ppn))
}

fn frame_dealloc(ppn: PhysPageNum) {
    FRAME_ALLOCATOR
        .exclusive_access()
        .dealloc(ppn);
}

pub struct FrameTracker {
    pub ppn: PhysPageNum,
}

impl FrameTracker {
    pub fn new(ppn: PhysPageNum) -> Self {
        // page cleaning
        let bytes_array = ppn.get_bytes_array();
        for i in bytes_array {
            *i = 0;
        }
        Self { ppn }
    }
}

impl Drop for FrameTracker {
    fn drop(&mut self) {
        frame_dealloc(self.ppn);
    }
}

impl LinkedListFrameAllocator {
    // Initialize the free list with pages in [l, r)
    pub fn init(&mut self, l: PhysPageNum, r: PhysPageNum) {
        assert!(l.0 > 0, "Invalid starting physical page number");
        assert!(l.0 < r.0, "Invalid physical page range");

        // Store the valid range
        self.range = (l, r);

        // Create a linked list of free pages
        self.head = 0;

        // Iterate from right to left (r-1 down to l)
        // This way we'll allocate from lower addresses first
        let mut current = r.0;
        while current > l.0 {
            current -= 1;
            let frame_address = current << PAGE_SIZE_BITS;

            // Store next pointer at start of page
            unsafe {
                let ptr = frame_address as *mut usize;
                *ptr = self.head;

                // Store magic number after the next pointer
                let magic_ptr = ptr.add(1);
                *magic_ptr = VANITY_MAGIC_NUMBER;
            }

            // Update head to point to this page
            self.head = current;
        }
    }

    // Helper to check if a PPN is within our valid range
    fn is_valid_ppn(&self, ppn: PhysPageNum) -> bool {
        ppn.0 >= self.range.0.0 && ppn.0 < self.range.1.0
    }
}

impl FrameAllocator for LinkedListFrameAllocator {
    fn new() -> Self {
        Self {
            head: 0, // invalid until initialized
            range: (PhysPageNum(0), PhysPageNum(0)), // invalid range
        }
    }

    fn alloc(&mut self) -> Option<PhysPageNum> {
        if self.head == 0 {
            return None; // No free frames
        }

        let allocated_ppn = PhysPageNum(self.head);
        let frame_address = self.head << PAGE_SIZE_BITS;

        // Read the next pointer
        unsafe {
            let ptr = frame_address as *const usize;
            self.head = *ptr;

            // Verify magic number
            let magic_ptr = ptr.add(1);
            assert_eq!(*magic_ptr, VANITY_MAGIC_NUMBER,
                       "Memory corruption detected during allocation");

            // Clear the whole page
            let page_ptr = frame_address as *mut u8;
            core::ptr::write_bytes(page_ptr, 0, PAGE_SIZE);
        }

        Some(allocated_ppn)
    }

    fn dealloc(&mut self, ppn: PhysPageNum) {
        // Validate the PPN
        assert!(ppn.0 > 0, "Attempting to deallocate invalid PPN 0");
        assert!(self.is_valid_ppn(ppn), "PPN outside valid memory range");

        let frame_address = ppn.0 << PAGE_SIZE_BITS;

        // Check for double free by looking at the magic number position
        unsafe {
            let magic_ptr = (frame_address as *const usize).add(1);
            assert_ne!(*magic_ptr, VANITY_MAGIC_NUMBER,
                       "Double free detected for PPN {}", ppn.0);

            // Store next pointer and magic number
            let ptr = frame_address as *mut usize;
            *ptr = self.head;
            *ptr.add(1) = VANITY_MAGIC_NUMBER;
        }

        // Update head to point to this newly freed frame
        self.head = ppn.0;
    }
}

pub struct StackFrameAllocator {
    current: usize,
    end: usize,
    recycled: Vec<usize>,
}

impl FrameAllocator for StackFrameAllocator {
    fn new() -> Self {
        Self {
            current: 0,
            end: 0,
            recycled: Vec::new(),
        }
    }
    fn alloc(&mut self) -> Option<PhysPageNum> {
        if let Some(ppn) = self.recycled.pop() {
            Some(ppn.into())
        } else {
            if self.current == self.end {
                None
            } else {
                self.current += 1;
                Some((self.current - 1).into())
            }
        }
    }
    fn dealloc(&mut self, ppn: PhysPageNum) {
        let ppn = ppn.0;
        // validity check
        if ppn >= self.current || self.recycled
            .iter()
            .find(|&v| {*v == ppn})
            .is_some() {
            panic!("Frame ppn={:#x} has not been allocated!", ppn);
        }
        // recycle
        self.recycled.push(ppn);
    }
}

impl StackFrameAllocator {
    pub fn init(&mut self, l: PhysPageNum, r: PhysPageNum) {
        self.current = l.0;
        self.end = r.0;
    }
}

#[allow(unused)]
pub fn frame_allocator_test() {
    let mut v: Vec<FrameTracker> = Vec::new();
    for i in 0..5 {
        let frame = frame_alloc().unwrap();
        println!("{:?}", frame);
        v.push(frame);
    }
    v.clear();
    for i in 0..5 {
        let frame = frame_alloc().unwrap();
        println!("{:?}", frame);
        v.push(frame);
    }
    drop(v);
    println!("frame_allocator_test passed!");
}