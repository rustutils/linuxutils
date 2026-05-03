#include <stdio.h>
#include "uuid.h"

int main() {
    uuid_t u;

    uuid_generate_random(u);
    printf("random: type=%d variant=%d\n", uuid_type(u), uuid_variant(u));

    uuid_generate_time(u);
    printf("time: type=%d variant=%d\n", uuid_type(u), uuid_variant(u));

    return 0;
}
