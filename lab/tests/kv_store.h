#ifndef KV_STORE_H
#define KV_STORE_H

#include <unistd.h>

#define SYS_WRITE_KV 451
#define SYS_READ_KV 452

int write_kv(int key, int value) {
    return syscall(SYS_WRITE_KV, key, value);
}

int read_kv(int key) {
    return syscall(SYS_READ_KV, key);
}

#endif /* KV_STORE_H */