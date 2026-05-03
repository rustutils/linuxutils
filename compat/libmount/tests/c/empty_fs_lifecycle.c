#include <stdio.h>
#include "libmount.h"

int main(void) {
    struct libmnt_fs *fs = mnt_new_fs();
    if (!fs) { fprintf(stderr, "mnt_new_fs returned NULL\n"); return 1; }

    // Default fs has empty fstype, so getter returns NULL.
    if (mnt_fs_get_fstype(fs) != NULL) {
        fprintf(stderr, "empty fstype should be NULL\n"); return 1;
    }

    // Ref-count round-trip: ref bumps, unref drops, final unref
    // frees. We can't observe the count so this just checks for
    // crashes or memory abuse under valgrind.
    mnt_ref_fs(fs);
    mnt_unref_fs(fs);
    mnt_unref_fs(fs);

    printf("OK\n");
    return 0;
}
