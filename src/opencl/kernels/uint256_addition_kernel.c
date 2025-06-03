#include "src/opencl/headers/big_uint/big_uint_addition.h"
#include "src/opencl/headers/big_uint/uint256_from_bytes.h"
#include "src/opencl/headers/big_uint/uint256_to_bytes.h"


__kernel void uint256_addition_kernel(
    __global unsigned char *input_a,
    __global unsigned char *input_b,
    __global unsigned char *result)
{

    unsigned char local_a[32];
    unsigned char local_b[32];
    unsigned char local_result[32];

    for (unsigned char i = 0; i < 32; i++)
    {
        local_a[i] = input_a[i];
        local_b[i] = input_b[i];
    }

    const Uint256 a = uint256_from_bytes(local_a);
    const Uint256 b = uint256_from_bytes(local_b);

    Uint256 local_class_result;

    uint256_addition(&a, &b, &local_class_result);

    uint256_to_bytes(local_class_result, local_result);

    for (unsigned char i = 0; i < 32; i++)
    {
        result[i] = local_result[i];
    }
}
