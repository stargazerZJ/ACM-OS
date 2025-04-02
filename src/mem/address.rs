

#[repr(C)]
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct PhysPageNum(pub usize);

// impl PhysPageNum {
//     pub fn get_pte_array(&self) -> &'static mut [PageTableEntry] {
//         let pa: PhysAddr = self.clone().into();
//         unsafe {
//             core::slice::from_raw_parts_mut(pa.0 as *mut PageTableEntry, 512)
//         }
//     }
//     pub fn get_bytes_array(&self) -> &'static mut [u8] {
//         let pa: PhysAddr = self.clone().into();
//         unsafe {
//             core::slice::from_raw_parts_mut(pa.0 as *mut u8, 4096)
//         }
//     }
//     pub fn get_mut<T>(&self) -> &'static mut T {
//         let pa: PhysAddr = self.clone().into();
//         unsafe {
//             (pa.0 as *mut T).as_mut().unwrap()
//         }
//     }
// }