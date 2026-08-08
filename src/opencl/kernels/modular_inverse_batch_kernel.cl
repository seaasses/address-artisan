#include "src/opencl/headers/modular_operations/modular_inverse_batch.cl.h"
#include "src/opencl/headers/big_uint/big_uint_from_bytes.cl.h"
#include "src/opencl/headers/big_uint/big_uint_to_bytes.cl.h"

#define MAX_BATCH 32

__kernel void modular_inverse_batch_kernel(
    __global const uchar *values_buffer, // count * 32 bytes, big-endian
    const uint count,
    __global uchar *results_buffer) // count * 32 bytes, big-endian
{
    Uint256 values[MAX_BATCH];
    Uint256 prefix_scratch[MAX_BATCH];

    uchar bytes[32];
    for (uint i = 0; i < count; i++)
    {
        for (uint j = 0; j < 32; j++)
        {
            bytes[j] = values_buffer[i * 32 + j];
        }
        values[i] = UINT256_FROM_BYTES(bytes);
    }

    modular_inverse_batch(values, prefix_scratch, count);

    for (uint i = 0; i < count; i++)
    {
        uint256_to_bytes(values[i], bytes);
        for (uint j = 0; j < 32; j++)
        {
            results_buffer[i * 32 + j] = bytes[j];
        }
    }
}
