#include "src/opencl/headers/modular_operations/modular_square.cl.h"
#include "src/opencl/headers/big_uint/big_uint_from_bytes.cl.h"
#include "src/opencl/headers/big_uint/big_uint_to_bytes.cl.h"

__kernel void modular_square_kernel(
    __global uchar *a_buffer,
    __global uchar *result_buffer)
{
    uchar a_private[32];

    for (int i = 0; i < 32; i++) {
        a_private[i] = a_buffer[i];
    }

    const Uint256 a = UINT256_FROM_BYTES(a_private);

    Uint256 result = modular_square(a);

    uchar result_private[32];
    uint256_to_bytes(result, result_private);

    for (int i = 0; i < 32; i++) {
        result_buffer[i] = result_private[i];
    }
}
