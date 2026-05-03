#include <stdio.h>
#include <stdlib.h>
#include "blkid.h"

int main(void) {
    blkid_cache cache = NULL;
    blkid_get_cache(&cache, NULL);

    // First resolve the label so we know the canonical path.
    char *path = blkid_evaluate_tag("LABEL", "test-root", &cache);
    if (!path) { fprintf(stderr, "no test-root\n"); return 1; }
    printf("path: %s\n", path);

    blkid_dev d = blkid_get_dev(cache, path, BLKID_DEV_NORMAL);
    if (!d) { fprintf(stderr, "get_dev failed for %s\n", path); return 1; }

    int has_label = blkid_dev_has_tag(d, "LABEL", "test-root");
    int has_uuid = blkid_dev_has_tag(d, "UUID", NULL);
    int has_other = blkid_dev_has_tag(d, "LABEL", "wrong");
    printf("has-label-eq: %d\n", has_label);
    printf("has-uuid-any: %d\n", has_uuid);
    printf("has-other: %d\n", has_other);

    char *uuid = blkid_get_tag_value(cache, "UUID", path);
    if (uuid) {
        printf("uuid: %s\n", uuid);
        free(uuid);
    } else {
        printf("uuid: (none)\n");
    }

    blkid_dev found =
        blkid_find_dev_with_tag(cache, "LABEL", "test-root");
    if (!found) { fprintf(stderr, "find_dev_with_tag failed\n"); return 1; }
    if (found != d) { fprintf(stderr, "find_dev returned different dev\n"); return 1; }

    free(path);
    blkid_put_cache(cache);
    printf("OK\n");
    return 0;
}
