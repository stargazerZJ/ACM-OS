#!/usr/bin/env bash

script_dir=$(dirname "$(realpath "$0")")
project_root=$(realpath "$script_dir/..")
initram_dir="$project_root/kvm/busybox-1.36.1/_install"
test_dir="$project_root/kernel/tests/out"

# copy test executables to initramfs
cp -r "$test_dir"/* "$initram_dir/bin/"
cp $project_root/kernel/mylinux/drivers/firmware/efi/my_runtime_service.ko "$initram_dir"

# make initramfs
cd "$initram_dir" || exit 1
find . -print0 | cpio --null -ov --format=newc 2>/dev/null | gzip -9 > ../initramfs.cpio.gz

# copy kernel and initramfs to EFI
cd "$project_root" || exit 1
cp kernel/mylinux/arch/x86/boot/bzImage kernel/uefi/bzImage
cp kvm/busybox-1.36.1/initramfs.cpio.gz kernel/uefi/initramfs.cpio.gz

# run qemu
cd "$project_root" || exit 1
qemu-system-x86_64 \
    -machine q35 \
    -m 2G \
    -drive if=pflash,format=raw,unit=0,file="/usr/share/OVMF/OVMF_CODE_4M.fd",readonly=on \
    -drive if=pflash,format=raw,unit=1,file="/usr/share/OVMF/OVMF_VARS_4M.fd" \
    -drive file=fat:rw:"kernel/uefi",format=raw,if=ide,index=0 \
    -nographic \
    -serial mon:stdio