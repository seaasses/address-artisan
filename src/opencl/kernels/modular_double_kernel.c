#include "src/opencl/headers/modular_operations/modular_double.h"
#include "src/opencl/headers/big_uint/big_uint_from_bytes.h"
#include "src/opencl/headers/big_uint/big_uint_to_bytes.h"

__kernel void modular_double_kernel(
    __global unsigned char *a_buffer,
    __global unsigned char *result_buffer)
{
    Uint256 a;
    Uint256 result;

    // Copy data from global to private memory and convert
    unsigned char a_private[32];

    for (int i = 0; i < 32; i++) {
        a_private[i] = a_buffer[i];
    }

    // Convert byte array to Uint256
    a = UINT256_FROM_BYTES(a_private);

    // Perform modular double
    result = modular_double(a);

    // Convert result back to bytes and copy to global memory
    unsigned char result_private[32];
    uint256_to_bytes(result, result_private);

    for (int i = 0; i < 32; i++) {
        result_buffer[i] = result_private[i];
    }
}