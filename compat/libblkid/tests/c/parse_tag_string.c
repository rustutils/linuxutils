#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include "blkid.h"

int main(void) {
    char *type = NULL;
    char *val = NULL;
    int rc = blkid_parse_tag_string("UUID=abcd-1234", &type, &val);
    if (rc != 0) {
        fprintf(stderr, "parse failed: rc=%d\n", rc); return 1;
    }
    if (strcmp(type, "UUID") != 0) {
        fprintf(stderr, "wrong type: %s\n", type); return 1;
    }
    if (strcmp(val, "abcd-1234") != 0) {
        fprintf(stderr, "wrong val: %s\n", val); return 1;
    }
    free(type);
    free(val);

    // Malformed input: no `=`.
    rc = blkid_parse_tag_string("BARE", &type, &val);
    if (rc == 0) {
        fprintf(stderr, "expected failure on bare token\n"); return 1;
    }

    printf("OK\n");
    return 0;
}
