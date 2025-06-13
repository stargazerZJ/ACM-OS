#include <stdio.h>
#include <stdlib.h>
#include <pthread.h>
#include <time.h>
#include "kv_store.h"

#define NUM_THREADS 100
#define NUM_ITERATIONS 1000
#define MAX_KEY 2048

pthread_mutex_t mutex[MAX_KEY];
int expected_values[MAX_KEY];

void* thread_function(void* arg) {
    for (int i = 0; i < NUM_ITERATIONS; ++i) {
        int k = rand() % MAX_KEY;
        int v = rand();
        int operation = rand() % 2;

        if (operation) {
            int success = 0;
            pthread_mutex_lock(&mutex[k]);
            if (write_kv(k, v) != -1) {
                expected_values[k] = v;
                success = 1;
            }
            pthread_mutex_unlock(&mutex[k]);
            if (!success)   printf("Error writing key %d\n", k);
        } else {
            pthread_mutex_lock(&mutex[k]);
            int read_value = read_kv(k);
            int expected_value = expected_values[k];
            pthread_mutex_unlock(&mutex[k]);

            if (read_value != -1 && read_value != expected_value) {
                printf("Data inconsistency detected for key %d: expected %d, got %d\n", k, expected_value, read_value);
            }
        }
    }
    return NULL;
}

int main() {
    pthread_t threads[NUM_THREADS];
    srand(time(NULL));

    for (int i = 0; i < MAX_KEY; ++i) {
        pthread_mutex_init(&mutex[i], NULL);
        expected_values[i] = -1;
    }

    for (int i = 0; i < NUM_THREADS; ++i) {
        if (pthread_create(&threads[i], NULL, thread_function, NULL) != 0) {
            printf("Error creating thread %d\n", i);
            return -1;
        }
    }

    for (int i = 0; i < NUM_THREADS; ++i) {
        pthread_join(threads[i], NULL);
    }

    for (int i = 0; i < MAX_KEY; ++i) {
        pthread_mutex_destroy(&mutex[i]);
    }
    printf("Data consistency test completed.\n");
    return 0;
}