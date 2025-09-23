#include "src/opencl/headers/big_uint/big_uint_multiplication.h"
#include "src/opencl/headers/big_uint/big_uint_from_bytes.h"
#include "src/opencl/headers/big_uint/big_uint_to_bytes.h"

__kernel void uint256_ulong_multiplication_kernel(
    __global unsigned char *a_buffer,
    __global unsigned char *b_buffer,
    __global unsigned char *result_buffer)
{
    Uint256 a;
    ulong b;
    Uint320 result;

    // Copy data from global to private memory and convert
    unsigned char a_private[32];
    unsigned char b_private[8];

    for (int i = 0; i < 32; i++) {
        a_private[i] = a_buffer[i];
    }
    for (int i = 0; i < 8; i++) {
        b_private[i] = b_buffer[i];
    }

    // Convert byte arrays to Uint256 and ulong
    bytes_to_uint256(a_private, &a);
    b = bytes_to_ulong(b_private);

    // Perform multiplication
    uint256_ulong_multiplication(&a, b, &result);

    // Convert result back to bytes and copy to global memory
    unsigned char result_private[40];
    uint320_to_bytes(result, result_private);

    for (int i = 0; i < 40; i++) {
        result_buffer[i] = result_private[i];
    }
}