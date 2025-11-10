#include "src/opencl/headers/cache/cache_lookup.h"
#include "src/opencl/headers/secp256k1/ckdpub.h"
#include "src/opencl/headers/hash/hash160.h"

#define NON_HARDENED_MAX_INDEX 0x7FFFFFFF
#define NON_HARDENED_COUNT ((ulong)(NON_HARDENED_MAX_INDEX) + 1)
#define MAX_MATCHES 1000

// Branchless compare: returns 1 if a >= b, 0 otherwise
inline int hash160_gte(const uchar a[20], __global const uchar *b)
{
    int gt = 0; // Found byte where a > b
    int eq = 1; // All bytes equal so far

#pragma unroll
    for (int i = 0; i < 20; i++)
    {
        int a_byte = a[i];
        int b_byte = b[i];

        int is_greater = (a_byte > b_byte);
        int is_equal = (a_byte == b_byte);

        // If still equal and this byte is greater, mark gt
        gt |= (eq & is_greater);

        // Keep eq flag only if was equal AND this byte is equal
        eq &= is_equal;
    }

    return gt | eq; // a >= b if (a > b) OR (a == b)
}

// Branchless compare: returns 1 if a <= b, 0 otherwise
inline int hash160_lte(const uchar a[20], __global const uchar *b)
{
    int lt = 0; // Found byte where a < b
    int eq = 1; // All bytes equal so far

#pragma unroll
    for (int i = 0; i < 20; i++)
    {
        int a_byte = a[i];
        int b_byte = b[i];

        int is_less = (a_byte < b_byte);
        int is_equal = (a_byte == b_byte);

        // If still equal and this byte is less, mark lt
        lt |= (eq & is_less);

        // Keep eq flag only if was equal AND this byte is equal
        eq &= is_equal;
    }

    return lt | eq; // a <= b if (a < b) OR (a == b)
}

__kernel void batch_address_search(
    __global const CacheKey *cache_keys,
    __global const XPub *cache_values,
    __global const uchar *range_lows,
    __global const uchar *range_highs,
    const uint range_count,
    __global const uint *cache_size_buffer,  // Now a buffer instead of scalar
    const ulong start_counter,
    const uint max_depth,
    __global uchar *matches_hash160,
    __global uint *matches_b,
    __global uint *matches_a,
    __global uint *matches_index,
    __global uint *match_count,
    __global uint *cache_miss_error)
{
    uint gid = get_global_id(0);
    ulong counter = start_counter + gid;

    // Read cache size from buffer
    uint cache_size = cache_size_buffer[0];

    // Counter -> [b, a, index]
    // c = 0 always (already cached)
    uint index = (uint)(counter % max_depth);
    ulong temp = counter / max_depth;
    uint a = (uint)(temp % NON_HARDENED_COUNT);
    uint b = (uint)(temp / NON_HARDENED_COUNT);

    // Lookup cache [b, a]
    CacheKey search_key;
    search_key.b = b;
    search_key.a = a;

    int found;
    XPub parent = cache_lookup_value(cache_keys, cache_values, cache_size, search_key, &found);

    if (!found)
    {
        atomic_inc(cache_miss_error); // Increment error counter on cache miss
        return;
    }

    // Derive child at index
    uchar compressed_key[33];
    ckdpub(parent, index, compressed_key);

    // Calculate hash160
    uchar hash160[20];
    hash160_33(compressed_key, hash160);

    // Check all ranges
    for (uint r = 0; r < range_count; r++)
    {
        __global const uchar *low = &range_lows[r * 20];
        __global const uchar *high = &range_highs[r * 20];

        // Check if low <= hash160 <= high
        // this if is ok because matches are expected to be rare
        if (hash160_gte(hash160, low) && hash160_lte(hash160, high))
        {
            // MATCH! Save atomically
            uint slot = atomic_inc(match_count);

            if (slot < MAX_MATCHES)
            {
                // Save hash160
                for (int i = 0; i < 20; i++)
                {
                    matches_hash160[slot * 20 + i] = hash160[i];
                }

                // Save path [b, a, index]
                matches_b[slot] = b;
                matches_a[slot] = a;
                matches_index[slot] = index;
            }

            return; // Found match, no need to check other ranges
        }
    }
}
