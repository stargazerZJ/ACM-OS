# NCCL vs Ethernet

## Environment Setup

**Important** This lab requires multiple NVIDIA GPUs and, preferbly, NVLink support.

### Step 1: Install NCCL

NCCL is usually already installed on the system if you have access to one. If not, it should be included in a standard system-wide CUDA installation.

### Step 2: Install MPI

```bash
sudo apt-get install openmpi-bin libopenmpi-dev openmpi-doc
```

### Step 3: Build the Lab

You may change the `ARCH` variable to match your GPU architecture. For example, if you have NVIDIA A100 GPUs, you can set `ARCH=sm_80`. If you have NVIDIA H100 GPUs, you can set `ARCH=sm_90`.

```bash
ARCH=sm_90 make
```