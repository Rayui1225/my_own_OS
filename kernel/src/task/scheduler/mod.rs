use alloc::collections::VecDeque;

use crate::{
    arch::riscv64::{
        context_switch::{self, Context},
        interrupt::InterruptGuard,
    },
    println,
};

use super::registry::TaskRegistry;
use super::thread::{Task, TaskEntry, TaskId, TaskState};

static mut SCHEDULER: Option<Scheduler> = None;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpawnError {
    NotInitialized,
    AlreadyStarted,
}

struct Scheduler {
    tasks: TaskRegistry,
    ready_queue: VecDeque<TaskId>,
    current: Option<TaskId>,
    bootstrap_context: Context,
    next_id: usize,
    started: bool,
}

struct TaskSwitch {
    current_id: TaskId,
    next_id: TaskId,
    current_context: *mut Context,
    next_context: *const Context,
}

impl Scheduler {
    fn new() -> Self {
        Self {
            tasks: TaskRegistry::new(),
            ready_queue: VecDeque::new(),
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
        self.tasks.insert(Task::new(id, entry));
        self.ready_queue.push_back(id);
        self.assert_invariants();
        Ok(id)
    }

    fn state(&self, id: TaskId) -> Option<TaskState> {
        self.tasks.get(id).map(|task| task.state)
    }

    fn exit_code(&self, id: TaskId) -> Option<i32> {
        self.tasks.get(id).and_then(|task| task.exit_code)
    }

    fn first_switch(&mut self) -> Option<(*mut Context, *const Context)> {
        if self.started {
            return None;
        }
        self.started = true;

        let next = self.ready_queue.pop_front()?;
        self.task_mut(next).transition_to(TaskState::Running);
        self.current = Some(next);
        self.assert_invariants();

        let current_context = &mut self.bootstrap_context as *mut Context;
        let next_context = &self.task(next).context as *const Context;
        Some((current_context, next_context))
    }

    fn yield_switch(&mut self) -> Option<TaskSwitch> {
        let current = self.current?;
        if self.ready_queue.is_empty() {
            self.assert_invariants();
            return None;
        }

        self.task_mut(current).transition_to(TaskState::Ready);
        self.ready_queue.push_back(current);

        let next = self
            .ready_queue
            .pop_front()
            .expect("ready queue became empty");
        self.task_mut(next).transition_to(TaskState::Running);
        self.current = Some(next);
        self.assert_invariants();

        let current_context = &mut self.task_mut(current).context as *mut Context;
        let next_context = &self.task(next).context as *const Context;
        Some(TaskSwitch {
            current_id: current,
            next_id: next,
            current_context,
            next_context,
        })
    }

    fn exit_switch(&mut self, code: i32) -> (*mut Context, *const Context) {
        let current = self.current.expect("no running task to exit");
        let current_task = self.task_mut(current);
        current_task.exit_code = Some(code);
        current_task.transition_to(TaskState::Exited);

        let next = self.ready_queue.pop_front();
        let current_context = &mut self.task_mut(current).context as *mut Context;

        if let Some(next) = next {
            self.task_mut(next).transition_to(TaskState::Running);
            self.current = Some(next);
            self.assert_invariants();
            let next_context = &self.task(next).context as *const Context;
            (current_context, next_context)
        } else {
            self.current = None;
            self.assert_invariants();
            let bootstrap_context = &self.bootstrap_context as *const Context;
            (current_context, bootstrap_context)
        }
    }

    fn task(&self, id: TaskId) -> &Task {
        self.tasks.get(id).expect("task is missing from registry")
    }

    fn task_mut(&mut self, id: TaskId) -> &mut Task {
        self.tasks
            .get_mut(id)
            .expect("task is missing from registry")
    }

    fn assert_invariants(&self) {
        let running_count = self
            .tasks
            .iter()
            .filter(|task| task.state == TaskState::Running)
            .count();
        debug_assert_eq!(running_count, usize::from(self.current.is_some()));

        for task in self.tasks.iter() {
            let queue_entries = self.ready_queue.iter().filter(|id| **id == task.id).count();

            if task.state == TaskState::Ready {
                debug_assert_eq!(queue_entries, 1);
            } else {
                debug_assert_eq!(queue_entries, 0);
            }
        }
    }
}

pub fn init() {
    let _interrupt_guard = InterruptGuard::new();
    unsafe {
        assert!(SCHEDULER.is_none(), "scheduler already initialized");
        SCHEDULER = Some(Scheduler::new());
    }
}

#[cfg(feature = "test-kernel")]
pub(crate) fn reset_for_test() {
    let _interrupt_guard = InterruptGuard::new();
    unsafe {
        if let Some(scheduler) = SCHEDULER.as_ref() {
            assert!(
                scheduler.current.is_none(),
                "cannot reset a running scheduler"
            );
        }
        SCHEDULER = None;
    }
}

pub fn spawn(entry: TaskEntry) -> Result<TaskId, SpawnError> {
    let _interrupt_guard = InterruptGuard::new();
    unsafe {
        SCHEDULER
            .as_mut()
            .ok_or(SpawnError::NotInitialized)?
            .spawn(entry)
    }
}

pub fn state(id: TaskId) -> Option<TaskState> {
    let _interrupt_guard = InterruptGuard::new();
    unsafe { SCHEDULER.as_ref().and_then(|scheduler| scheduler.state(id)) }
}

pub(crate) fn exit_code(id: TaskId) -> Option<i32> {
    let _interrupt_guard = InterruptGuard::new();
    unsafe {
        SCHEDULER
            .as_ref()
            .and_then(|scheduler| scheduler.exit_code(id))
    }
}

pub(crate) fn current_id() -> Option<TaskId> {
    let _interrupt_guard = InterruptGuard::new();
    unsafe { SCHEDULER.as_ref().and_then(|scheduler| scheduler.current) }
}

pub fn run() {
    let _interrupt_guard = InterruptGuard::new();
    let switch = unsafe { SCHEDULER.as_mut().and_then(Scheduler::first_switch) };
    if let Some((current, next)) = switch {
        unsafe { context_switch::switch_context(current, next) };
    }
}

#[cfg_attr(not(feature = "test-kernel"), allow(dead_code))]
pub fn yield_now() {
    let _interrupt_guard = InterruptGuard::new();
    let switch = unsafe { SCHEDULER.as_mut().and_then(Scheduler::yield_switch) };
    if let Some(task_switch) = switch {
        unsafe {
            context_switch::switch_context(task_switch.current_context, task_switch.next_context)
        };
    }
}

pub(crate) fn on_timer_tick() {
    let _interrupt_guard = InterruptGuard::new();
    let switch = unsafe { SCHEDULER.as_mut().and_then(Scheduler::yield_switch) };

    if let Some(task_switch) = switch {
        println!(
            "[scheduler] switch task {} -> task {}",
            task_switch.current_id.value(),
            task_switch.next_id.value()
        );
        unsafe {
            context_switch::switch_context(task_switch.current_context, task_switch.next_context)
        };
    }
}

pub(crate) fn exit_current(code: i32) -> ! {
    let _interrupt_guard = InterruptGuard::new();
    let (current, next) = unsafe {
        SCHEDULER
            .as_mut()
            .expect("scheduler is not initialized")
            .exit_switch(code)
    };

    unsafe { context_switch::switch_context(current, next) };
    panic!("an exited task was resumed")
}
