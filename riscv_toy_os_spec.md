# RISC-V Toy OS 練習規格書

## 1. 專案定位

本專案是一個以 **RISC-V** 為目標架構的教育用 Toy OS。目標不是做出可取代 Linux 的完整作業系統，而是透過實作一個最小但完整的 OS，理解作業系統從 boot 到 userspace 的核心流程。

核心學習路線：

```text
boot
  -> kernel init
  -> console / logging
  -> trap / interrupt
  -> physical memory
  -> virtual memory
  -> heap allocation
  -> task / process
  -> syscall
  -> block device
  -> filesystem
  -> userspace shell
```

完成後，這個 OS 至少要能：

1. 在 QEMU RISC-V virt machine 上啟動。
2. 印出 kernel log。
3. 處理 trap / exception / interrupt。
4. 啟用 virtual memory。
5. 支援 kernel heap allocation。
6. 建立簡單 task / process abstraction。
7. 透過 syscall 從 userspace 呼叫 kernel。
8. 從簡單 filesystem 載入 userspace program。
9. 跑一個最小 shell。

---

## 2. 技術選型

### 2.1 語言

建議使用：

```text
Rust
```

原因：

- 適合寫 bare-metal kernel。
- 可以使用 `#![no_std]` 建立不依賴標準函式庫的 kernel。
- ownership / type system 有助於控制 unsafe 範圍。
- 適合練習「unsafe boundary 應該放在哪裡」。

### 2.2 目標架構

固定使用：

```text
RISC-V 64-bit
```

建議 target：

```text
riscv64gc-unknown-none-elf
```

建議模擬器：

```text
QEMU virt machine
```

理由：

- RISC-V ISA 相對乾淨。
- privilege mode、trap、MMU、page table 概念清楚。
- 很適合學 OS 的本質，而不是被 x86 legacy 細節分散注意力。

### 2.3 開發原則

本專案應該遵守以下原則：

1. 每個 milestone 都要能在 QEMU 觀察結果。
2. 每個核心模組都應該盡量有測試或可驗收輸出。
3. 先做最小可用版本，再考慮最佳化。
4. unsafe code 要集中在少數底層模組。
5. 不要太早做過度抽象。
6. 優先完成完整 OS skeleton，再回頭優化細節。

---

## 3. 可優化事項 / 延伸挑戰

以下內容不是第一階段必做項目，而是完成核心 OS 後可以視興趣追加。

### 3.1 POSIX 相容

第一階段不要求 POSIX 相容。

可優化方向：

- 實作更多 Linux-like syscall。
- 支援類似 file descriptor 的抽象。
- 支援 process exit status。
- 支援 pipe。
- 支援 basic signal model。

### 3.2 多使用者模型

第一階段不需要 user / group / permission。

可優化方向：

- 加入 UID / GID。
- filesystem permission。
- process owner。
- syscall permission check。

### 3.3 完整 ELF Loader

第一階段可以先用自定義簡單 binary format。

可優化方向：

- parse ELF header。
- parse program header。
- 支援 loadable segment。
- 支援 entry point。
- 支援 user stack 初始化。
- 支援 relocation。
- 支援 dynamic linking，若想挑戰更高難度。

### 3.4 Networking

第一階段不需要 networking。

可優化方向：

- virtio-net driver。
- Ethernet frame parsing。
- ARP。
- IPv4。
- ICMP ping。
- UDP。
- TCP，難度較高。

### 3.5 SMP / Multi-core

第一階段只支援 single hart / single core。

可優化方向：

- 多 hart 啟動。
- per-core data。
- spinlock。
- cross-core interrupt。
- multi-core scheduler。
- work stealing scheduler。

### 3.6 真實硬體支援

第一階段只要求 QEMU。

可優化方向：

- 支援實際 RISC-V board。
- 處理不同 device tree。
- 真實 UART driver。
- 真實 storage driver。
- bootloader 適配。

### 3.7 完整安全模型

第一階段只做最小 userspace isolation。

可優化方向：

- syscall argument validation。
- copy_from_user / copy_to_user。
- kernel / user address space isolation。
- W^X memory policy。
- ASLR。
- capability-based security。

### 3.8 效能最佳化

第一階段不追求效能。

可優化方向：

- page table cache。
- slab allocator。
- buddy allocator。
- scheduler optimization。
- zero-copy IO。
- page cache。
- filesystem cache。

---

## 4. 系統架構總覽

建議 repo 結構：

```text
riscv-toy-os/
  kernel/
    src/
      arch/
        riscv64/
          boot/
          trap/
          interrupt/
          mmu/
          context_switch/
      console/
      memory/
        frame_allocator/
        page_table/
        heap_allocator/
        address_space/
      task/
        scheduler/
        process/
        thread/
      syscall/
      driver/
        uart/
        timer/
        block/
      fs/
        vfs/
        simplefs/
      loader/
      test/
  user/
    init/
    shell/
    ls/
    cat/
  tools/
    image_builder/
  docs/
```

---

## 5. Kernel 模組責任

### 5.1 `arch/riscv64`

負責所有 RISC-V 架構相關邏輯。

包含：

- boot entry。
- linker script。
- trap vector。
- CSR 操作。
- context switch。
- MMU 啟用。
- privilege mode 切換。

此模組會包含最多 unsafe code，應該明確封裝。

---

### 5.2 `console`

負責 kernel 輸出。

包含：

- UART write。
- `print!` / `println!`。
- kernel log macro。
- log level。

---

### 5.3 `memory`

負責記憶體管理。

包含：

- physical frame allocator。
- page table abstraction。
- virtual memory mapping。
- kernel heap。
- process address space。

---

### 5.4 `task`

負責 task / process 管理。

包含：

- task struct。
- process struct。
- task state。
- kernel stack。
- user stack。
- context switch。
- scheduler。

---

### 5.5 `syscall`

負責 userspace 和 kernel 的邊界。

包含：

- syscall number 定義。
- syscall dispatcher。
- syscall argument validation。
- syscall return value。

---

### 5.6 `driver`

負責裝置驅動。

初期包含：

- UART。
- timer。
- ramdisk 或 virtio-blk。

---

### 5.7 `fs`

負責 filesystem。

包含：

- VFS-like abstraction。
- simple filesystem。
- directory entry。
- file read。
- path lookup。

---

### 5.8 `loader`

負責載入 userspace program。

初期可以支援自定義 simple binary format。

進階再支援 ELF。

---

## 6. Milestone 規格

---

## Milestone 0：Bootable Freestanding Kernel

### 目標

建立一個可以被 QEMU 啟動的 RISC-V kernel。

### 功能需求

- 使用 `#![no_std]`。
- 使用 `#![no_main]`。
- 自訂 panic handler。
- 建立 linker script。
- 建立 kernel entry point。
- QEMU 啟動後能進入 Rust kernel main function。

### 驗收標準

QEMU 啟動後可以看到：

```text
[boot] kernel entered
[boot] arch = riscv64
```

### Agent 討論重點

- entry point 要用 assembly 還是 Rust naked function？
- stack 在哪裡初始化？
- linker script 要如何安排 kernel sections？
- QEMU 啟動參數如何設計？

---

## Milestone 1：Console / UART Output

### 目標

讓 kernel 可以輸出 debug log。

### 功能需求

- 初始化 UART。
- 實作 blocking write。
- 支援 `print!` / `println!`。
- 支援簡單 log macro。

### 驗收標準

```text
[info] kernel initialized
[debug] uart ready
```

### Agent 討論重點

- UART MMIO address 從哪裡取得？
- 要不要先 hardcode QEMU virt UART address？
- console API 要不要跟底層 UART driver 分離？

---

## Milestone 2：Panic / Assertion / Test Harness

### 目標

建立可測試的 kernel 開發流程。

### 功能需求

- panic 時輸出 panic message。
- 支援 QEMU exit。
- 支援 basic kernel tests。
- 測試可以自動判斷成功或失敗。

### 驗收標準

```text
running 3 tests
console_print ... ok
panic_handler ... ok
frame_allocator_basic ... ok
```

### Agent 討論重點

- 如何讓 QEMU test 自動結束？
- 測試輸出走 UART 還是特殊 device？
- panic test 如何判斷成功？

---

## Milestone 3：Trap / Exception Handling

### 目標

讓 kernel 可以接住 CPU trap。

### 功能需求

- 設定 `stvec` 或 `mtvec`。
- 建立 trap entry。
- 保存 registers。
- 建立 TrapFrame。
- 根據 cause 分派 handler。
- 支援 unknown trap panic。

### 最小支援 trap

```text
illegal instruction
breakpoint
load page fault
store page fault
ecall from user mode
timer interrupt
```

### 驗收標準

手動觸發錯誤時，kernel 可以印出：

```text
[trap] cause = load page fault
[trap] fault address = 0x0
```

### Agent 討論重點

- trap handler 要跑在 machine mode 還是 supervisor mode？
- TrapFrame layout 要怎麼設計？
- 哪些 CSR 需要讀寫？
- trap entry assembly 和 Rust handler 的邊界怎麼切？

---

## Milestone 4：Timer Interrupt

### 目標

讓 kernel 有時間概念，為 scheduler 做準備。

### 功能需求

- 初始化 timer。
- 每隔固定時間觸發 interrupt。
- 維護 global tick counter。
- 每次 timer interrupt 後設定下一次 interrupt。

### 驗收標準

```text
[timer] tick = 1
[timer] tick = 2
[timer] tick = 3
```

### Agent 討論重點

- QEMU virt 上 timer 透過哪個機制觸發？
- 使用 CLINT 還是 SBI？
- timer interrupt 要先只印 log，還是直接接 scheduler？

---

## Milestone 5：Physical Memory Manager

### 目標

管理 physical frames。

### 功能需求

- 取得可用 physical memory range。
- 以 page / frame 為單位管理 memory。
- 實作：

```text
alloc_frame()
dealloc_frame()
```

- 初期可以使用 bitmap allocator 或 stack allocator。

### 驗收標準

```text
[memory] total usable frames = 32768
[memory] alloc frame = 0x80200000
[memory] free frame = 0x80200000
```

### Agent 討論重點

- memory map 從 device tree 讀，還是先 hardcode？
- kernel image 佔用的區間如何排除？
- allocator metadata 放在哪？

---

## Milestone 6：Virtual Memory / Paging / MMU

### 目標

啟用 virtual memory。

### 功能需求

- 建立 kernel page table。
- map kernel code / data / rodata / bss / stack。
- 啟用 MMU。
- 支援 page mapping API：

```text
map_page(virt, phys, flags)
unmap_page(virt)
translate_addr(virt)
```

- page fault 時印出 fault address。

### Mapping 策略

第一階段建議使用：

```text
identity mapping
```

進階可改成：

```text
higher-half kernel mapping
```

### 驗收標準

```text
[vm] kernel page table initialized
[vm] mmu enabled
[vm] translate 0x80200000 -> 0x80200000
```

### Agent 討論重點

- RISC-V Sv39 還是 Sv48？
- page table entry flags 怎麼設計？
- identity mapping 是否足夠支撐初期 kernel？
- MMU enable 前後 virtual address 是否一致？

---

## Milestone 7：Kernel Heap Allocation

### 目標

讓 kernel 可以使用動態配置。

### 功能需求

- 定義 kernel heap virtual region。
- 為 heap map physical frames。
- 實作 global allocator。
- 支援 Rust `alloc` crate。
- 至少支援：

```text
Box
Vec
String
```

### 初期 allocator

```text
bump allocator
```

### 可優化 allocator

```text
linked-list allocator
fixed-size block allocator
buddy allocator
slab allocator
```

### 驗收標準

```rust
let mut v = Vec::new();
v.push(1);
v.push(2);
assert_eq!(v.len(), 2);
```

### Agent 討論重點

- heap region 要放在哪個 virtual address？
- allocator out-of-memory 時怎麼處理？
- global allocator 的 unsafe boundary 怎麼包？

---

## Milestone 8：Kernel Task / Context Switch

### 目標

建立最小 task abstraction。

### 功能需求

定義 Task：

```text
Task
  id
  state
  context
  kernel_stack
```

Task state：

```text
Ready
Running
Blocked
Exited
```

支援：

- 建立 kernel task。
- 保存 context。
- 恢復 context。
- 手動 yield。
- cooperative scheduling。

### 驗收標準

```text
[task 1] hello
[task 2] hello
[task 1] hello again
[task 2] hello again
```

### Agent 討論重點

- context 裡要存哪些 register？
- context switch assembly 怎麼寫？
- task stack 如何初始化？
- scheduler queue 用 VecDeque 還是 intrusive list？

---

## Milestone 9：Preemptive Scheduler

### 目標

讓 timer interrupt 觸發 task switch。

### 功能需求

- timer interrupt 呼叫 scheduler tick。
- running task 時間片用完後切換。
- round-robin scheduling。
- task 可以 yield。
- task 可以 exit。

### 驗收標準

```text
[scheduler] switch task 1 -> task 2
[scheduler] switch task 2 -> task 3
[scheduler] switch task 3 -> task 1
```

### Agent 討論重點

- interrupt 中可不可以直接 context switch？
- scheduler lock 怎麼處理？
- task state transition 怎麼設計？

---

## Milestone 10：Syscall Interface

### 目標

讓 userspace 可以透過 syscall 呼叫 kernel。

### 功能需求

支援最小 syscall set：

```text
sys_write(fd, buf, len)
sys_exit(code)
sys_yield()
sys_getpid()
```

Syscall flow：

```text
userspace program
  -> ecall
  -> trap handler
  -> syscall dispatcher
  -> kernel service
  -> return to userspace
```

### 驗收標準

Userspace 呼叫：

```text
write(1, "hello from user\n")
exit(0)
```

Kernel 輸出：

```text
[user] hello from user
[process] pid=1 exited with code 0
```

### Agent 討論重點

- syscall number 放在哪個 register？
- argument 放哪些 register？
- return value 放哪個 register？
- userspace pointer 如何檢查？

---

## Milestone 11：User Address Space / User Mode

### 目標

讓 process 有自己的 user virtual memory，並能切到 user mode 執行。

### 功能需求

每個 process 擁有：

```text
user code region
user data region
user stack
trap frame
page table
```

支援：

- 建立 user page table。
- map user program。
- map user stack。
- 設定 user entry point。
- 切換到 user mode。
- user trap 回 kernel。
- process exit 後釋放資源。

### 驗收標準

```text
[process] loading init
[process] switch to user mode
[user] hello
[process] init exited
```

### Agent 討論重點

- kernel 和 user address space 要不要共用 high mapping？
- user stack 放哪裡？
- 從 kernel return to user 的 CSR 要怎麼設定？
- process 結束後 page table 如何釋放？

---

## Milestone 12：Program Loader

### 目標

讓 kernel 可以載入 userspace program。

### 初期格式

先使用自定義 SimpleBin：

```text
SimpleBin
  magic
  entry_point
  text_size
  data_size
  text
  data
```

### 功能需求

- parse SimpleBin。
- 建立 user address space。
- map text segment。
- map data segment。
- 建立 user stack。
- 回傳 process entry point。

### 驗收標準

```text
[loader] load /bin/init
[loader] entry = 0x10000
[loader] user stack = 0x80000000
```

### Agent 討論重點

- SimpleBin 格式要怎麼設計才容易產生？
- user program build pipeline 怎麼做？
- loader 是否要直接讀 filesystem，還是吃 bytes？

---

## Milestone 13：Block Device Driver

### 目標

讓 OS 可以讀寫 block device。

### 初期建議

先使用：

```text
ramdisk
```

之後再挑戰：

```text
virtio-blk
```

### 功能需求

提供 block-level API：

```text
read_block(block_id, buffer)
write_block(block_id, buffer)
```

### 驗收標準

```text
[block] read block 0
[block] write block 1
[block] verify ok
```

### Agent 討論重點

- block size 固定多少？
- ramdisk image 如何嵌入 kernel 或掛到 QEMU？
- block driver 和 filesystem API 怎麼分層？

---

## Milestone 14：Simple Filesystem

### 目標

實作一個最小 filesystem。

### 功能需求

支援：

```text
open(path)
read(fd, buf)
close(fd)
readdir(path)
```

Filesystem 格式可極簡：

```text
superblock
inode table
data blocks
directory entries
```

需要支援以下檔案：

```text
/bin/init
/bin/shell
/bin/ls
/etc/motd
```

### 驗收標準

```text
fs> ls /
bin
etc

fs> ls /bin
init
shell
ls
```

### Agent 討論重點

- 要不要先做 read-only filesystem？
- path lookup 怎麼做？
- fd table 是 process-level 還是 global？
- directory entry 格式怎麼設計？

---

## Milestone 15：Userspace Shell

### 目標

做出一個可以互動的 userspace shell。

### 功能需求

Shell 支援：

```text
help
ls
cat
echo
pwd
cd
exit
```

### 驗收標準

```text
$ help
$ ls /
bin etc
$ cat /etc/motd
Welcome to toy-os
```

### Agent 討論重點

- shell input 從 UART 讀還是 console driver？
- `cd` 的 current working directory 放在哪？
- command parsing 要多簡單？
- shell command 是 built-in 還是 external program？

---

## 7. 建議開發順序

建議順序如下：

```text
1. bootable kernel
2. UART console
3. panic / test harness
4. trap / exception
5. timer interrupt
6. physical frame allocator
7. virtual memory
8. kernel heap allocator
9. kernel task / context switch
10. preemptive scheduler
11. syscall
12. user address space
13. program loader
14. block device
15. simple filesystem
16. userspace shell
```

不要一開始就碰 filesystem 或 scheduler 最終型態。

原因：

- 沒有 console，debug 會非常痛苦。
- 沒有 trap，syscall 和 page fault 都做不了。
- 沒有 physical memory manager，virtual memory 做不起來。
- 沒有 virtual memory，user process 沒有隔離。
- 沒有 heap，kernel data structure 寫起來會很卡。
- 沒有 syscall，userspace 沒有意義。
- 沒有 loader 和 filesystem，program 只能 hardcode。

---

## 8. 每個 Milestone 的通用完成定義

每個 milestone 完成時，至少應該回答以下問題：

1. 這個 milestone 新增了哪些 module？
2. 每個 module 的責任是什麼？
3. 對外 public API 是什麼？
4. unsafe code 集中在哪裡？
5. QEMU 上如何觀察結果？
6. 是否有測試？
7. 失敗時 kernel 如何回報？
8. 下一個 milestone 是否可以依賴這個功能？

---

## 9. Agent 討論用 Prompt 範本

每次要開始一個 milestone，可以先丟這段給 agent：

```text
你現在是 RISC-V OS kernel engineer。

目前我要做 RISC-V Toy OS 的 Milestone X：<名稱>。

請你先不要寫 code。
請先根據目前 repo 結構，提出：

1. 需要新增哪些 module
2. 每個 module 的責任
3. public API 設計
4. unsafe boundary 應該放在哪裡
5. 需要哪些測試
6. QEMU 上的最小可驗收行為是什麼
7. 這個 milestone 不應該碰哪些範圍，避免 scope creep

限制：

- 不要一次實作超過這個 milestone
- 不要引入不必要 abstraction
- 優先讓 QEMU 可觀察、可測試
- 如果有多種設計，請列出 trade-off，但最後選一個最小可行方案
```

---

## 10. 建議的完成等級

### Level 1：Kernel Skeleton

完成：

```text
boot
console
panic
test harness
trap
timer
physical frame allocator
```

代表你已經理解 kernel 如何啟動、輸出、處理 CPU 事件，以及管理最基礎的實體記憶體。

---

### Level 2：Memory + Scheduler Kernel

完成：

```text
virtual memory
kernel heap
task abstraction
context switch
preemptive scheduler
```

代表你已經理解 OS 如何管理 memory 和 CPU execution。

---

### Level 3：Userspace OS

完成：

```text
syscall
user address space
program loader
block device
filesystem
shell
```

代表你已經做出一個有完整 OS 形狀的系統。

---

### Level 4：可優化版本

可以選擇性加入：

```text
ELF loader
virtio-blk
virtio-net
read-write filesystem
multi-core scheduler
POSIX-like syscall
page cache
slab allocator
real hardware support
```

這些不是第一階段目標，而是當核心 OS skeleton 穩定後再做的挑戰。

---

## 11. 最終建議版本

本專案第一版建議採用以下範圍：

```text
Language: Rust
Arch: RISC-V 64-bit
Emulator: QEMU virt
Privilege: supervisor mode preferred
Memory model: Sv39
Kernel mapping: identity mapping first
Filesystem: read-only simplefs first
Loader: custom SimpleBin first
Scheduler: cooperative first, preemptive later
Block device: ramdisk first, virtio-blk later
Allocator: bump allocator first, linked-list or slab later
```

最重要的策略是：

```text
先讓 OS 從 boot 到 shell 跑通，再回頭優化每一層。
```

