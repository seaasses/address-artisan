#include "src/opencl/headers/big_uint/big_uint_subtraction.h"
#include "src/opencl/headers/big_uint/big_uint_from_bytes.h"
#include "src/opencl/headers/big_uint/big_uint_to_bytes.h"

__kernel void uint256_subtraction_with_underflow_flag_kernel(
    __global uchar *input_a,
    __global uchar *input_b,
    __global uchar *result,
    __global uint *underflow_flag)
{

    uchar local_a[32];
    uchar local_b[32];
    uchar local_result[32];

    for (uchar i = 0; i < 32; i++)
    {
        local_a[i] = input_a[i];
        local_b[i] = input_b[i];
    }

    const Uint256 a = UINT256_FROM_BYTES(local_a);
    const Uint256 b = UINT256_FROM_BYTES(local_b);

    Uint256WithUnderflow subtraction_result = uint256_subtraction_with_underflow_flag(a, b);

    uint256_to_bytes(subtraction_result.result, local_result);

    for (uchar i = 0; i < 32; i++)
    {
        result[i] = local_result[i];
    }

    *underflow_flag = subtraction_result.underflow;
}