#!/usr/bin/env bash

script_dir=$(dirname "$(realpath "$0")")
project_root=$(realpath "$script_dir/..")
initram_dir="$project_root/kvm/busybox-1.36.1/_install"
test_dir="$project_root/kernel/tests/out"

# copy test executables to initramfs
cp -r "$test_dir"/* "$initram_dir/bin/"

# make initramfs
cd "$initram_dir" || exit 1
find . -print0 | cpio --null -ov --format=newc 2>/dev/null | gzip -9 > ../initramfs.cpio.gz

# run qemu
cd "$project_root" || exit 1
qemu-system-x86_64 \
    -kernel ./kernel/mylinux/arch/x86/boot/bzImage \
    -append "init=/init console=ttyS0" \
    -initrd ./kvm/busybox-1.36.1/initramfs.cpio.gz \
    -nographic \
    -m 1G