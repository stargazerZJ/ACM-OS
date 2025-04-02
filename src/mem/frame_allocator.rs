use crate::config::{PAGE_SIZE, PAGE_SIZE_BITS};
use crate::mem::address::PhysPageNum;

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
