#include <stdio.h>
#include <string.h>
#include "blkid.h"

int main(void) {
    const char *v = NULL;
    const char *d = NULL;
    int rc = blkid_get_library_version(&v, &d);
    if (rc < 0 || !v || !d) {
        fprintf(stderr, "version not populated: rc=%d v=%p d=%p\n", rc, v, d);
        return 1;
    }
    printf("version: %s\n", v);
    printf("date: %s\n", d);
    return 0;
}
