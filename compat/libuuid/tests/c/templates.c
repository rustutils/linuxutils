#include <stdio.h>
#include <stdlib.h>
#include "uuid.h"

int main() {
    const uuid_t *dns = uuid_get_template("dns");
    const uuid_t *nil = uuid_get_template("nil");
    const uuid_t *bad = uuid_get_template("nonexistent");

    if (dns == NULL) { fprintf(stderr, "dns is null\n"); return 1; }
    if (nil == NULL) { fprintf(stderr, "nil is null\n"); return 1; }
    if (bad != NULL) { fprintf(stderr, "bad should be null\n"); return 1; }

    char buf[UUID_STR_LEN];
    uuid_unparse_lower(*dns, buf);
    printf("dns: %s\n", buf);

    if (!uuid_is_null(*nil)) {
        fprintf(stderr, "nil template should be null\n");
        return 1;
    }

    printf("OK\n");
    return 0;
}
