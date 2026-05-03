#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include "blkid.h"

int main(void) {
    blkid_cache cache = NULL;
    char *path = blkid_evaluate_tag("LABEL", "test-root", &cache);
    if (!path) {
        fprintf(stderr, "evaluate_tag returned NULL\n"); return 1;
    }
    printf("resolved: %s\n", path);
    free(path);

    char *missing = blkid_evaluate_tag("LABEL", "no-such-label", &cache);
    if (missing != NULL) {
        fprintf(stderr, "expected NULL for missing label, got %s\n", missing);
        free(missing);
        return 1;
    }

    // Spec form: same result as the explicit tag form.
    char *via_spec = blkid_evaluate_spec("LABEL=test-root", &cache);
    if (!via_spec) {
        fprintf(stderr, "evaluate_spec failed\n"); return 1;
    }
    printf("spec-resolved: %s\n", via_spec);
    free(via_spec);

    // Bare path passthrough.
    char *passthrough = blkid_evaluate_spec("/dev/sda99", &cache);
    if (!passthrough || strcmp(passthrough, "/dev/sda99") != 0) {
        fprintf(stderr, "spec passthrough mismatch: %s\n", passthrough); return 1;
    }
    free(passthrough);

    blkid_put_cache(cache);
    printf("OK\n");
    return 0;
}
