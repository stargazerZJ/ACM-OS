/*************************************************************************
 * NCCL vs MPI Communication Benchmark
 *
 * This application compares the performance of NCCL-based GPU communication
 * with traditional MPI-based Ethernet communication.
 ************************************************************************/

#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
#include <string.h>
#include <cuda_runtime.h>
#include <nccl.h>
#include <mpi.h>
#include <math.h>

#define CUDACHECK(cmd) do {                         \
  cudaError_t err = cmd;                            \
  if (err != cudaSuccess) {                        \
    printf("CUDA error: %s\n", cudaGetErrorString(err)); \
    exit(EXIT_FAILURE);                             \
  }                                                 \
} while(0)

#define NCCLCHECK(cmd) do {                         \
  ncclResult_t res = cmd;                           \
  if (res != ncclSuccess) {                         \
    printf("NCCL error: %s\n", ncclGetErrorString(res)); \
    exit(EXIT_FAILURE);                             \
  }                                                 \
} while(0)

#define MPICHECK(cmd) do {                          \
  int err = cmd;                                    \
  if (err != MPI_SUCCESS) {                         \
    char errstr[MPI_MAX_ERROR_STRING];              \
    int errlen;                                     \
    MPI_Error_string(err, errstr, &errlen);         \
    printf("MPI error: %s\n", errstr);              \
    exit(EXIT_FAILURE);                             \
  }                                                 \
} while(0)

typedef enum {
  OP_BROADCAST,
  OP_ALLREDUCE
} operation_t;

const char* operation_names[] = {
  "Broadcast",
  "AllReduce"
};

// Run NCCL operation and measure performance
double run_nccl_op(void* sendbuff, void* recvbuff, size_t count, ncclDataType_t datatype,
                  ncclRedOp_t op, int root, int rank, ncclComm_t comm, cudaStream_t stream, operation_t operation) {
  cudaEvent_t start, stop;
  CUDACHECK(cudaEventCreate(&start));
  CUDACHECK(cudaEventCreate(&stop));

  // Warmup
  for (int i = 0; i < 5; i++) {
    if (operation == OP_BROADCAST) {
      NCCLCHECK(ncclBroadcast(sendbuff, recvbuff, count, datatype, root, comm, stream));
    } else if (operation == OP_ALLREDUCE) {
      NCCLCHECK(ncclAllReduce(sendbuff, recvbuff, count, datatype, op, comm, stream));
    }
  }
  CUDACHECK(cudaStreamSynchronize(stream));

  // Benchmark
  int iterations = 20;
  CUDACHECK(cudaEventRecord(start, stream));
  for (int i = 0; i < iterations; i++) {
    if (operation == OP_BROADCAST) {
      NCCLCHECK(ncclBroadcast(sendbuff, recvbuff, count, datatype, root, comm, stream));
    } else if (operation == OP_ALLREDUCE) {
      NCCLCHECK(ncclAllReduce(sendbuff, recvbuff, count, datatype, op, comm, stream));
    }
  }
  CUDACHECK(cudaEventRecord(stop, stream));
  CUDACHECK(cudaStreamSynchronize(stream));

  float milliseconds = 0;
  CUDACHECK(cudaEventElapsedTime(&milliseconds, start, stop));
  CUDACHECK(cudaEventDestroy(start));
  CUDACHECK(cudaEventDestroy(stop));

  return milliseconds / iterations;
}

// Run MPI operation and measure performance
double run_mpi_op(void* sendbuff, void* recvbuff, size_t count, MPI_Datatype datatype,
                MPI_Op op, int root, int rank, int nranks, operation_t operation) {
  // Warmup
  for (int i = 0; i < 5; i++) {
    if (operation == OP_BROADCAST) {
      if (rank == root) {
        // For the root process, we need to use the sendbuff as source
        memcpy(recvbuff, sendbuff, count * sizeof(float));
      }
      MPICHECK(MPI_Bcast(recvbuff, count, datatype, root, MPI_COMM_WORLD));
    } else if (operation == OP_ALLREDUCE) {
      MPICHECK(MPI_Allreduce(sendbuff, recvbuff, count, datatype, op, MPI_COMM_WORLD));
    }
  }

  // Benchmark
  int iterations = 20;
  double start_time = MPI_Wtime();
  for (int i = 0; i < iterations; i++) {
    if (operation == OP_BROADCAST) {
      if (rank == root) {
        // For the root process, we need to use the sendbuff as source
        memcpy(recvbuff, sendbuff, count * sizeof(float));
      }
      MPICHECK(MPI_Bcast(recvbuff, count, datatype, root, MPI_COMM_WORLD));
    } else if (operation == OP_ALLREDUCE) {
      MPICHECK(MPI_Allreduce(sendbuff, recvbuff, count, datatype, op, MPI_COMM_WORLD));
    }
  }
  double end_time = MPI_Wtime();

  return ((end_time - start_time) * 1000.0) / iterations; // in milliseconds
}

// Initialize data for test
void init_data(float* data, size_t count, int rank) {
  for (size_t i = 0; i < count; i++) {
    data[i] = rank + 1.0f + (float)i / 1000.0f;
  }
}

// Verify the correctness of the data transfer
bool verify_data(float* result, float* expected, size_t count) {
  for (size_t i = 0; i < count; i++) {
    if (fabs(result[i] - expected[i]) > 1e-5) {
      printf("Data verification failed at index %zu: got %f, expected %f\n",
             i, result[i], expected[i]);
      return false;
    }
  }
  return true;
}

int main(int argc, char* argv[]) {
  // Initialize MPI
  int rank, nranks;
  MPICHECK(MPI_Init(&argc, &argv));
  MPICHECK(MPI_Comm_rank(MPI_COMM_WORLD, &rank));
  MPICHECK(MPI_Comm_size(MPI_COMM_WORLD, &nranks));

  // Get number of GPUs per node
  int device_count;
  CUDACHECK(cudaGetDeviceCount(&device_count));

  // Select GPU based on local rank
  int local_rank = rank % device_count;
  CUDACHECK(cudaSetDevice(local_rank));

  // Get device properties
  cudaDeviceProp prop;
  CUDACHECK(cudaGetDeviceProperties(&prop, local_rank));

  // Print system information
  // Have rank 0 print the header
  if (rank == 0) {
    printf("Running on %d processes\n", nranks);
  }

  // Barrier to ensure clean output
  MPI_Barrier(MPI_COMM_WORLD);

  // Each rank prints its GPU info
  printf("Rank %d using GPU %d: %s\n", rank, local_rank, prop.name);

  // Barrier before continuing
  MPI_Barrier(MPI_COMM_WORLD);

  // NCCL initialization
  ncclUniqueId nccl_id;
  ncclComm_t comm;
  cudaStream_t stream;

  // Root process generates the NCCL ID and broadcasts it to all
  if (rank == 0) {
    NCCLCHECK(ncclGetUniqueId(&nccl_id));
  }
  MPICHECK(MPI_Bcast(&nccl_id, sizeof(nccl_id), MPI_BYTE, 0, MPI_COMM_WORLD));

  // Initialize NCCL
  NCCLCHECK(ncclCommInitRank(&comm, nranks, nccl_id, rank));
  CUDACHECK(cudaStreamCreate(&stream));

  // Array of buffer sizes to test
  uint64_t buffer_sizes[] = {1ULL << 10, 1ULL << 14, 1ULL << 18, 1ULL << 20, 1ULL << 22, 1ULL << 24, 1ULL << 26, 1ULL << 28, 1ULL << 30, 1ULL << 32};
  int num_sizes = sizeof(buffer_sizes) / sizeof(buffer_sizes[0]);

  // Operations to test
  operation_t operations[] = {OP_BROADCAST, OP_ALLREDUCE};
  int num_operations = sizeof(operations) / sizeof(operations[0]);

  if (rank == 0) {
    printf("\n%20s | %15s | %15s | %15s | %15s | %10s\n",
           "Operation", "Size (bytes)", "NCCL Time (ms)", "MPI Time (ms)", "Speedup", "Verified");
    printf("----------------------------------------------------------------------------------------------------------\n");
  }

  // Main benchmark loop
  for (int op_idx = 0; op_idx < num_operations; op_idx++) {
    operation_t operation = operations[op_idx];
    int root = 0; // Root for broadcast operations

    for (int s = 0; s < num_sizes; s++) {
      size_t size = (size_t)buffer_sizes[s];
      size_t count = size / sizeof(float);

      // Allocate and initialize host data
      float *h_sendbuff, *h_recvbuff, *h_expected;
      h_sendbuff = (float*)malloc(size);
      h_recvbuff = (float*)malloc(size);
      h_expected = (float*)malloc(size);

      init_data(h_sendbuff, count, rank);
      memset(h_recvbuff, 0, size);

      // Prepare expected results for verification
      if (operation == OP_BROADCAST) {
        if (rank == root) {
          memcpy(h_expected, h_sendbuff, size);
        } else {
          float *root_data = (float*)malloc(size);
          init_data(root_data, count, root);
          memcpy(h_expected, root_data, size);
          free(root_data);
        }
      } else if (operation == OP_ALLREDUCE) {
        for (size_t i = 0; i < count; i++) {
          h_expected[i] = 0;
          for (int r = 0; r < nranks; r++) {
            h_expected[i] += r + 1.0f + (float)i / 1000.0f;
          }
        }
      }

      // Allocate device memory
      float *d_sendbuff, *d_recvbuff;
      CUDACHECK(cudaMalloc(&d_sendbuff, size));
      CUDACHECK(cudaMalloc(&d_recvbuff, size));

      // Copy data to device
      CUDACHECK(cudaMemcpy(d_sendbuff, h_sendbuff, size, cudaMemcpyHostToDevice));

      // For broadcast, root's receive buffer shouldn't be zero initialized
      if (operation == OP_BROADCAST && rank == root) {
        CUDACHECK(cudaMemcpy(d_recvbuff, h_sendbuff, size, cudaMemcpyHostToDevice));
      } else {
        CUDACHECK(cudaMemset(d_recvbuff, 0, size));
      }

      // Run NCCL operation
      double nccl_time = run_nccl_op(
        d_sendbuff, d_recvbuff, count, ncclFloat,
        ncclSum, root, rank, comm, stream, operation
      );

      // Verify NCCL result
      CUDACHECK(cudaMemcpy(h_recvbuff, d_recvbuff, size, cudaMemcpyDeviceToHost));
      bool nccl_verified = verify_data(h_recvbuff, h_expected, count);

      // Clean up for NCCL test
      CUDACHECK(cudaMemset(d_recvbuff, 0, size));

      // Run MPI operation
      memset(h_recvbuff, 0, size);
      double mpi_time = run_mpi_op(
        h_sendbuff, h_recvbuff, count, MPI_FLOAT,
        MPI_SUM, root, rank, nranks, operation
      );

      // Verify MPI result
      bool mpi_verified = verify_data(h_recvbuff, h_expected, count);

      // Calculate speedup
      double speedup = mpi_time / nccl_time;

      // Synchronize to ensure verification is complete
      MPI_Barrier(MPI_COMM_WORLD);

      // Aggregate verification results
      int nccl_verify_all = nccl_verified ? 1 : 0;
      int mpi_verify_all = mpi_verified ? 1 : 0;
      int nccl_verify_sum = 0;
      int mpi_verify_sum = 0;

      MPICHECK(MPI_Reduce(&nccl_verify_all, &nccl_verify_sum, 1, MPI_INT, MPI_SUM, 0, MPI_COMM_WORLD));
      MPICHECK(MPI_Reduce(&mpi_verify_all, &mpi_verify_sum, 1, MPI_INT, MPI_SUM, 0, MPI_COMM_WORLD));

      // Print results
      if (rank == 0) {
        bool all_verified = (nccl_verify_sum == nranks) && (mpi_verify_sum == nranks);
        printf("%20s | %15zu | %15.4f | %15.4f | %15.2fx | %10s\n",
               operation_names[operation], size, nccl_time, mpi_time, speedup,
               all_verified ? "Yes" : "No");
      }

      // Free memory
      CUDACHECK(cudaFree(d_sendbuff));
      CUDACHECK(cudaFree(d_recvbuff));
      free(h_sendbuff);
      free(h_recvbuff);
      free(h_expected);
    }
  }

  // Cleanup
  NCCLCHECK(ncclCommDestroy(comm));
  CUDACHECK(cudaStreamDestroy(stream));
  MPICHECK(MPI_Finalize());

  return 0;
}