#include "src/opencl/headers/big_uint/big_uint_shift.h"
#include "src/opencl/headers/big_uint/big_uint_from_bytes.h"
#include "src/opencl/headers/big_uint/big_uint_to_bytes.h"

__kernel void uint256_shift_right_kernel(
    __global uchar *input_result)
{

    uchar local_input[32];
    uchar local_result[32];

    for (uchar i = 0; i < 32; i++)
    {
        local_input[i] = input_result[i];
    }

    Uint256 x = UINT256_FROM_BYTES(local_input);

    x = uint256_shift_right(x);

    uint256_to_bytes(x, local_result);

    for (uchar i = 0; i < 32; i++)
    {
        input_result[i] = local_result[i];
    }
}