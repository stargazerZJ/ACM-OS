# Common Environment Setup for OS Lab

See [official Lab Introduction](https://github.com/peterzheng98/os-2024-tutorial) for more details.

The Author is using Ubuntu 24.04 LTS with QEMU 8.2.2 and Linux kernel 5.19.17.

### Expected Directory Structure

```bash
.
├── kernel
│   ├── mylinux             # Kernel source directory
│   │   └── some-feature.patch
│   ├── tests               # Test directory for the kernel
│   │   ├── out             # Directory for compiled test binaries
│   │   └── some-test.c
│   ├── run.sh              # Scripts provided in this repository are expected to be in this directory
│   └── uefi                # Optional directory for UEFI applications
│       └── acpi-viewer.efi # See https://github.com/stargazerZJ/ACM-Acpi-Viewer for details
└── kvm
    └── busybox-1.36.1
        └── _install        # Busybox installation directory
```

### Step 1: Install Dependencies

Qemu:
```bash
sudo apt-get install qemu-system
```

EDK2:
```bash
sudo apt-get install ovmf
```

Build tools:
```bash
sudo apt-get install git fakeroot build-essential ncurses-dev xz-utils libssl-dev bc flex libelf-dev bison gcc-12 g++-12
```

If you have installed `gcc` earlier, you have to switch to `gcc-12` and `g++-12` to build the kernel, as the default `gcc` is `gcc-13` which is not supported by the kernel version used in this lab.
```bash
sudo update-alternatives --install /usr/bin/gcc gcc /usr/bin/gcc-12 100
sudo update-alternatives --install /usr/bin/g++ g++ /usr/bin/g++-12 100
```

### Step 2: Install Busybox

```bash
mkdir kvm && cd kvm
wget https://busybox.net/downloads/busybox-1.36.1.tar.bz2
tar -xjf busybox-1.36.1.tar.bz2
cd busybox-1.36.1

make menuconfig
# Enable: Settings > Build Options > [*] Build static binary
# Disable: Shells > [ ] Job control

make -j$(nproc) && make install
```

If you encounter compilation error `TCA_CBQ_MAX undeclared`, you can simply delete the `tc.c` file that causes the error, as it is not used in this lab.

The `_install` directory will be created in the `busybox-1.36.1` directory, which is the file system that will be used in the QEMU virtual machine. You should copy [init](init) file to the `_install` directory, which is a simple shell script that will be executed when the kernel boots up

```bash
cp path/to/the/provided/init/in/this/repository busybox-1.36.1/_install/init
```

Optional: You may add dynamically linked binaries and its required libraries to the `_install` directory, but it is not recommended as it may cause issues with the kernel's static linking. We provide a convenience script to add a dynamically binary.

```bash
## Add your system's bash binary
add-binary.sh $(which bash)
```

### Step 3: Download the Kernel Source

```bash
wget https://git.kernel.org/pub/scm/linux/kernel/git/stable/linux.git/snapshot/linux-5.19.17.tar.gz
tar -xzf linux-5.19.17.tar.gz
mv linux-5.19.17 mylinux
rm linux-5.19.17.tar.gz
cd mylinux
git init
```

### Step 4: Apply Patches

(You may skip this step the first time to try building the kernel without any modifications.)

```bash
# Go to a new branch to avoid modifying the original kernel source
git switch -c some-feature
# Apply your patches here, e.g.
git apply some-feature.patch
```

### Step 5: Build the Kernel

Some patches may add new configuration options, enable them when prompted during the configuration step.
```bash
make defconfig
make -j$(nproc)
```
Note: Compilation may take a while, depending on your system's performance. My Intel i9-13900K CPU takes about 45 seconds to compile the kernel.

Compile the test executables:
```bash
cd tests
compile() { gcc -static "$1" -o "out/${1:t:r}"; }
compile some-test.c
```

### Step 6: Run the Kernel in QEMU

You may change the paths in the `run.sh` script to match your directory structure.

```bash
./kernel/run.sh
```

Optionally, you can modify the `init` script in the `_install` directory to run your test executables automatically when the kernel boots up.