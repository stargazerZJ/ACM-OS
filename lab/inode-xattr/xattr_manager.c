#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/types.h>
#include <sys/stat.h>
#include <unistd.h>
#include <errno.h>
#include <fcntl.h>
#include <sys/xattr.h>

void print_inode_info(const char *filename) {
    struct stat file_stat;

    if (stat(filename, &file_stat) == -1) {
        perror("stat failed");
        return;
    }

    printf("Basic Inode Information for %s:\n", filename);
    printf("  Inode number: %ld\n", (long)file_stat.st_ino);
    printf("  File mode: %o\n", file_stat.st_mode);
    printf("  Link count: %ld\n", (long)file_stat.st_nlink);
    printf("  UID: %d\n", file_stat.st_uid);
    printf("  GID: %d\n", file_stat.st_gid);
    printf("  Size: %ld bytes\n", (long)file_stat.st_size);
    printf("  Block size: %ld bytes\n", (long)file_stat.st_blksize);
    printf("  Blocks allocated: %ld\n", (long)file_stat.st_blocks);
    printf("  Last access time: %ld\n", (long)file_stat.st_atime);
    printf("  Last modification time: %ld\n", (long)file_stat.st_mtime);
    printf("  Last status change time: %ld\n", (long)file_stat.st_ctime);
    printf("\n");
}

int set_xattr(const char *path, const char *name, const char *value) {
    int result = setxattr(path, name, value, strlen(value), 0);
    if (result == -1) {
        perror("setxattr failed");
        return -1;
    }
    printf("Set extended attribute '%s' to '%s'\n", name, value);
    return 0;
}

int get_xattr(const char *path, const char *name) {
    char buffer[256];
    ssize_t size = getxattr(path, name, buffer, sizeof(buffer) - 1);

    if (size == -1) {
        if (errno == ENODATA) {
            printf("No extended attribute named '%s'\n", name);
        } else {
            perror("getxattr failed");
        }
        return -1;
    }

    buffer[size] = '\0';
    printf("Extended attribute '%s' = '%s'\n", name, buffer);
    return 0;
}

int list_xattrs(const char *path) {
    char list[1024];
    ssize_t len = listxattr(path, list, sizeof(list));

    if (len == -1) {
        perror("listxattr failed");
        return -1;
    }

    printf("Extended attributes for %s:\n", path);
    if (len == 0) {
        printf("  (none)\n");
        return 0;
    }

    char *name = list;
    while (name < list + len) {
        printf("  %s\n", name);
        get_xattr(path, name);
        name += strlen(name) + 1;
    }

    return 0;
}

int remove_xattr(const char *path, const char *name) {
    int result = removexattr(path, name);
    if (result == -1) {
        if (errno == ENODATA) {
            printf("No extended attribute named '%s' to remove\n", name);
        } else {
            perror("removexattr failed");
        }
        return -1;
    }

    printf("Removed extended attribute '%s'\n", name);
    return 0;
}

int main(int argc, char *argv[]) {
    if (argc < 2) {
        fprintf(stderr, "Usage: %s <filename>\n", argv[0]);
        return EXIT_FAILURE;
    }

    const char *filename = argv[1];

    // Check if file exists
    if (access(filename, F_OK) == -1) {
        printf("File %s does not exist. Creating it...\n", filename);
        FILE *f = fopen(filename, "w");
        if (f == NULL) {
            perror("Failed to create file");
            return EXIT_FAILURE;
        }
        fprintf(f, "This is a test file for xattr demonstration.\n");
        fclose(f);
    }

    // Print inode information
    print_inode_info(filename);

    printf("\n--- Extended Attribute Operations ---\n\n");

    // Set some extended attributes
    set_xattr(filename, "user.comment", "This is a comment");
    set_xattr(filename, "user.category", "test file");
    set_xattr(filename, "user.rating", "5 stars");

    // List all extended attributes
    printf("\nListing all extended attributes:\n");
    list_xattrs(filename);

    // Get a specific attribute
    printf("\nGetting specific attribute:\n");
    get_xattr(filename, "user.comment");

    // Remove an attribute
    printf("\nRemoving an attribute:\n");
    remove_xattr(filename, "user.rating");

    // List attributes after removal
    printf("\nListing attributes after removal:\n");
    list_xattrs(filename);

    return EXIT_SUCCESS;
}