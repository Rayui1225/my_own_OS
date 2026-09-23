//! The first process abstraction and its fixed, embedded init image.

use core::{arch::global_asm, slice};

use crate::{
    arch::riscv64::{csr, trap},
    memory::{
        address_space::{AddressSpace, AddressSpaceError, USER_CODE_BASE, USER_STACK_TOP},
        paging,
    },
    println, task,
    task::{TaskId, TaskState},
};

global_asm!(include_str!("init.S"));

extern "C" {
    static __user_init_start: u8;
    static __user_init_end: u8;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProcessError {
    AlreadyLoaded,
    AddressSpace(AddressSpaceError),
    TaskSpawnFailed,
    NotLoaded,
    StillRunning,
    MissingExitCode,
    KernelPageTableUnavailable,
}

struct Process {
    task_id: TaskId,
    address_space: AddressSpace,
}

static mut INIT_PROCESS: Option<Process> = None;

pub fn spawn_init() -> Result<TaskId, ProcessError> {
    if unsafe { INIT_PROCESS.is_some() } {
        return Err(ProcessError::AlreadyLoaded);
    }

    let address_space = AddressSpace::new(init_image()).map_err(ProcessError::AddressSpace)?;
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
        sp: USER_STACK_TOP,
        sepc: USER_CODE_BASE,
        sstatus: csr::user_sstatus(),
        ..trap::TrapFrame::default()
    };
    unsafe { trap::enter_user(&frame) }
}

fn init_image() -> &'static [u8] {
    unsafe {
        let start = core::ptr::addr_of!(__user_init_start);
        let end = core::ptr::addr_of!(__user_init_end);
        let length = end.offset_from(start) as usize;
        slice::from_raw_parts(start, length)
    }
}
