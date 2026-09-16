pub const SYS_WRITE: usize = 1;
pub const SYS_EXIT: usize = 2;
pub const SYS_YIELD: usize = 3;
pub const SYS_GETPID: usize = 4;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SyscallNumber {
    Write,
    Exit,
    Yield,
    GetPid,
}

impl TryFrom<usize> for SyscallNumber {
    type Error = Errno;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        match value {
            SYS_WRITE => Ok(Self::Write),
            SYS_EXIT => Ok(Self::Exit),
            SYS_YIELD => Ok(Self::Yield),
            SYS_GETPID => Ok(Self::GetPid),
            _ => Err(Errno::NoSys),
        }
    }
}

#[repr(isize)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Errno {
    NoSuchProcess = 3,
    BadFileDescriptor = 9,
    Fault = 14,
    NoSys = 38,
}

pub struct SyscallRequest {
    pub number: usize,
    pub args: [usize; 6],
}

pub fn encode_result(result: Result<usize, Errno>) -> usize {
    match result {
        Ok(value) => value,
        Err(errno) => (-(errno as isize)) as usize,
    }
}
