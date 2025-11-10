#include "src/opencl/headers/hash/hash160.h"

__kernel void hash160_33_bytes_kernel(
    __global const uchar *input_buffer,
    __global uchar *output_buffer)
{
    uint gid = get_global_id(0);
    uint offset_in = gid * 33;
    uint offset_out = gid * 20;

    uchar input[33];
    for (int i = 0; i < 33; i++)
    {
        input[i] = input_buffer[offset_in + i];
    }

    uchar output[20];
    hash160_33(input, output);

    for (int i = 0; i < 20; i++)
    {
        output_buffer[offset_out + i] = output[i];
    }
}
