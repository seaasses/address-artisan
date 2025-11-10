#include "src/opencl/headers/modular_operations/modular_inverse.h"
#include "src/opencl/headers/big_uint/big_uint_from_bytes.h"
#include "src/opencl/headers/big_uint/big_uint_to_bytes.h"

__kernel void modular_inverse_kernel(
    __global uchar *a_buffer,
    __global uchar *result_buffer)
{
    Uint256 a;
    Uint256 result;

    // Copy data from global to private memory and convert
    uchar a_private[32];

    for (int i = 0; i < 32; i++) {
        a_private[i] = a_buffer[i];
    }

    // Convert byte arrays to Uint256
    a = UINT256_FROM_BYTES(a_private);

    // Perform modular inverse
    result = modular_inverse(a);

    // Convert result back to bytes and copy to global memory
    uchar result_private[32];
    uint256_to_bytes(result, result_private);

    for (int i = 0; i < 32; i++) {
        result_buffer[i] = result_private[i];
    }
}