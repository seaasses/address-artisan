#include "src/opencl/headers/cache/cache_index.cl.h"

__kernel void cache_index_kernel(
    __global const uint *query_b,
    __global const uint *query_a,
    const uint first_b,
    const uint first_a,
    const uint cache_size,
    const uint query_count,
    __global uint *result_index,
    __global int *result_found)
{
    uint gid = get_global_id(0);
    if (gid >= query_count)
    {
        return;
    }

    int found;
    uint index = cache_index(query_b[gid], query_a[gid], first_b, first_a, cache_size, &found);

    result_index[gid] = index;
    result_found[gid] = found;
}
