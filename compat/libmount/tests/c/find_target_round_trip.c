#include <stdio.h>
#include <string.h>
#include "libmount.h"

int main(void) {
    struct libmnt_table *tb = mnt_new_table();
    if (mnt_table_parse_fstab(tb, "/*F0*/") != 0) {
        fprintf(stderr, "parse_fstab failed\n"); return 1;
    }

    struct libmnt_fs *fs =
        mnt_table_find_target(tb, "/home", MNT_ITER_FORWARD);
    if (!fs) { fprintf(stderr, "find_target /home failed\n"); return 1; }
    printf("source: %s\n", mnt_fs_get_source(fs));
    printf("fstype: %s\n", mnt_fs_get_fstype(fs));

    fs = mnt_table_find_target(tb, "/nonexistent", MNT_ITER_FORWARD);
    if (fs != NULL) {
        fprintf(stderr, "expected NULL for /nonexistent\n"); return 1;
    }

    mnt_unref_table(tb);
    printf("OK\n");
    return 0;
}
