#include "src/opencl/headers/hash/ripemd160.h"

__kernel void ripemd160_32_bytes_kernel(
    __global const unsigned char *input_message,
    __global unsigned char *output_hash)
{
    // Copy input to local memory
    unsigned char local_message[RIPEMD160_32_BYTES_MESSAGE_SIZE];

#pragma unroll
    for (unsigned int i = 0; i < RIPEMD160_32_BYTES_MESSAGE_SIZE; i++)
    {
        local_message[i] = input_message[i];
    }

    // Compute hash
    unsigned char local_hash[RIPEMD160_HASH_SIZE];
    ripemd160_32_bytes(local_message, local_hash);

// Copy result to global memory
#pragma unroll
    for (unsigned int i = 0; i < RIPEMD160_HASH_SIZE; i++)
    {
        output_hash[i] = local_hash[i];
    }
}
