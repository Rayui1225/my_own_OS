use core::fmt::{self, Write};

use crate::driver::uart;

pub fn init() {
    uart::init();
}

pub fn print(args: fmt::Arguments<'_>) {
    let _ = Console.write_fmt(args);
}

struct Console;

impl Write for Console {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        uart::write_bytes(s.as_bytes());
        Ok(())
    }
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {{
        $crate::console::print(core::format_args!($($arg)*));
    }};
}

#[macro_export]
macro_rules! println {
    () => {{
        $crate::print!("\n");
    }};
    ($fmt:expr) => {{
        $crate::print!(concat!($fmt, "\n"));
    }};
    ($fmt:expr, $($arg:tt)*) => {{
        $crate::print!(concat!($fmt, "\n"), $($arg)*);
    }};
}
