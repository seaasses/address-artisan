#include "src/opencl/headers/cache/cache_index.cl.h"
#include "src/opencl/headers/secp256k1/ckdpub.cl.h"
#include "src/opencl/headers/secp256k1/jacobian_to_affine.cl.h"
#include "src/opencl/headers/modular_operations/modular_inverse_batch.cl.h"
#include "src/opencl/headers/big_uint/big_uint_to_bytes.cl.h"
#include "src/opencl/headers/hash/hash160.cl.h"

#define NON_HARDENED_MAX_INDEX 0x7FFFFFFF
#define NON_HARDENED_COUNT ((ulong)(NON_HARDENED_MAX_INDEX) + 1)
#define MAX_MATCHES 1000

// Number of addresses each thread processes. When > 1, the thread derives
// that many child points, then converts them all to affine with a SINGLE
// batched modular inversion (Montgomery's trick) instead of one inversion
// per address. Overridden at build time via -D POINTS_PER_THREAD=N.
#ifndef POINTS_PER_THREAD
#define POINTS_PER_THREAD 1
#endif


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
    __global const Hash160RangeGpu *ranges,
    const uint range_count,
    __global const uint *cache_size_buffer,  // Now a buffer instead of scalar
    const ulong start_counter,
    const uint max_depth,
    __global uchar *matches_hash160,
    __global uint *matches_b,
    __global uint *matches_a,
    __global uint *matches_index,
    __global uchar *matches_prefix_id,
    __global uint *match_count,
    __global uint *cache_miss_error,
    __global const Point *g_times_tables)
{
    uint gid = get_global_id(0);
    ulong base_counter = start_counter + (ulong)gid * POINTS_PER_THREAD;

    uint cache_size = cache_size_buffer[0];
    CacheKey first_key = cache_keys[0];

    // Per-point state for this thread's POINTS_PER_THREAD addresses
    JacobianPoint points[POINTS_PER_THREAD];
    Uint256 z_values[POINTS_PER_THREAD];
    Uint256 inv_scratch[POINTS_PER_THREAD];
    uint b_arr[POINTS_PER_THREAD];
    uint a_arr[POINTS_PER_THREAD];
    uint index_arr[POINTS_PER_THREAD];
    int valid[POINTS_PER_THREAD];

    // Phase 1: derive the jacobian child point for each of this thread's counters
    for (uint m = 0; m < POINTS_PER_THREAD; m++)
    {
        ulong counter = base_counter + m;

        // Counter -> [b, a, index]. c = 0 always (already cached)
        uint index = (uint)(counter % max_depth);
        ulong temp = counter / max_depth;
        uint a = (uint)(temp % NON_HARDENED_COUNT);
        uint b = (uint)(temp / NON_HARDENED_COUNT);

        b_arr[m] = b;
        a_arr[m] = a;
        index_arr[m] = index;

        // O(1) cache lookup: contiguous keys ordered by ordinal (b * 2^31 + a)
        int found;
        uint parent_index = cache_index(b, a, first_key.b, first_key.a, cache_size, &found);

        if (!found)
        {
            // Never happens in production (cache fully preloaded); the host
            // aborts on any miss. Use the identity so a stray miss cannot
            // poison the shared batch inversion of the other points.
            atomic_inc(cache_miss_error);
            valid[m] = 0;
            z_values[m] = (Uint256){.limbs = {0, 0, 0, 1}};
            points[m].z = z_values[m];
            continue;
        }

        valid[m] = 1;
        points[m] = ckdpub_jacobian(cache_values[parent_index], index, g_times_tables);
        z_values[m] = points[m].z;
    }

    // Phase 2: invert all Z coordinates with a single modular inverse
    modular_inverse_batch(z_values, inv_scratch, POINTS_PER_THREAD);

    // Phase 3: finish each point (affine -> compressed key -> hash160 -> match)
    for (uint m = 0; m < POINTS_PER_THREAD; m++)
    {
        if (!valid[m])
        {
            continue;
        }

        Point affine = jacobian_to_affine_with_z_inverse(points[m], z_values[m]);

        uchar compressed_key[33];
        compressed_key[0] = (uchar)(0x02 | (((uchar)(affine.y.limbs[3])) & 1));
        uint256_to_bytes(affine.x, &compressed_key[1]);

        uchar hash160[20];
        hash160_33(compressed_key, hash160);

        for (uint r = 0; r < range_count; r++)
        {
            __global const Hash160RangeGpu *range = &ranges[r];

            // this if is ok because matches are expected to be rare
            if (hash160_gte(hash160, range->low) && hash160_lte(hash160, range->high))
            {
                uint slot = atomic_inc(match_count);

                if (slot < MAX_MATCHES)
                {
                    for (int i = 0; i < 20; i++)
                    {
                        matches_hash160[slot * 20 + i] = hash160[i];
                    }
                    matches_b[slot] = b_arr[m];
                    matches_a[slot] = a_arr[m];
                    matches_index[slot] = index_arr[m];
                    matches_prefix_id[slot] = range->prefix_id;
                }

                break; // this point matched; move to the next one
            }
        }
    }
}
