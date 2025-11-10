#include "src/opencl/headers/big_uint/big_uint_addition.h"
#include "src/opencl/headers/big_uint/big_uint_from_bytes.h"
#include "src/opencl/headers/big_uint/big_uint_to_bytes.h"

__kernel void uint320_uint256_addition_kernel(
    __global uchar *input_a,
    __global uchar *input_b,
    __global uchar *result)
{

    uchar local_a[40];
    uchar local_b[32];
    uchar local_result[40];
    unsigned int local_overflow_flag;

    for (uchar i = 0; i < 40; i++)
    {
        local_a[i] = input_a[i];
    }
    for (uchar i = 0; i < 32; i++)
    {
        local_b[i] = input_b[i];
    }

    const Uint320 a = UINT320_FROM_BYTES(local_a);
    const Uint256 b = UINT256_FROM_BYTES(local_b);

    Uint320 local_class_result = uint320_uint256_addition(a, b);

    uint320_to_bytes(local_class_result, local_result);

    for (uchar i = 0; i < 40; i++)
    {
        result[i] = local_result[i];
    }
}
