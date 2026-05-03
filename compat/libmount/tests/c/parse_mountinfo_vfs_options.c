#include <stdio.h>
#include "libmount.h"

int main(void) {
    struct libmnt_table *tb = mnt_new_table();
    if (mnt_table_parse_mtab(tb, "/*F0*/") != 0) {
        fprintf(stderr, "parse_mtab failed\n"); return 1;
    }
    struct libmnt_fs *fs = mnt_table_find_target(tb, "/", MNT_ITER_FORWARD);
    if (!fs) { fprintf(stderr, "no / mount\n"); return 1; }
    printf("vfs: %s\n", mnt_fs_get_vfs_options(fs));
    printf("fs: %s\n", mnt_fs_get_fs_options(fs));
    printf("id: %d\n", mnt_fs_get_id(fs));
    printf("parent: %d\n", mnt_fs_get_parent_id(fs));
    mnt_unref_table(tb);
    return 0;
}
