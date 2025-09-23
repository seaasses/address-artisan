#include "src/opencl/headers/modular_operations/modular_subtraction.h"
#include "src/opencl/headers/big_uint/big_uint_from_bytes.h"
#include "src/opencl/headers/big_uint/big_uint_to_bytes.h"

__kernel void modular_subtraction_kernel(
    __global unsigned char *a_buffer,
    __global unsigned char *b_buffer,
    __global unsigned char *result_buffer)
{
    Uint256 a;
    Uint256 b;
    Uint256 result;

    // Copy data from global to private memory and convert
    unsigned char a_private[32];
    unsigned char b_private[32];

    for (int i = 0; i < 32; i++) {
        a_private[i] = a_buffer[i];
        b_private[i] = b_buffer[i];
    }

    // Convert byte arrays to Uint256
    bytes_to_uint256(a_private, &a);
    bytes_to_uint256(b_private, &b);

    // Perform modular subtraction
    modular_subtraction(&a, &b, &result);

    // Convert result back to bytes and copy to global memory
    unsigned char result_private[32];
    uint256_to_bytes(result, result_private);

    for (int i = 0; i < 32; i++) {
        result_buffer[i] = result_private[i];
    }
}