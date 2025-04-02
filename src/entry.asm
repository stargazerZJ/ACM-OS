    .section .text.entry
    .globl _start
_start:
    la sp, boot_stack_top
    # call rust_main
    call setup_machine_mode
    j .

    .globl setup_machine_mode
setup_machine_mode:
    # 1. Set mstatus register: MPP field to S-mode (1)
    #    MPP (Machine Previous Privilege) bits are typically [12:11].
    #    S-mode = 01 binary.
    #    We need to clear the current MPP bits and then set them to 01.
    #    Mask for MPP bits = (3 << 11) = 0x1800
    #    Value for S-mode = (1 << 11) = 0x800

    li t0, 0x1800          # Load mask for MPP bits
    csrc mstatus, t0       # Clear current MPP bits in mstatus
    li t0, 0x800           # Load value for S-mode (01) in MPP position
    csrs mstatus, t0       # Set MPP bits to S-mode in mstatus

    # 2. Set mepc register to the S-mode entry point
    #    'mepc' holds the return address for 'mret'.
    #    We set it to where we want S-mode execution to begin.

    la t0, rust_main           # Load the address of the S-mode entry point into t0
    csrw mepc, t0              # Write the address to mepc

    # 3. Temporarily disable page tables (virtual memory)
    #    Set satp (Supervisor Address Translation and Protection) register to 0.
    #    This selects Bare mode (no translation).

    csrw satp, zero            # Write 0 to satp (zero is the hardwired zero register)

    # 4. Enable delegation of interrupts and exceptions to S-mode
    #    Set medeleg (Machine Exception Delegation) and
    #    mideleg (Machine Interrupt Delegation) registers to 0xffff.
    #    This delegates exceptions 0-15 and interrupts 0-15 to be handled
    #    directly in S-mode, if possible.
    #    Note: For full delegation on RV64, you might use -1 (0xffffffffffffffff).
    #          Using 0xffff as explicitly requested.

    li t0, 0xffff
    csrw medeleg, t0           # Delegate exceptions 0-15 to S-mode
    csrw mideleg, t0           # Delegate interrupts 0-15 to S-mode

    # 5. Enable S-mode interrupts in 'sie'
    #    Set SEIE (Supervisor External Interrupt Enable),
    #    STIE (Supervisor Timer Interrupt Enable), and
    #    SSIE (Supervisor Software Interrupt Enable) bits in the 'sie' register.
    #    These enablements will take effect once the core enters S-mode.
    #    SEIE = bit 9
    #    STIE = bit 5
    #    SSIE = bit 1
    #    Mask = (1 << 9) | (1 << 5) | (1 << 1) = 0x200 | 0x20 | 0x2 = 0x222

    li t0, 0x222
    csrs sie, t0               # Set SEIE, STIE, SSIE bits in sie

    # 6. Configure Physical Memory Protection (PMP) entry 0
    #    Allow full Read/Write/Execute access across a large range.
    #    pmpaddr0 defines the region boundary/encoding.
    #    pmpcfg0 defines the permissions and matching mode (A field).
    #    Setting pmpaddr0 = 0x3fffffffffffff and A=NAPOT (Naturally Aligned Power-of-Two)
    #    covers the address range 0 to 2^55 - 1 (assuming base address is 0).
    #    Setting pmpcfg0 = 0xf:
    #      bit 0 (R) = 1 (Read)
    #      bit 1 (W) = 1 (Write)
    #      bit 2 (X) = 1 (Execute)
    #      bits[4:3](A) = 11 (NAPOT)
    #      Result: 0b00001111 = 0xf (Permissions for pmp entry 0)
    #    This effectively grants S-mode access to the lower 32 Petabytes of physical memory.

    li t0, 0x3fffffffffffff    # Load the PMP address boundary/encoding
    csrw pmpaddr0, t0          # Write to pmpaddr0

    li t0, 0xf                 # Load the configuration value (R=1,W=1,X=1, A=NAPOT)
    csrw pmpcfg0, t0           # Write to pmpcfg0 (configures PMP entry 0)

    # 7. Put hartid in a0
    csrr a0, mhartid

    # M-mode setup is complete.
    mret

    .section .bss.stack
    .globl boot_stack_lower_bound
boot_stack_lower_bound:
    .space 4096 * 16
    .globl boot_stack_top
boot_stack_top:
    .section .bss.heap
    .globl kernel_heap_beg
kernel_heap_beg:
    .space 4096 * 768   # 3 MB heap
    .globl kernel_heap_end
kernel_heap_end:
