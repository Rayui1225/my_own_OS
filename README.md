# my_own_OS

Learning-focused RISC-V toy OS project built in Rust.

Current progress: Milestones 0-13, through a safe, memory-backed block device behind a reusable
block I/O interface.

Run the kernel or its QEMU test suite from PowerShell:

```powershell
.\scripts\run-kernel.ps1
.\scripts\run-tests.ps1
```

Both scripts rebuild `user/bin/init.sbin` before compiling the kernel.
