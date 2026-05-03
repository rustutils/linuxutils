#include <stdio.h>
#include "libmount.h"

int main(void) {
    struct libmnt_table *tb = mnt_new_table();
    mnt_table_parse_fstab(tb, "/*F0*/");

    struct libmnt_iter *fwd = mnt_new_iter(MNT_ITER_FORWARD);
    struct libmnt_iter *bwd = mnt_new_iter(MNT_ITER_BACKWARD);

    if (mnt_iter_get_direction(fwd) != MNT_ITER_FORWARD) return 1;
    if (mnt_iter_get_direction(bwd) != MNT_ITER_BACKWARD) return 2;

    struct libmnt_fs *fs = NULL;
    mnt_table_next_fs(tb, fwd, &fs);
    printf("first-fwd: %s\n", mnt_fs_get_target(fs));

    mnt_table_next_fs(tb, bwd, &fs);
    printf("first-bwd: %s\n", mnt_fs_get_target(fs));

    mnt_free_iter(fwd);
    mnt_free_iter(bwd);
    mnt_unref_table(tb);
    return 0;
}
