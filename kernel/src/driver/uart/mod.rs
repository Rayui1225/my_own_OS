use core::ptr::write_volatile;

// QEMU virt machine exposes a 16550-compatible UART at this MMIO base address.
const UART_BASE: usize = 0x1000_0000;

// THR: Transmit Holding Register. Writing a byte here sends it out to serial.
const THR: usize = 0;
// IER: Interrupt Enable Register. Controls which UART interrupts are enabled.
const IER: usize = 1;
// FCR: FIFO Control Register. Controls the UART's internal transmit/receive queues.
const FCR: usize = 2;
// LCR: Line Control Register. Configures word length, parity, stop bits, and DLAB.
const LCR: usize = 3;
// MCR: Modem Control Register. Carries extra control bits like DTR/RTS.
const MCR: usize = 4;
// DLL/DLM: Divisor Latch Low/High. Together they store the baud-rate divisor.
const DLL: usize = 0;
const DLM: usize = 1;

// DLAB: Divisor Latch Access Bit. When set, offsets 0/1 become DLL/DLM.
const LCR_DLAB: u8 = 1 << 7;
// 8N1: 8 data bits, No parity, 1 stop bit. A very common serial configuration.
const LCR_8N1: u8 = 0x03;
const FIFO_ENABLE_AND_CLEAR: u8 = 0x07;
const MODEM_READY: u8 = 0x03;

pub fn init() {
    disable_interrupts();
    configure_baud_rate_divisor(1);
    configure_line_control_8n1();
    enable_and_clear_fifos();
    set_modem_ready();
}

pub fn write_byte(byte: u8) {
    if byte == b'\n' {
        write_raw_byte(b'\r');
    }

    write_raw_byte(byte);
}

pub fn write_bytes(bytes: &[u8]) {
    for &byte in bytes {
        write_byte(byte);
    }
}

fn write_raw_byte(byte: u8) {
    write_reg(THR, byte);
}

fn disable_interrupts() {
    // We use polling-style output for now, so UART interrupts stay disabled.
    write_reg(IER, 0x00);
}

fn configure_baud_rate_divisor(divisor: u16) {
    // Set DLAB so offsets 0/1 temporarily mean DLL/DLM instead of THR/IER.
    write_reg(LCR, LCR_DLAB);
    write_reg(DLL, divisor as u8);
    write_reg(DLM, (divisor >> 8) as u8);
}

fn configure_line_control_8n1() {
    // Switch back to normal line-control mode and select the common 8N1 format.
    write_reg(LCR, LCR_8N1);
}

fn enable_and_clear_fifos() {
    // Turn on the FIFO buffers and clear any stale bytes left in them.
    write_reg(FCR, FIFO_ENABLE_AND_CLEAR);
}

fn set_modem_ready() {
    // DTR/RTS are traditional "ready" control lines on UART-style serial ports.
    write_reg(MCR, MODEM_READY);
}

fn write_reg(offset: usize, value: u8) {
    unsafe {
        write_volatile((UART_BASE + offset) as *mut u8, value);
    }
}
