use alloc::{boxed::Box, vec};

use crate::arch::riscv64::context_switch::{self, Context};

const KERNEL_STACK_SIZE: usize = 16 * 1024;
const STACK_WORDS: usize = KERNEL_STACK_SIZE / core::mem::size_of::<u128>();

pub type TaskEntry = fn();

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TaskId(usize);

impl TaskId {
    pub const fn new(value: usize) -> Self {
        Self(value)
    }

    pub const fn value(self) -> usize {
        self.0
    }
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TaskState {
    Ready,
    Running,
    Blocked,
    Exited,
}

pub(super) struct Task {
    pub id: TaskId,
    pub state: TaskState,
    pub context: Context,
    _kernel_stack: KernelStack,
}

impl Task {
    pub fn new(id: TaskId, entry: TaskEntry) -> Self {
        let kernel_stack = KernelStack::new();
        let context = Context {
            ra: context_switch::task_entry_address(),
            sp: kernel_stack.top(),
            s0: entry as usize,
            ..Context::default()
        };

        Self {
            id,
            state: TaskState::Ready,
            context,
            _kernel_stack: kernel_stack,
        }
    }

    pub fn transition_to(&mut self, next: TaskState) {
        let is_valid = matches!(
            (self.state, next),
            (TaskState::Ready, TaskState::Running)
                | (TaskState::Running, TaskState::Ready)
                | (TaskState::Running, TaskState::Blocked)
                | (TaskState::Running, TaskState::Exited)
                | (TaskState::Blocked, TaskState::Ready)
        );
        assert!(is_valid, "invalid task state transition");
        self.state = next;
    }
}

struct KernelStack {
    storage: Box<[u128]>,
}

impl KernelStack {
    fn new() -> Self {
        Self {
            storage: vec![0; STACK_WORDS].into_boxed_slice(),
        }
    }

    fn top(&self) -> usize {
        let start = self.storage.as_ptr() as usize;
        let top = start + self.storage.len() * core::mem::size_of::<u128>();
        debug_assert_eq!(top & 0xf, 0);
        top
    }
}
