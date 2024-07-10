use x86_64::{
    structures::paging::{PageTable, PageTableFlags, PhysFrame, Size4KiB},
    PhysAddr, VirtAddr,
};
use x86_64::structures::paging::mapper::MapToError;
use x86_64::structures::paging::{FrameAllocator, Mapper, Page, PageTableFlags, Size4KiB};

pub struct MemoryManager {
    next_free_frame: PhysAddr,
}

impl MemoryManager {
    pub fn new() -> Self {
        MemoryManager {
            next_free_frame: PhysAddr::new(0x100000), // Start at 1 MB
        }
    }

    pub fn allocate_frame(&mut self) -> Option<PhysFrame> {
        let frame = PhysFrame::containing_address(self.next_free_frame);
        self.next_free_frame += Size4KiB::SIZE;
        Some(frame)
    }
}

pub fn init() {
    let mut memory_manager = MemoryManager::new();
    let mut mapper = unsafe { memory::init(PhysAddr::new(0xb8000)) };

    for i in 0..10 {
        let page = Page::containing_address(VirtAddr::new(0x400000 + i * 0x1000));
        let frame = memory_manager.allocate_frame().expect("no more frames");
        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
        unsafe {
            mapper.map_to(page, frame, flags, &mut memory_manager)
                .expect("map_to failed")
                .flush();
        }
    }
}

// Implement FrameAllocator for MemoryManager
unsafe impl FrameAllocator<Size4KiB> for MemoryManager {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        self.allocate_frame()
    }
}
