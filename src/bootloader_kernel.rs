#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

use core::panic::PanicInfo;
use core::sync::atomic::{AtomicUsize, Ordering};
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};
use x86_64::instructions::{port::Port, interrupts};
use x86_64::VirtAddr;
use spin::Mutex;
use alloc::{vec::Vec, string::String, boxed::Box};
use lazy_static::lazy_static;

extern crate alloc;

mod vga_buffer;
mod memory;
mod task;
mod filesystem;

use vga_buffer::{WRITER, Color};
use memory::MemoryManager;
use task::{Task, SCHEDULER};
use filesystem::FileSystem;

static TIMER_TICKS: AtomicUsize = AtomicUsize::new(0);

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        idt.double_fault.set_handler_fn(double_fault_handler);
        idt[InterruptIndex::Timer.as_usize()].set_handler_fn(timer_interrupt_handler);
        idt[InterruptIndex::Keyboard.as_usize()].set_handler_fn(keyboard_interrupt_handler);
        idt
    };
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = 32,
    Keyboard,
}

impl InterruptIndex {
    fn as_u8(self) -> u8 {
        self as u8
    }

    fn as_usize(self) -> usize {
        usize::from(self.as_u8())
    }
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    println!("Initializing RustOS...");

    memory::init();
    IDT.load();
    x86_64::instructions::interrupts::enable();

    let mut fs = FileSystem::new();
    fs.create_file("welcome.txt", "Welcome to RustOS!".as_bytes());

    SCHEDULER.lock().add_task(Task::new(task1));
    SCHEDULER.lock().add_task(Task::new(task2));

    println!("RustOS initialized successfully!");
    println!("Type 'help' for available commands.");

    loop {
        print!("> ");
        let command = read_line();
        handle_command(&command, &mut fs);
        SCHEDULER.lock().run_next_task();
    }
}

fn handle_command(command: &str, fs: &mut FileSystem) {
    match command.trim() {
        "help" => {
            println!("Available commands:");
            println!("  help - Show this help message");
            println!("  clear - Clear the screen");
            println!("  reboot - Reboot the system");
            println!("  ls - List files");
            println!("  cat <filename> - Display file contents");
            println!("  write <filename> <content> - Write content to a file");
        }
        "clear" => vga_buffer::WRITER.lock().clear_screen(),
        "reboot" => reboot(),
        "ls" => {
            for file in fs.list_files() {
                println!("{}", file);
            }
        }
        cmd if cmd.starts_with("cat ") => {
            let filename = &cmd[4..];
            match fs.read_file(filename) {
                Some(content) => println!("{}", core::str::from_utf8(&content).unwrap_or("Invalid UTF-8")),
                None => println!("File not found: {}", filename),
            }
        }
        cmd if cmd.starts_with("write ") => {
            let parts: Vec<&str> = cmd.splitn(3, ' ').collect();
            if parts.len() == 3 {
                fs.create_file(parts[1], parts[2].as_bytes());
                println!("File written: {}", parts[1]);
            } else {
                println!("Usage: write <filename> <content>");
            }
        }
        _ => println!("Unknown command. Type 'help' for available commands."),
    }
}

fn read_line() -> String {
    let mut input = String::new();
    loop {
        let key = wait_for_key();
        match key {
            b'\r' => {
                println!();
                return input;
            }
            8 => {
                if !input.is_empty() {
                    input.pop();
                    print!("\x08 \x08");
                }
            }
            32..=126 => {
                input.push(key as char);
                print!("{}", key as char);
            }
            _ => {}
        }
    }
}

fn wait_for_key() -> u8 {
    let mut port = Port::new(0x64);
    let mut data_port = Port::new(0x60);
    unsafe {
        while port.read() & 1 == 0 {}
        data_port.read()
    }
}

fn reboot() {
    unsafe {
        Port::new(0x64).write(0xFE as u8);
    }
    loop {}
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn double_fault_handler(stack_frame: InterruptStackFrame, _error_code: u64) -> ! {
    panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    TIMER_TICKS.fetch_add(1, Ordering::Relaxed);
    unsafe {
        Port::new(0x20).write(0x20 as u8);
    }
}

extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: InterruptStackFrame) {
    let mut port = Port::new(0x60);
    let scancode: u8 = unsafe { port.read() };
    // Handle keyboard input here
    unsafe {
        Port::new(0x20).write(0x20 as u8);
    }
}

fn task1() {
    loop {
        println!("Task 1 running");
        for _ in 0..1000000 {}
    }
}

fn task2() {
    loop {
        println!("Task 2 running");
        for _ in 0..1000000 {}
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("Kernel panic: {}", info);
    loop {}
}
