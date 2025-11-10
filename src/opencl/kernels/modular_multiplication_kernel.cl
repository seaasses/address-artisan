#include "src/opencl/headers/modular_operations/modular_multiplication.cl.h"
#include "src/opencl/headers/big_uint/big_uint_from_bytes.cl.h"
#include "src/opencl/headers/big_uint/big_uint_to_bytes.cl.h"

__kernel void modular_multiplication_kernel(
    __global uchar *a_buffer,
    __global uchar *b_buffer,
    __global uchar *result_buffer)
{
    Uint256 a;
    Uint256 b;
    Uint256 result;

    // Copy data from global to private memory and convert
    uchar a_private[32];
    uchar b_private[32];

    for (int i = 0; i < 32; i++) {
        a_private[i] = a_buffer[i];
        b_private[i] = b_buffer[i];
    }

    // Convert byte arrays to Uint256
    a = UINT256_FROM_BYTES(a_private);
    b = UINT256_FROM_BYTES(b_private);

    // Perform modular multiplication
    result = modular_multiplication(a, b);

    // Convert result back to bytes and copy to global memory
    uchar result_private[32];
    uint256_to_bytes(result, result_private);

    for (int i = 0; i < 32; i++) {
        result_buffer[i] = result_private[i];
    }
}