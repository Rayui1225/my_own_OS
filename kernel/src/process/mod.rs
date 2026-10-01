//! The first process abstraction backed by a loader-produced user image.

use crate::{
    arch::riscv64::{csr, trap},
    loader::{self, LoadError},
    memory::{address_space::AddressSpace, paging},
    println, task,
    task::{TaskId, TaskState},
};

const INIT_IMAGE: &[u8] = include_bytes!("../../../user/bin/init.sbin");

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProcessError {
    AlreadyLoaded,
    Load(LoadError),
    TaskSpawnFailed,
    NotLoaded,
    StillRunning,
    MissingExitCode,
    KernelPageTableUnavailable,
}

struct Process {
    task_id: TaskId,
    address_space: AddressSpace,
    entry_point: usize,
    stack_pointer: usize,
}

static mut INIT_PROCESS: Option<Process> = None;

pub fn spawn_init() -> Result<TaskId, ProcessError> {
    if unsafe { INIT_PROCESS.is_some() } {
        return Err(ProcessError::AlreadyLoaded);
    }

    println!("[loader] load /bin/init");
    let program = loader::load(INIT_IMAGE).map_err(ProcessError::Load)?;
    println!("[loader] entry = {:#x}", program.entry_point());
    println!("[loader] user stack = {:#x}", program.stack_pointer());
    let (address_space, entry_point, stack_pointer) = program.into_parts();
    let task_id = match task::spawn(user_process_entry) {
        Ok(task_id) => task_id,
        Err(_) => {
            address_space.destroy();
            return Err(ProcessError::TaskSpawnFailed);
        }
    };

    unsafe {
        INIT_PROCESS = Some(Process {
            task_id,
            address_space,
            entry_point,
            stack_pointer,
        });
    }
    Ok(task_id)
}

pub fn read_current_user_byte(address: usize) -> Option<u8> {
    let current = task::current_id()?;
    let process = unsafe { INIT_PROCESS.as_ref()? };
    if process.task_id != current {
        return None;
    }

    process.address_space.read_user_byte(address)
}

pub fn reap_init() -> Result<i32, ProcessError> {
    let task_id = unsafe {
        INIT_PROCESS
            .as_ref()
            .map(|process| process.task_id)
            .ok_or(ProcessError::NotLoaded)?
    };
    if task::state(task_id) != Some(TaskState::Exited) {
        return Err(ProcessError::StillRunning);
    }

    paging::activate_kernel_page_table().map_err(|_| ProcessError::KernelPageTableUnavailable)?;
    let exit_code = task::exit_code(task_id).ok_or(ProcessError::MissingExitCode)?;
    let process = unsafe { INIT_PROCESS.take().ok_or(ProcessError::NotLoaded)? };
    process.address_space.destroy();
    Ok(exit_code)
}

fn user_process_entry() {
    println!("[process] switch to user mode");
    let process = unsafe {
        INIT_PROCESS
            .as_ref()
            .expect("init process disappeared before entering user mode")
    };
    process.address_space.activate();

    let frame = trap::TrapFrame {
        sp: process.stack_pointer,
        sepc: process.entry_point,
        sstatus: csr::user_sstatus(),
        ..trap::TrapFrame::default()
    };
    unsafe { trap::enter_user(&frame) }
}
