$ErrorActionPreference = "Stop"

$repoRoot = Split-Path -Parent $PSScriptRoot
$qemuDir = "C:\Program Files\qemu"
$kernelPath = Join-Path $repoRoot "target\riscv64gc-unknown-none-elf\debug\kernel"

$env:PATH = "$qemuDir;$env:PATH"

Push-Location $repoRoot
try {
    cargo build -p kernel --target riscv64gc-unknown-none-elf --features test-kernel

    qemu-system-riscv64 `
        -machine virt `
        -display none `
        -monitor none `
        -serial mon:stdio `
        -kernel $kernelPath
}
finally {
    Pop-Location
}

