#include "src/opencl/structs/structs.h"

XPub cache_lookup_value(
    __global const CacheKey* cache_keys,
    __global const XPub* cache_values,
    const uint cache_size,
    const CacheKey search_key,
    int* found
);
