#ifndef __IMPL_H__
#define __IMPL_H__

#include <stdio.h>
#include <stdlib.h>
#include <fcntl.h>
#include <sys/mman.h>
#include <sys/stat.h>
#include <unistd.h>
#include <string.h>
#include <errno.h>

/**
 * @brief Remap a virtual memory region
 * @param addr Original mapped memory address, if NULL system will automatically choose a suitable address
 * @param size Size to be mapped (in bytes)
 * @return Returns the mapped address on success, NULL on failure
 * @details This function is used to remap a new virtual memory region. If the addr parameter is NULL,
 *          the system will automatically choose a suitable address for mapping. The mapped memory region size is size bytes.
 *          Returns NULL on mapping failure.
 */
void* mmap_remap(void *addr, size_t size) {
    // Create a new mapping with the same permissions and flags
    void* new_addr = mmap(NULL, size, PROT_READ | PROT_WRITE,
                          MAP_PRIVATE | MAP_ANONYMOUS, -1, 0);

    if (new_addr == MAP_FAILED) {
        perror("mmap failed in mmap_remap");
        return NULL;
    }

    // Copy the contents from the old address to the new one
    if (addr != NULL) {
        memcpy(new_addr, addr, size);
    }

    return new_addr;
}

/**
 * @brief Use mmap for file reading and writing
 * @param filename Path of the file to operate on
 * @param offset Offset in the file for writing (in bytes)
 * @param content Content to write to the file
 * @return Returns 0 on success, -1 on failure
 * @details This function uses memory mapping (mmap) for file write operations.
 *          It specifies the file to write to through filename,
 *          the starting position with offset,
 *          and the content to write with content.
 *          Returns 0 on successful write, -1 on failure.
 */
 int file_mmap_write(const char* filename, size_t offset, char* content) {
    int fd;
    struct stat statbuf;
    size_t content_len = strlen(content);
    size_t required_size = offset + content_len;

    // Open the file (create if it doesn't exist)
    fd = open(filename, O_RDWR | O_CREAT, 0666);
    if (fd == -1) {
        perror("open failed");
        return -1;
    }

    // Get the current file size
    if (fstat(fd, &statbuf) == -1) {
        perror("fstat failed");
        close(fd);
        return -1;
    }

    // If the file is smaller than the required size, extend it
    if (statbuf.st_size < required_size) {
        if (ftruncate(fd, required_size) == -1) {
            perror("ftruncate failed");
            close(fd);
            return -1;
        }
    }

    // Calculate page-aligned mapping parameters
    size_t page_size = sysconf(_SC_PAGE_SIZE);
    size_t page_offset = offset & ~(page_size - 1);  // Round down to page boundary
    size_t offset_in_map = offset - page_offset;
    size_t map_size = offset_in_map + content_len;

    // Map the file into memory
    void* mapped_addr = mmap(NULL, map_size, PROT_READ | PROT_WRITE,
                            MAP_SHARED, fd, page_offset);
    if (mapped_addr == MAP_FAILED) {
        perror("mmap failed");
        close(fd);
        return -1;
    }

    // Write the content at the calculated offset within the map
    memcpy((char*)mapped_addr + offset_in_map, content, content_len);

    // Sync changes to disk
    if (msync(mapped_addr, map_size, MS_SYNC) == -1) {
        perror("msync failed");
        munmap(mapped_addr, map_size);
        close(fd);
        return -1;
    }

    // Unmap the file
    if (munmap(mapped_addr, map_size) == -1) {
        perror("munmap failed");
        close(fd);
        return -1;
    }

    // Close the file
    close(fd);

    return 0;
}

#endif // __IMPL_H__