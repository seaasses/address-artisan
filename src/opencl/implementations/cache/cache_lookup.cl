#include "src/opencl/headers/cache/cache_lookup.h"

inline XPub cache_lookup_value(
    __global const CacheKey *cache_keys,
    __global const XPub *cache_values,
    const uint cache_size,
    const CacheKey search_key,
    int *found)
{
    int found_index = 0;
    int found_flag = 0;

    for (uint i = 0; i < cache_size; i++)
    {
        CacheKey key = cache_keys[i];

        int match_b = (key.b == search_key.b);
        int match_a = (key.a == search_key.a);
        int full_match = match_b & match_a;

        int accept_match = full_match & (1 - found_flag);
        found_flag = found_flag | full_match;

        int mask = -accept_match;
        found_index = (found_index & ~mask) | ((int)i & mask);
    }

    *found = found_flag;

    // This if is not a problem because found_flag is expected to be 1 100% of the time.
    if (found_flag)
    {
        return cache_values[found_index];
    }
    else
    {
        return (XPub){0};
    }
}
