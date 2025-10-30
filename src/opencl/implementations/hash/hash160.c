#include "src/opencl/headers/hash/hash160.h"
#include "src/opencl/headers/hash/sha256.h"
#include "src/opencl/headers/hash/ripemd160.h"

inline void hash160_33(const uchar *restrict input, uchar *restrict output)
{
    uchar sha256_hash[32];
    sha256_33_bytes(input, sha256_hash);
    ripemd160_32_bytes(sha256_hash, output);
}
