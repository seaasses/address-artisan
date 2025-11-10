#include "src/opencl/headers/big_uint/big_uint_multiplication.cl.h"
#include "src/opencl/headers/big_uint/big_uint_from_bytes.cl.h"
#include "src/opencl/headers/big_uint/big_uint_to_bytes.cl.h"

__kernel void uint256_ulong_multiplication_kernel(
    __global uchar *a_buffer,
    __global uchar *b_buffer,
    __global uchar *result_buffer)
{
    Uint256 a;
    ulong b;
    Uint320 result;

    // Copy data from global to private memory and convert
    uchar a_private[32];
    uchar b_private[8];

    for (int i = 0; i < 32; i++) {
        a_private[i] = a_buffer[i];
    }
    for (int i = 0; i < 8; i++) {
        b_private[i] = b_buffer[i];
    }

    // Convert byte arrays to Uint256 and ulong
    a = UINT256_FROM_BYTES(a_private);
    b = ULONG_FROM_BYTES(b_private);

    // Perform multiplication
    result = uint256_ulong_multiplication(a, b);

    // Convert result back to bytes and copy to global memory
    uchar result_private[40];
    uint320_to_bytes(result, result_private);

    for (int i = 0; i < 40; i++) {
        result_buffer[i] = result_private[i];
    }
}