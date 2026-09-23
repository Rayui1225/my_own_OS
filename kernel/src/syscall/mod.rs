mod abi;
mod user_memory;

use crate::arch::riscv64::trap::TrapFrame;
use crate::{console, println, task};

pub(crate) use abi::Errno;
use abi::{encode_result, SyscallNumber, SyscallRequest};
#[cfg(feature = "test-kernel")]
pub(crate) use abi::{SYS_EXIT, SYS_GETPID, SYS_WRITE, SYS_YIELD};
#[cfg(feature = "test-kernel")]
pub(crate) use user_memory::SliceUserMemory;
use user_memory::{CurrentUserMemory, UserMemory};

const STDOUT: usize = 1;
const STDERR: usize = 2;

enum SyscallOutcome {
    Return(Result<usize, Errno>),
    Exit(i32),
}

pub fn handle(frame: &mut TrapFrame) {
    handle_with_memory(frame, &CurrentUserMemory);
}

pub(crate) fn handle_with_memory<M: UserMemory>(frame: &mut TrapFrame, memory: &M) {
    let request = SyscallRequest {
        number: frame.a7,
        args: [frame.a0, frame.a1, frame.a2, frame.a3, frame.a4, frame.a5],
    };

    // Returning to the same ecall would immediately invoke the syscall again.
    frame.advance_sepc_by_4();

    match dispatch(request, memory) {
        SyscallOutcome::Return(result) => frame.a0 = encode_result(result),
        SyscallOutcome::Exit(code) => {
            let pid = task::current_id()
                .expect("sys_exit called without a running task")
                .value();
            println!("[process] pid={} exited with code {}", pid, code);
            task::exit_current(code);
        }
    }
}

fn dispatch<M: UserMemory>(request: SyscallRequest, memory: &M) -> SyscallOutcome {
    let number = match SyscallNumber::try_from(request.number) {
        Ok(number) => number,
        Err(errno) => return SyscallOutcome::Return(Err(errno)),
    };

    match number {
        SyscallNumber::Write => SyscallOutcome::Return(sys_write(
            request.args[0],
            request.args[1],
            request.args[2],
            memory,
        )),
        SyscallNumber::Exit => SyscallOutcome::Exit(request.args[0] as i32),
        SyscallNumber::Yield => {
            task::yield_now();
            SyscallOutcome::Return(Ok(0))
        }
        SyscallNumber::GetPid => SyscallOutcome::Return(
            task::current_id()
                .map(|id| id.value())
                .ok_or(Errno::NoSuchProcess),
        ),
    }
}

fn sys_write<M: UserMemory>(
    fd: usize,
    buffer: usize,
    length: usize,
    memory: &M,
) -> Result<usize, Errno> {
    if fd != STDOUT && fd != STDERR {
        return Err(Errno::BadFileDescriptor);
    }
    buffer.checked_add(length).ok_or(Errno::Fault)?;

    let mut written = 0;
    while written < length {
        let byte = match memory.read_byte(buffer + written) {
            Ok(byte) => byte,
            Err(_) if written > 0 => return Ok(written),
            Err(_) => return Err(Errno::Fault),
        };
        console::write_byte(byte);
        written += 1;
    }

    Ok(written)
}
