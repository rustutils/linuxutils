#include <stdio.h>
#include "uuid.h"

int main() {
    uuid_t u;
    uuid_generate(u);

    if (uuid_is_null(u)) {
        fprintf(stderr, "generated uuid should not be null\n");
        return 1;
    }

    uuid_clear(u);

    if (!uuid_is_null(u)) {
        fprintf(stderr, "cleared uuid should be null\n");
        return 1;
    }

    printf("OK\n");
    return 0;
}
