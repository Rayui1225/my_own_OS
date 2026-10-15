# my_own_OS

Learning-focused RISC-V toy OS project built in Rust.

Current progress: Milestones 0-14, including a read-only SimpleFS on the RAM-backed block device,
per-process file descriptors, and loading `/bin/init` from the filesystem image.

Run the kernel or its QEMU test suite from PowerShell:

```powershell
.\scripts\run-kernel.ps1
.\scripts\run-tests.ps1
```

Both scripts rebuild `user/bin/init.sbin` and `fs/image/simplefs.img` before compiling the kernel.
Milestone 14 provides kernel-internal `open`/`read`/`close` and `readdir`; userspace file syscalls
and an interactive shell are not implemented yet.
