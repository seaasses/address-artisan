#include "src/opencl/headers/big_uint/big_uint_addition.h"
#include "src/opencl/headers/big_uint/big_uint_from_bytes.h"
#include "src/opencl/headers/big_uint/big_uint_to_bytes.h"

__kernel void uint320_uint256_addition_kernel(
    __global unsigned char *input_a,
    __global unsigned char *input_b,
    __global unsigned char *result)
{

    unsigned char local_a[40];
    unsigned char local_b[32];
    unsigned char local_result[40];
    unsigned int local_overflow_flag;

    for (unsigned char i = 0; i < 40; i++)
    {
        local_a[i] = input_a[i];
    }
    for (unsigned char i = 0; i < 32; i++)
    {
        local_b[i] = input_b[i];
    }

    const Uint320 a = uint320_from_bytes(local_a);
    const Uint256 b = uint256_from_bytes(local_b);

    Uint320 local_class_result;

    uint320_uint256_addition(&a, &b, &local_class_result);

    uint320_to_bytes(local_class_result, local_result);

    for (unsigned char i = 0; i < 40; i++)
    {
        result[i] = local_result[i];
    }
}
