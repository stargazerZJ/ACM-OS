# file target/riscv64gc-unknown-none-elf/debug/os
target remote :1234
layout split
b *0x80000000
c

define hook-quit
    set confirm off
end

