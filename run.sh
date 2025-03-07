#!/bin/bash

# Define the path to your OS binary
OS_BINARY="target/riscv64gc-unknown-none-elf/debug/os"

# Start QEMU with the specified parameters
qemu-system-riscv64 \
    -machine virt \
    -nographic \
    -bios none \
    -device loader,file="$OS_BINARY" \
    -s -S &

# Capture the QEMU process ID
QEMU_PID=$!

## Give QEMU a moment to start
#sleep 1

# Start GDB and connect to QEMU
gdb-multiarch \
    -ex "file $OS_BINARY" \
    -q \
    -x "./gdbinit"

# After GDB exits, terminate QEMU
if ps -p $QEMU_PID > /dev/null; then
  kill $QEMU_PID
fi