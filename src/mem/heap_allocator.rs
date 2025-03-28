
use buddy_system_allocator::LockedHeap;
use crate::config::KERNEL_HEAP_SIZE;
use crate::println;

#[global_allocator]
static HEAP_ALLOCATOR: LockedHeap = LockedHeap::empty();

unsafe extern "C" {
    fn kernel_heap_beg();
    fn kernel_heap_end();
}

pub fn init_heap() {
    unsafe {
        let actual_heap_size = kernel_heap_end as usize - kernel_heap_beg as usize;
        assert_eq!(
            actual_heap_size, KERNEL_HEAP_SIZE,
            "Kernel heap size mismatch: expected {}, got {}",
            KERNEL_HEAP_SIZE, actual_heap_size
        );
        HEAP_ALLOCATOR
            .lock()
            .init(kernel_heap_beg as usize, KERNEL_HEAP_SIZE);
    }
}

#[alloc_error_handler]
pub fn handle_alloc_error(layout: core::alloc::Layout) -> ! {
    panic!("Heap allocation error, layout = {:?}", layout);
}

#[allow(unused)]
pub fn heap_test() {
    use alloc::boxed::Box;
    use alloc::vec::Vec;
    let bss_range = kernel_heap_beg as usize..kernel_heap_end as usize;
    let a = Box::new(5);
    assert_eq!(*a, 5);
    assert!(bss_range.contains(&(a.as_ref() as *const _ as usize)));
    drop(a);
    let mut v: Vec<usize> = Vec::new();
    for i in 0..500 {
        v.push(i);
    }
    for i in 0..500 {
        assert_eq!(v[i], i);
    }
    assert!(bss_range.contains(&(v.as_ptr() as usize)));
    drop(v);
    println!("heap_test passed!");
}