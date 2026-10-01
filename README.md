# my_own_OS

Learning-focused RISC-V toy OS project built in Rust.

Current progress: Milestones 0-12, through loading a Rust userspace program from the custom
SimpleBin format.

Run the kernel or its QEMU test suite from PowerShell:

```powershell
.\scripts\run-kernel.ps1
.\scripts\run-tests.ps1
```

Both scripts rebuild `user/bin/init.sbin` before compiling the kernel.
