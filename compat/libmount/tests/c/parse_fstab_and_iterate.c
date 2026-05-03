#include <stdio.h>
#include <string.h>
#include "libmount.h"

int main(void) {
    struct libmnt_table *tb = mnt_new_table();
    if (mnt_table_parse_fstab(tb, "/*F0*/") != 0) {
        fprintf(stderr, "parse_fstab failed\n"); return 1;
    }
    int n = mnt_table_get_nents(tb);
    printf("nents=%d\n", n);

    struct libmnt_iter *itr = mnt_new_iter(MNT_ITER_FORWARD);
    struct libmnt_fs *fs = NULL;
    while (mnt_table_next_fs(tb, itr, &fs) == 0) {
        const char *src = mnt_fs_get_source(fs);
        const char *tgt = mnt_fs_get_target(fs);
        const char *typ = mnt_fs_get_fstype(fs);
        printf("entry: %s %s %s\n",
               src ? src : "(null)",
               tgt ? tgt : "(null)",
               typ ? typ : "(null)");
    }
    mnt_free_iter(itr);
    mnt_unref_table(tb);
    return 0;
}
