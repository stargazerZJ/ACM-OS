# Makefile for NCCL vs MPI Benchmark

CUDA_HOME ?= /usr/local/cuda
MPI_HOME ?= /usr/lib/x86_64-linux-gnu/openmpi
ARCH ?= sm_90

NVCC = $(CUDA_HOME)/bin/nvcc
CUDA_INC = -I$(CUDA_HOME)/include
CUDA_LIB = -L$(CUDA_HOME)/lib64 -lcudart

MPI_INC = -I$(MPI_HOME)/include
MPI_LIB = -L$(MPI_HOME)/lib -lmpi

NCCL_INC = -I/usr/include
NCCL_LIB = -lnccl

INCLUDES = $(CUDA_INC) $(MPI_INC) $(NCCL_INC)
LIBS = $(CUDA_LIB) $(MPI_LIB) $(NCCL_LIB)

NVCCFLAGS = -O3 -std=c++11 -Xcompiler -fopenmp -arch=$(ARCH) $(INCLUDES)
LDFLAGS = $(LIBS)

TARGET = nccl_vs_mpi_benchmark

all: $(TARGET)

$(TARGET): nccl_vs_mpi_benchmark.cu
	$(NVCC) $(NVCCFLAGS) -o $@ $< $(LDFLAGS)

clean:
	rm -f $(TARGET)

.PHONY: all clean