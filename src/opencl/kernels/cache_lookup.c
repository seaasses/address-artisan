#include "src/opencl/headers/cache/cache_lookup.h"

__kernel void cache_lookup(
    __global const CacheKey* cache_keys,
    __global const XPub* cache_values,
    __global const CacheKey* search_keys,
    __global XPub* output,
    __global int* found_flags,
    const uint cache_size,
    const uint search_count
)
{
    uint gid = get_global_id(0);
    if (gid >= search_count) return;

    CacheKey search_key = search_keys[gid];

    int found;
    XPub result = cache_lookup_value(cache_keys, cache_values, cache_size, search_key, &found);

    output[gid] = result;
    found_flags[gid] = found;
}
