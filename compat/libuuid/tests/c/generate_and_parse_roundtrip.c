#include <stdio.h>
#include <string.h>
#include "uuid.h"

int main() {
    uuid_t u1, u2;
    char buf[UUID_STR_LEN];

    uuid_generate_random(u1);
    uuid_unparse(u1, buf);

    // Must be 36 chars (8-4-4-4-12).
    if (strlen(buf) != 36) {
        fprintf(stderr, "bad length: %zu\n", strlen(buf));
        return 1;
    }

    // Parse it back and compare.
    if (uuid_parse(buf, u2) != 0) {
        fprintf(stderr, "parse failed\n");
        return 1;
    }
    if (uuid_compare(u1, u2) != 0) {
        fprintf(stderr, "roundtrip mismatch\n");
        return 1;
    }

    printf("OK\n");
    return 0;
}
