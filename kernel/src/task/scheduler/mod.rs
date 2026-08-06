use alloc::vec::Vec;

use crate::arch::riscv64::context_switch::{self, Context};

use super::thread::{Task, TaskEntry, TaskId, TaskState};

static mut SCHEDULER: Option<Scheduler> = None;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpawnError {
    NotInitialized,
    AlreadyStarted,
}

struct Scheduler {
    tasks: Vec<Task>,
    current: Option<usize>,
    bootstrap_context: Context,
    next_id: usize,
    started: bool,
}

impl Scheduler {
    fn new() -> Self {
        Self {
            tasks: Vec::new(),
            current: None,
            bootstrap_context: Context::default(),
            next_id: 1,
            started: false,
        }
    }

    fn spawn(&mut self, entry: TaskEntry) -> Result<TaskId, SpawnError> {
        if self.started {
            return Err(SpawnError::AlreadyStarted);
        }

        let id = TaskId::new(self.next_id);
        self.next_id += 1;
        self.tasks.push(Task::new(id, entry));
        Ok(id)
    }

    fn state(&self, id: TaskId) -> Option<TaskState> {
        self.tasks
            .iter()
            .find(|task| task.id == id)
            .map(|task| task.state)
    }

    fn first_switch(&mut self) -> Option<(*mut Context, *const Context)> {
        if self.started {
            return None;
        }
        self.started = true;

        let next = self.find_next_ready(0)?;
        self.tasks[next].state = TaskState::Running;
        self.current = Some(next);

        let current_context = &mut self.bootstrap_context as *mut Context;
        let next_context = &self.tasks[next].context as *const Context;
        Some((current_context, next_context))
    }

    fn yield_switch(&mut self) -> Option<(*mut Context, *const Context)> {
        let current = self.current?;
        let next = self.find_next_ready(current + 1)?;

        self.tasks[current].state = TaskState::Ready;
        self.tasks[next].state = TaskState::Running;
        self.current = Some(next);

        let tasks = self.tasks.as_mut_ptr();
        let current_context = unsafe { &mut (*tasks.add(current)).context as *mut Context };
        let next_context = unsafe { &(*tasks.add(next)).context as *const Context };
        Some((current_context, next_context))
    }

    fn exit_switch(&mut self) -> (*mut Context, *const Context) {
        let current = self.current.expect("no running task to exit");
        self.tasks[current].state = TaskState::Exited;

        let next = self.find_next_ready(current + 1);
        let tasks = self.tasks.as_mut_ptr();
        let current_context = unsafe { &mut (*tasks.add(current)).context as *mut Context };

        if let Some(next) = next {
            self.tasks[next].state = TaskState::Running;
            self.current = Some(next);
            let next_context = unsafe { &(*tasks.add(next)).context as *const Context };
            (current_context, next_context)
        } else {
            self.current = None;
            let bootstrap_context = &self.bootstrap_context as *const Context;
            (current_context, bootstrap_context)
        }
    }

    fn find_next_ready(&self, start: usize) -> Option<usize> {
        if self.tasks.is_empty() {
            return None;
        }

        for offset in 0..self.tasks.len() {
            let index = (start + offset) % self.tasks.len();
            if self.tasks[index].state == TaskState::Ready {
                return Some(index);
            }
        }
        None
    }
}

pub fn init() {
    unsafe {
        assert!(SCHEDULER.is_none(), "scheduler already initialized");
        SCHEDULER = Some(Scheduler::new());
    }
}

pub fn spawn(entry: TaskEntry) -> Result<TaskId, SpawnError> {
    unsafe {
        SCHEDULER
            .as_mut()
            .ok_or(SpawnError::NotInitialized)?
            .spawn(entry)
    }
}

pub fn state(id: TaskId) -> Option<TaskState> {
    unsafe { SCHEDULER.as_ref().and_then(|scheduler| scheduler.state(id)) }
}

pub fn run() {
    let switch = unsafe { SCHEDULER.as_mut().and_then(Scheduler::first_switch) };
    if let Some((current, next)) = switch {
        unsafe { context_switch::switch_context(current, next) };
    }
}

pub fn yield_now() {
    let switch = unsafe { SCHEDULER.as_mut().and_then(Scheduler::yield_switch) };
    if let Some((current, next)) = switch {
        unsafe { context_switch::switch_context(current, next) };
    }
}

pub(super) fn exit_current() -> ! {
    let (current, next) = unsafe {
        SCHEDULER
            .as_mut()
            .expect("scheduler is not initialized")
            .exit_switch()
    };

    unsafe { context_switch::switch_context(current, next) };
    panic!("an exited task was resumed")
}
