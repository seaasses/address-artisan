#include "src/opencl/headers/cache/cache_index.cl.h"

// O(1) cache position lookup.
//
// The cache holds XPubs for CONTIGUOUS (b, a) keys, ordered by the key
// ordinal b * 2^31 + a (a is always < 2^31: it is a non-hardened BIP32
// index). CacheRangeAnalyzer (Rust) generates the keys in exactly this
// order - see test_generated_keys_are_contiguous_ordinals - so the
// position of any key is its ordinal distance from the first cached key.
//
// Preconditions: a < 2^31 and first_a < 2^31.
//
// Sets *found to 0 when (b, a) is outside the cached range; the returned
// index is only meaningful when *found is 1.
inline uint cache_index(
    const uint b,
    const uint a,
    const uint first_b,
    const uint first_a,
    const uint cache_size,
    int *found)
{
    const ulong ordinal = ((ulong)b << 31) + (ulong)a;
    const ulong first_ordinal = ((ulong)first_b << 31) + (ulong)first_a;
    const ulong offset = ordinal - first_ordinal; // wraps when ordinal < first_ordinal

    *found = (ordinal >= first_ordinal) & (offset < (ulong)cache_size);

    return (uint)offset;
}
