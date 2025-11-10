#include "src/opencl/headers/big_uint/big_uint_multiplication.cl.h"
#include "src/opencl/headers/big_uint/big_uint_from_bytes.cl.h"
#include "src/opencl/headers/big_uint/big_uint_to_bytes.cl.h"

__kernel void uint256_multiplication_kernel(
    __global uchar *input_a,
    __global uchar *input_b,
    __global uchar *result)
{

    uchar local_a[32];
    uchar local_b[32];
    uchar local_result[64];

    for (uchar i = 0; i < 32; i++)
    {
        local_a[i] = input_a[i];
        local_b[i] = input_b[i];
    }

    const Uint256 a = UINT256_FROM_BYTES(local_a);
    const Uint256 b = UINT256_FROM_BYTES(local_b);

    Uint512 local_class_result = uint256_multiplication(a, b);

    uint512_to_bytes(local_class_result, local_result);

    for (uchar i = 0; i < 64; i++)
    {
        result[i] = local_result[i];
    }
}