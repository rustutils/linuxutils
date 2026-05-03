#include <stdio.h>
#include "blkid.h"

int main(void) {
    blkid_cache cache = NULL;
    if (blkid_get_cache(&cache, NULL) != 0) {
        fprintf(stderr, "get_cache failed\n"); return 1;
    }

    int n_devs = 0;
    blkid_dev_iterate it = blkid_dev_iterate_begin(cache);
    blkid_dev d = NULL;
    while (blkid_dev_next(it, &d) == 0) {
        const char *name = blkid_dev_devname(d);
        printf("dev: %s\n", name ? name : "(null)");
        n_devs++;

        blkid_tag_iterate ti = blkid_tag_iterate_begin(d);
        const char *t = NULL, *v = NULL;
        while (blkid_tag_next(ti, &t, &v) == 0) {
            printf("  tag: %s=%s\n", t, v);
        }
        blkid_tag_iterate_end(ti);
    }
    blkid_dev_iterate_end(it);
    printf("count: %d\n", n_devs);

    blkid_put_cache(cache);
    return 0;
}
