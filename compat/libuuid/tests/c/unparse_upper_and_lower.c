#include <stdio.h>
#include <ctype.h>
#include <string.h>
#include "uuid.h"

int main() {
    uuid_t u;
    char lower[UUID_STR_LEN], upper[UUID_STR_LEN];

    uuid_parse("550e8400-e29b-41d4-a716-446655440000", u);
    uuid_unparse_lower(u, lower);
    uuid_unparse_upper(u, upper);

    printf("lower: %s\n", lower);
    printf("upper: %s\n", upper);

    // Verify case.
    for (int i = 0; i < 36; i++) {
        if (isalpha(lower[i]) && !islower(lower[i])) return 1;
        if (isalpha(upper[i]) && !isupper(upper[i])) return 1;
    }

    return 0;
}
