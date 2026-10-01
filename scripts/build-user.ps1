$ErrorActionPreference = "Stop"

$repoRoot = Split-Path -Parent $PSScriptRoot
$userRoot = Join-Path $repoRoot "user\init"
$buildRoot = Join-Path $repoRoot "target\user-init"
$elfPath = Join-Path $buildRoot "riscv64gc-unknown-none-elf\debug\init"
$textPath = Join-Path $buildRoot "init.text.bin"
$dataPath = Join-Path $buildRoot "init.data.bin"
$outputDir = Join-Path $repoRoot "user\bin"
$outputPath = Join-Path $outputDir "init.sbin"
$llvmBin = Join-Path (rustc --print sysroot) "lib\rustlib\x86_64-pc-windows-msvc\bin"
$objcopy = Join-Path $llvmBin "llvm-objcopy.exe"
$nm = Join-Path $llvmBin "llvm-nm.exe"

New-Item -ItemType Directory -Force $outputDir | Out-Null
$oldEncodedRustFlags = $env:CARGO_ENCODED_RUSTFLAGS
$oldTargetDir = $env:CARGO_TARGET_DIR
try {
    $env:CARGO_ENCODED_RUSTFLAGS = "-Clink-arg=-T$($userRoot)\linker.ld"
    $env:CARGO_TARGET_DIR = $buildRoot
    cargo build --manifest-path (Join-Path $userRoot "Cargo.toml") --target riscv64gc-unknown-none-elf
    if ($LASTEXITCODE -ne 0) { throw "userspace cargo build failed" }
}
finally {
    $env:CARGO_ENCODED_RUSTFLAGS = $oldEncodedRustFlags
    $env:CARGO_TARGET_DIR = $oldTargetDir
}

& $objcopy -O binary --only-section=.text $elfPath $textPath
if ($LASTEXITCODE -ne 0) { throw "failed to extract userspace text section" }
& $objcopy -O binary --only-section=.data $elfPath $dataPath
if ($LASTEXITCODE -ne 0) { throw "failed to extract userspace data section" }

$symbols = & $nm --defined-only $elfPath
if ($LASTEXITCODE -ne 0) { throw "failed to read userspace symbols" }
function Find-Symbol([string]$name) {
    $line = $symbols | Where-Object { $_ -match "^[0-9a-fA-F]+\s+\w\s+$name$" } | Select-Object -First 1
    if (-not $line) { throw "missing symbol: $name" }
    return [Convert]::ToUInt64(($line -split "\s+")[0], 16)
}

$entry = [UInt64](Find-Symbol "_start")
$bssStart = Find-Symbol "__bss_start"
$bssEnd = Find-Symbol "__bss_end"
$bssSize = [UInt32]($bssEnd - $bssStart)
$text = [IO.File]::ReadAllBytes($textPath)
$data = [IO.File]::ReadAllBytes($dataPath)

$stream = [IO.File]::Open($outputPath, [IO.FileMode]::Create)
$writer = [IO.BinaryWriter]::new($stream)
try {
    $writer.Write([Text.Encoding]::ASCII.GetBytes("SBIN"))
    $writer.Write([UInt16]1)
    $writer.Write([UInt16]32)
    $writer.Write($entry)
    $writer.Write([UInt32]$text.Length)
    $writer.Write([UInt32]$data.Length)
    $writer.Write($bssSize)
    $writer.Write([UInt32]0)
    $writer.Write($text)
    $writer.Write($data)
}
finally {
    $writer.Dispose()
    $stream.Dispose()
}

Write-Host "Built $outputPath"
