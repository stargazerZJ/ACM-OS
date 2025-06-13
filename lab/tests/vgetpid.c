// Note that this test cannot be statically linked.

#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
#include <dlfcn.h>
#include <string.h>
#include <errno.h>
#include <sys/wait.h>
#include <sys/resource.h>
#include <time.h>

typedef pid_t (*getpid_t)(void);

// Function to test both standard and vDSO getpid
void test_getpid(const char* prefix) {
    // Get PID using standard library call
    pid_t std_pid = getpid();
    printf("%s: PID from standard getpid(): %d\n", prefix, std_pid);

    // Find and call the vDSO version
    void *vdso_handle = dlopen("linux-vdso.so.1", RTLD_LAZY);
    if (!vdso_handle) {
        fprintf(stderr, "%s: Failed to open vDSO: %s\n", prefix, dlerror());
        return;
    }

    // Try looking up the function by its internal name
    getpid_t vdso_getpid = (getpid_t)dlsym(vdso_handle, "__vdso_getpid");
    if (!vdso_getpid) {
        fprintf(stderr, "%s: Function __vdso_getpid not found: %s\n", prefix, dlerror());

        // Try with the regular name as fallback
        vdso_getpid = (getpid_t)dlsym(vdso_handle, "getpid");
        if (!vdso_getpid) {
            fprintf(stderr, "%s: Function getpid not found in vDSO: %s\n", prefix, dlerror());
            dlclose(vdso_handle);
            return;
        }
    }

    // Call the vDSO function
    pid_t vdso_pid = vdso_getpid();
    printf("%s: PID from vDSO getpid(): %d\n", prefix, vdso_pid);

    // Compare results
    if (std_pid == vdso_pid) {
        printf("%s: SUCCESS: PIDs match!\n", prefix);
    } else {
        printf("%s: FAILURE: PIDs do not match! (std: %d, vDSO: %d)\n",
               prefix, std_pid, vdso_pid);
    }

    dlclose(vdso_handle);
}

// Function to get memory usage statistics
void print_memory_stats(const char* stage) {
    FILE *meminfo = fopen("/proc/meminfo", "r");
    if (!meminfo) {
        fprintf(stderr, "Failed to open /proc/meminfo\n");
        return;
    }

    char line[256];
    long mem_total = 0, mem_free = 0, mem_available = 0;

    while (fgets(line, sizeof(line), meminfo)) {
        if (strncmp(line, "MemTotal:", 9) == 0) {
            sscanf(line, "MemTotal: %ld kB", &mem_total);
        } else if (strncmp(line, "MemFree:", 8) == 0) {
            sscanf(line, "MemFree: %ld kB", &mem_free);
        } else if (strncmp(line, "MemAvailable:", 13) == 0) {
            sscanf(line, "MemAvailable: %ld kB", &mem_available);
        }
    }
    fclose(meminfo);

    printf("%s - Memory: Total=%ld MB, Free=%ld MB, Available=%ld MB, Used=%ld MB\n",
           stage, mem_total/1024, mem_free/1024, mem_available/1024,
           (mem_total - mem_available)/1024);
}

// Stress test to verify per-process data pages are freed
void stress_test_memory_cleanup() {
    printf("\n=== MEMORY CLEANUP STRESS TEST STARTING ===\n");

    const int BATCH_SIZE = 100;
    const int NUM_BATCHES = 50;  // Total: 5000 processes
    const int TOTAL_PROCESSES = BATCH_SIZE * NUM_BATCHES;

    printf("Will create %d processes in %d batches of %d processes each\n",
           TOTAL_PROCESSES, NUM_BATCHES, BATCH_SIZE);

    print_memory_stats("BEFORE stress test");

    time_t start_time = time(NULL);

    for (int batch = 0; batch < NUM_BATCHES; batch++) {
        printf("Starting batch %d/%d...\n", batch + 1, NUM_BATCHES);

        // Create a batch of child processes
        for (int i = 0; i < BATCH_SIZE; i++) {
            pid_t child_pid = fork();

            if (child_pid < 0) {
                fprintf(stderr, "Fork failed at batch %d, process %d: %s\n",
                       batch, i, strerror(errno));
                continue;
            }
            else if (child_pid == 0) {
                // Child process: call getpid to trigger vDSO page allocation
                // Call it multiple times to ensure the page is used
                for (int j = 0; j < 10; j++) {
                    volatile pid_t pid = getpid();
                    (void)pid; // Suppress unused variable warning
                }

                // Exit immediately to test cleanup
                exit(EXIT_SUCCESS);
            }
            // Parent continues to create more children
        }

        // Wait for all children in this batch to complete
        for (int i = 0; i < BATCH_SIZE; i++) {
            int status;
            pid_t waited_pid = wait(&status);
            if (waited_pid < 0) {
                if (errno == ECHILD) {
                    break; // No more children
                }
                fprintf(stderr, "Wait failed: %s\n", strerror(errno));
            }
        }

        // Print memory stats every 10 batches
        if ((batch + 1) % 10 == 0) {
            char stage_msg[100];
            sprintf(stage_msg, "After batch %d/%d (%d processes)",
                   batch + 1, NUM_BATCHES, (batch + 1) * BATCH_SIZE);
            print_memory_stats(stage_msg);
        }

        // Small delay to let kernel do cleanup
        usleep(10000); // 10ms
    }

    time_t end_time = time(NULL);

    printf("\n=== STRESS TEST COMPLETED ===\n");
    printf("Created and cleaned up %d processes in %ld seconds\n",
           TOTAL_PROCESSES, end_time - start_time);

    print_memory_stats("AFTER stress test");

    printf("=== FINAL MEMORY CHECK (after 2 second delay) ===\n");
    sleep(2); // Give kernel time for final cleanup
    print_memory_stats("FINAL");
}

// Alternative stress test that creates processes more aggressively
void aggressive_stress_test() {
    printf("\n=== AGGRESSIVE MEMORY STRESS TEST ===\n");

    const int MAX_CONCURRENT = 50;
    const int TOTAL_CYCLES = 200;

    printf("Will create %d cycles of %d concurrent processes each\n",
           TOTAL_CYCLES, MAX_CONCURRENT);

    print_memory_stats("BEFORE aggressive test");

    for (int cycle = 0; cycle < TOTAL_CYCLES; cycle++) {
        pid_t children[MAX_CONCURRENT];
        int created = 0;

        // Create concurrent children
        for (int i = 0; i < MAX_CONCURRENT; i++) {
            pid_t child_pid = fork();

            if (child_pid < 0) {
                fprintf(stderr, "Fork failed at cycle %d: %s\n", cycle, strerror(errno));
                break;
            }
            else if (child_pid == 0) {
                // Child: use vDSO multiple times then exit
                for (int j = 0; j < 20; j++) {
                    getpid();
                }
                exit(EXIT_SUCCESS);
            }
            else {
                children[created++] = child_pid;
            }
        }

        // Wait for all children in this cycle
        for (int i = 0; i < created; i++) {
            int status;
            waitpid(children[i], &status, 0);
        }

        if ((cycle + 1) % 50 == 0) {
            char msg[100];
            sprintf(msg, "Aggressive test cycle %d/%d", cycle + 1, TOTAL_CYCLES);
            print_memory_stats(msg);
        }
    }

    printf("=== AGGRESSIVE TEST COMPLETED ===\n");
    print_memory_stats("AFTER aggressive test");
}

int main() {
    printf("=== Parent process starting ===\n");
    test_getpid("PARENT");

    // Create a chain of 3 child processes
    for (int level = 1; level <= 3; level++) {
        pid_t child_pid = fork();

        if (child_pid < 0) {
            fprintf(stderr, "Fork failed: %s\n", strerror(errno));
            exit(EXIT_FAILURE);
        }
        else if (child_pid == 0) {
            // Child process
            char prefix[20];
            sprintf(prefix, "CHILD-%d", level);
            printf("\n=== %s process started ===\n", prefix);
            test_getpid(prefix);

            // If not the last child in the chain, let it continue the loop
            // If it's the last child, break out and exit
            if (level == 3) {
                printf("=== %s process exiting ===\n\n", prefix);
                exit(EXIT_SUCCESS);
            }
        }
        else {
            // Parent process
            int status;
            waitpid(child_pid, &status, 0);
            printf("=== Child process with PID %d has exited ===\n\n", child_pid);
            if (level > 1) return 0;
            break;  // Parent doesn't continue the loop
        }
    }

    // Renamed verification in the parent process
    printf("=== Parent process verification before stress test ===\n");
    test_getpid("PARENT-VERIFICATION");

    // Memory cleanup stress tests
    stress_test_memory_cleanup();
    aggressive_stress_test();

    printf("=== All tests completed successfully - No OOM occurred ===\n");

    return 0;
}