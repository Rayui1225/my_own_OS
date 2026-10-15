$ErrorActionPreference = "Stop"

$repoRoot = Split-Path -Parent $PSScriptRoot
$initPath = Join-Path $repoRoot "user\bin\init.sbin"
$motdPath = Join-Path $repoRoot "fs\content\motd.txt"
$outputDir = Join-Path $repoRoot "fs\image"
$outputPath = Join-Path $outputDir "simplefs.img"
$blockSize = 512
$initBytes = [IO.File]::ReadAllBytes($initPath)
$motdBytes = [IO.File]::ReadAllBytes($motdPath)
$initBlocks = [int][Math]::Ceiling($initBytes.Length / $blockSize)
$motdBlocks = [int][Math]::Ceiling($motdBytes.Length / $blockSize)
$initStart = 5
$motdStart = $initStart + $initBlocks
$totalBlocks = $motdStart + $motdBlocks
$diskBytes = [byte[]]::new($totalBlocks * $blockSize)

function Write-U32([int]$offset, [uint32]$value) {
    $encoded = [BitConverter]::GetBytes($value)
    [Buffer]::BlockCopy($encoded, 0, $diskBytes, $offset, 4)
}

function Write-Inode([int]$id, [byte]$kind, [int]$size, [int]$start, [int]$blocks) {
    $offset = $blockSize + $id * 32
    $diskBytes[$offset] = $kind
    Write-U32 ($offset + 4) $size
    Write-U32 ($offset + 8) $start
    Write-U32 ($offset + 12) $blocks
}

function Write-Entry([int]$block, [int]$slot, [int]$inode, [string]$name) {
    $nameBytes = [Text.Encoding]::ASCII.GetBytes($name)
    if ($nameBytes.Length -eq 0 -or $nameBytes.Length -gt 27) {
        throw "invalid SimpleFS name: $name"
    }
    $offset = $block * $blockSize + $slot * 32
    Write-U32 $offset $inode
    $diskBytes[$offset + 4] = [byte]$nameBytes.Length
    [Buffer]::BlockCopy($nameBytes, 0, $diskBytes, $offset + 5, $nameBytes.Length)
}

[Buffer]::BlockCopy([Text.Encoding]::ASCII.GetBytes("SFS1"), 0, $diskBytes, 0, 4)
Write-U32 4 1
Write-U32 8 $totalBlocks
Write-U32 12 5
Write-U32 16 0
Write-U32 20 1

Write-Inode 0 2 64 2 1
Write-Inode 1 2 32 3 1
Write-Inode 2 2 32 4 1
Write-Inode 3 1 $initBytes.Length $initStart $initBlocks
Write-Inode 4 1 $motdBytes.Length $motdStart $motdBlocks

Write-Entry 2 0 1 "bin"
Write-Entry 2 1 2 "etc"
Write-Entry 3 0 3 "init"
Write-Entry 4 0 4 "motd"

[Buffer]::BlockCopy($initBytes, 0, $diskBytes, $initStart * $blockSize, $initBytes.Length)
[Buffer]::BlockCopy($motdBytes, 0, $diskBytes, $motdStart * $blockSize, $motdBytes.Length)
New-Item -ItemType Directory -Force $outputDir | Out-Null
[IO.File]::WriteAllBytes($outputPath, $diskBytes)
Write-Host "Built $outputPath ($totalBlocks blocks)"
