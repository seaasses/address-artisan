#include "src/opencl/headers/big_uint/big_uint_square.cl.h"
#include "src/opencl/headers/big_uint/big_uint_from_bytes.cl.h"
#include "src/opencl/headers/big_uint/big_uint_to_bytes.cl.h"

__kernel void uint256_square_kernel(
    __global uchar *input_a,
    __global uchar *result)
{

    uchar local_a[32];
    uchar local_result[64];

    for (uchar i = 0; i < 32; i++)
    {
        local_a[i] = input_a[i];
    }

    const Uint256 a = UINT256_FROM_BYTES(local_a);

    Uint512 local_class_result = uint256_square(a);

    uint512_to_bytes(local_class_result, local_result);

    for (uchar i = 0; i < 64; i++)
    {
        result[i] = local_result[i];
    }
}
