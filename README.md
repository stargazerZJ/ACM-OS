# ACM OS 🦀

A toy operating system for RISC-V 64, built in Rust.

[Final Project](https://acore-guide.sjtu.app/) of SJTU CS 2952.

This repository also contains my solutions to the course labs. I initially planned to do the final project, but decided to complete the individual lab assignments instead.

---

## Completed Labs

Implementations for the following labs, mostly available as patches in the `/lab` directory.

[Lab introduction](https://github.com/peterzheng98/os-2024-tutorial)
[Common Environment Setup](lab/README.md)

### Lab 1.1: Acpi Viewer
Difficulty: 🟢🟢🟢⚪⚪
A UEFI application that displays ACPI tables.
[repo](https://github.com/stargazerZJ/ACM-Acpi-Viewer/tree/main)

### Lab 1.2: Acpi Hacker
Difficulty: 🟢🟢🟢⚪⚪
A UEFI application that modifies ACPI tables.
[repo](https://github.com/stargazerZJ/ACM-Acpi-Viewer/tree/acpi-hacker)

### Lab 1.3: UEFI Runtime Driver
Difficulty: 🟢🟢🟢🟢🟢
A UEFI runtime driver that provides a function for user space applications to execute.
[repo](https://github.com/stargazerZJ/ACM-Acpi-Viewer/tree/runtime-driver)
Kernel side integration: [0001-uefi-runtime-service.patch](lab/0001-uefi-runtime-service.patch)

### Lab 2.1: Key-Value Store Syscalls
Difficulty: 🟢🟢🟢⚪⚪
A simple in-kernel key-value store implemented using system calls.
[0001-add-kv-store-syscalls.patch](lab/0001-add-kv-store-syscalls.patch)

### Lab 2.2: vDSO
Difficulty: 🟢🟢🟢🟢🟢
A Virtual Dynamic Shared Object version of the `getpid` system call.
[0001-vdso.patch](lab/0001-vdso.patch)

### Lab 3.1: Using mmap
Difficulty: 🟢⚪⚪⚪⚪
An exercise to use `mmap` and related system calls.
[lab/mmap](lab/mmap)

### Lab 3.2: Persistent Ramfs
Difficulty: 🟢🟢🟢🟢⚪
Modification of the existing RAM filesystem to support persistence.
[0001-ramfs-persistent.patch](lab/0001-ramfs-persistent.patch)

### Lab 4.1: Inode and Xattr
Difficulty: 🟢⚪⚪⚪⚪
An exercise to modify extended attributes (xattr) in the inode structure.
[lab/inode-xattr](lab/inode-xattr)

### Lab 4.2: FUSE
Difficulty: 🟢🟢⚪⚪⚪
An interface of Large Language Model (LLM) to the filesystem, based on FUSE.
[repo](https://github.com/stargazerZJ/ACM-FUSE)

### Lab 5.1.1: Customize `tcpdump`
Difficulty: 🟢⚪⚪⚪⚪
Add a custom filter to `tcpdump` to skip the first `n` packets. Based on `tcpdump` version 4.99.5.
[0001-tcpdump-skip-n.patch](lab/0001-tcpdump-skip-n.patch)
Side note: I later found that the exact same functionality would be added to `tcpdump` in version 5.0.0, which was released as of the time of writing.

### Lab 5.1.2: Thread Socket Limit
Difficulty: 🟢🟢⚪⚪⚪
Implement a limit on the number of sockets that can be opened by a single thread.
[0001-thread-socket-limit.patch](lab/0001-thread-socket-limit.patch)

### Lab 5.2: NCCL
Difficulty: 🟢🟢🟢⚪⚪
An exercise to use the NVIDIA Collective Communications Library (NCCL) for multi-GPU communication.
[lab/nccl](lab/nccl)