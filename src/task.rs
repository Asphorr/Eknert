use alloc::vec::Vec;
use spin::Mutex;
use lazy_static::lazy_static;

pub struct Task {
    id: usize,
    stack: Vec<u8>,
    stack_pointer: usize,
}

impl Task {
    pub fn new(entry_point: fn()) -> Self {
        let mut stack = Vec::with_capacity(4096);
        let stack_pointer = unsafe {
            let sp = stack.as_mut_ptr().add(4096);
            (sp as *mut usize).write(entry_point as usize);
            sp as usize
        };

        Task {
            id: 0,
            stack,
            stack_pointer,
        }
    }
}

pub struct Scheduler {
    tasks: Vec<Task>,
    current_task: usize,
}

impl Scheduler {
    pub fn new() -> Self {
        Scheduler {
            tasks: Vec::new(),
            current_task: 0,
        }
    }

    pub fn add_task(&mut self, task: Task) {
        self.tasks.push(task);
    }

    pub fn run_next_task(&mut self) {
        if self.tasks.is_empty() {
            return;
        }

        self.current_task = (self.current_task + 1) % self.tasks.len();
        let next_task = &mut self.tasks[self.current_task];

        unsafe {
            asm!(
                "mov rsp, {}",
                "ret",
                in(reg) next_task.stack_pointer,
                options(preserves_flags)
            );
        }
    }
}

lazy_static! {
    pub static ref SCHEDULER: Mutex<Scheduler> = Mutex::new(Scheduler::new());
}
