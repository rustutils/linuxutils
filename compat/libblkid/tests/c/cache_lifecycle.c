#include <stdio.h>
#include "blkid.h"

int main(void) {
    blkid_cache cache = NULL;
    int rc = blkid_get_cache(&cache, NULL);
    if (rc != 0 || !cache) {
        fprintf(stderr, "blkid_get_cache failed: rc=%d cache=%p\n", rc, cache);
        return 1;
    }
    blkid_put_cache(cache);

    // Tolerate NULL.
    blkid_put_cache(NULL);

    printf("OK\n");
    return 0;
}
