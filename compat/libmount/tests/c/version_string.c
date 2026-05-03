#include <stdio.h>
#include <string.h>
#include "libmount.h"

int main(void) {
    const char *v = NULL;
    const char *r = mnt_get_library_version(&v);
    if (!v || !r || strcmp(v, r) != 0) {
        fprintf(stderr, "version mismatch: r=%s v=%s\n", r, v);
        return 1;
    }
    printf("version: %s\n", v);
    return 0;
}
