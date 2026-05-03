#include <stdio.h>
#include "uuid.h"

int main() {
    uuid_t u1, u2;
    uuid_generate(u1);
    uuid_copy(u2, u1);

    if (uuid_compare(u1, u2) != 0) {
        fprintf(stderr, "copy should be equal\n");
        return 1;
    }

    printf("OK\n");
    return 0;
}
