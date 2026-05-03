#include <stdio.h>
#include <string.h>
#include "uuid.h"

int main() {
    const uuid_t *dns = uuid_get_template("dns");
    uuid_t u1, u2;
    char buf1[UUID_STR_LEN], buf2[UUID_STR_LEN];

    // v5 (SHA1) should be deterministic.
    uuid_generate_sha1(u1, *dns, "example.com", 11);
    uuid_generate_sha1(u2, *dns, "example.com", 11);

    uuid_unparse(u1, buf1);
    uuid_unparse(u2, buf2);

    printf("%s\n", buf1);

    if (strcmp(buf1, buf2) != 0) {
        fprintf(stderr, "SHA1 not deterministic\n");
        return 1;
    }
    if (uuid_type(u1) != 5) {
        fprintf(stderr, "expected type 5, got %d\n", uuid_type(u1));
        return 1;
    }

    printf("OK\n");
    return 0;
}
