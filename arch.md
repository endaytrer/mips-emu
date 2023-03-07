# MIPS Emulator: Additional Architecture Guidance

Daniel Gu

### 1. Bus Addressing:

The modified MIPS emulator uses 32bit addressing space for DRAM, ROM and IO.

```

Physical:                          Virtual (kernal) :
             ___________________                ___________________
 0xffffffff |                   |   0xffffffff |                   |
     .      |    Read-only      |              |  Physical Memory  |
     .      |   Memory (ROM)    |              |     Mapping       |
     .      |      (4KiB)       |   0x80000000 |___________________|    
 0xfffff000 |___________________|              |                   |
     .      |                   |              |       Stack       |
     .      |      VirtIO       |              |        🡓         |
     .      |      Device       |              |        🡑         |
     .      |      (4KiB)       |              |       Heap        |
 0xffffe000 |___________________|   0x10008000 |___________________|
     .      |                   |              |                   |
     .      |       UART        |              |       Data        |
     .      |      Device       |              |     (32kiB)       |
     .      |      (256B)       |   0x10000000 |___________________|
 0xffffd000 |___________________|              |                   |
     .      |                   |              |       Text        |
     .      |   Coprocessor 0   |              |     (252MiB)      |
     .      |      (32B)        |   0x00400000 |___________________|
 0xffffc000 |___________________|   0x00002100 |___________________|
 0x80000000 |___________________|              |                   |
     .      |                   |              |       UART        |
     .      |                   |              |      Device       |
     .      |                   |              |      (256B)       |
     .      |                   |   0x00002000 |___________________|
     .      |                   |              |                   |
     .      |   Random Access   |              |      VirtIO       |
     .      |    Memory (RAM)   |              |      Device       |
     .      |     (<= 2Gib)     |              |      (4kiB)       |
     .      |                   |   0x00001000 |___________________|
     .      |                   |              |                   |
     .      |                   |              |    Read-only      |
     .      |                   |              |   Memory (ROM)    |
     .      |                   |              |      (4KiB)       |
 0x00000000 |___________________|   0x00000000 |___________________|
```



### 2. Interrupt Vectors

On boot, Program counter is loaded in **0x00000000**

| Address       | Exception                                |
| ------------- | ---------------------------------------- |
| 0x0ffffffc(v) | Syscall (Software Interrupt)             |
| 0x00000000(v) | Boot, Reset, NMI                         |




### 3. Paging

Coprocessor 0 holds the PTBASE register(Register 4). which is a Page Table Entry.

The PTE is organized in this way:

| [31:12]                     | [11:7] | [6]  | [5]     | [4]   | [3]  | [2]  | [1]   | [0]   |
| --------------------------- | ------ | ---- | ------- | ----- | ---- | ---- | ----- | ----- |
| Physical Frame Number (PFN) | Unset  | Huge | Present | Valid | User | Read | Write | Dirty |
