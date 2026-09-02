use alloc::{slice, vec::Vec};

use super::thread::{Task, TaskId};

/// Owns tasks and resolves stable task IDs to their storage location.
pub(super) struct TaskRegistry {
    tasks: Vec<Task>,
}

impl TaskRegistry {
    pub const fn new() -> Self {
        Self { tasks: Vec::new() }
    }

    pub fn insert(&mut self, task: Task) {
        assert!(self.get(task.id).is_none(), "duplicate task ID");
        self.tasks.push(task);
    }

    pub fn get(&self, id: TaskId) -> Option<&Task> {
        self.tasks.iter().find(|task| task.id == id)
    }

    pub fn get_mut(&mut self, id: TaskId) -> Option<&mut Task> {
        self.tasks.iter_mut().find(|task| task.id == id)
    }

    pub fn iter(&self) -> slice::Iter<'_, Task> {
        self.tasks.iter()
    }
}
