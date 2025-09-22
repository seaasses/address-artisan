#include "src/opencl/headers/big_uint/big_uint_shift.h"
#include "src/opencl/headers/big_uint/big_uint_from_bytes.h"
#include "src/opencl/headers/big_uint/big_uint_to_bytes.h"

__kernel void uint256_shift_right_kernel(
    __global unsigned char *input_result)
{

    unsigned char local_input[32];
    unsigned char local_result[32];

    for (unsigned char i = 0; i < 32; i++)
    {
        local_input[i] = input_result[i];
    }

    Uint256 x = uint256_from_bytes(local_input);

    uint256_shift_right(&x, &x);

    uint256_to_bytes(x, local_result);

    for (unsigned char i = 0; i < 32; i++)
    {
        input_result[i] = local_result[i];
    }
}