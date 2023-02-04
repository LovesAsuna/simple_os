#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(kernel::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use core::panic::PanicInfo;

use x86_64::VirtAddr;

use bootloader_api::config::{BootloaderConfig, Mapping};
use bootloader_api::{entry_point, BootInfo};
use kernel::framebuffer::FrameBufferWriter;
use kernel::println;

pub static BOOTLOADER_CONFIG: BootloaderConfig = {
    let mut config = BootloaderConfig::new_default();
    config.mappings.physical_memory = Some(Mapping::Dynamic);
    config
};

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    kernel::hlt_loop()
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    kernel::test_panic_handler(info)
}

entry_point!(kernel_main, config = &BOOTLOADER_CONFIG);

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    kernel::init();
    let frame_buffer = boot_info.framebuffer.as_mut().unwrap();
    let info = frame_buffer.info();
    let buffer = frame_buffer.buffer_mut();
    kernel::framebuffer::WRITER
        .lock()
        .init(FrameBufferWriter::new(buffer, info));
    use kernel::allocator;
    use kernel::memory::{self, BootInfoFrameAllocator};

    use kernel::task::{executor::Executor, Task};
    println!("Hello World{}", "!");

    let phys_mem_offset = VirtAddr::new(
        boot_info
            .physical_memory_offset
            .clone()
            .into_option()
            .unwrap(),
    );
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&boot_info.memory_regions) };

    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("heap initialization failed");

    #[cfg(test)]
    test_main();

    let mut executor = Executor::new();
    executor.spawn(Task::new(kernel::task::keyboard::print_keypresses()));
    executor.run()
}

#[test_case]
fn trivial_assertion() {
    assert_eq!(1, 1);
}
