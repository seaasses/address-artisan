#include "src/opencl/headers/modular_operations/modular_reduction.cl.h"
#include "src/opencl/headers/big_uint/big_uint_from_bytes.cl.h"
#include "src/opencl/headers/big_uint/big_uint_to_bytes.cl.h"

__kernel void modular_reduction_kernel(
    __global uchar *x_buffer,
    __global uchar *result_buffer)
{
    uchar x_private[64];

    for (int i = 0; i < 64; i++) {
        x_private[i] = x_buffer[i];
    }

    const Uint512 x = UINT512_FROM_BYTES(x_private);

    Uint256 result = modular_reduction(x);

    uchar result_private[32];
    uint256_to_bytes(result, result_private);

    for (int i = 0; i < 32; i++) {
        result_buffer[i] = result_private[i];
    }
}
