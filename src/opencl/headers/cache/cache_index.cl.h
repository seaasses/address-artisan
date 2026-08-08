#include "src/opencl/structs/structs.cl.h"

uint cache_index(
    const uint b,
    const uint a,
    const uint first_b,
    const uint first_a,
    const uint cache_size,
    int *found
);
