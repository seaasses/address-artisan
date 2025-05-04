#include "src/opencl/headers/big_uint/uint256_addition.h"

void uint256_to_bytes(const Uint256 a, unsigned char *result)
{
    result[0] = a.limbs[0] >> 56;
    result[1] = a.limbs[0] >> 48;
    result[2] = a.limbs[0] >> 40;
    result[3] = a.limbs[0] >> 32;
    result[4] = a.limbs[0] >> 24;
    result[5] = a.limbs[0] >> 16;
    result[6] = a.limbs[0] >> 8;
    result[7] = a.limbs[0];

    result[8] = a.limbs[1] >> 56;
    result[9] = a.limbs[1] >> 48;
    result[10] = a.limbs[1] >> 40;
    result[11] = a.limbs[1] >> 32;
    result[12] = a.limbs[1] >> 24;
    result[13] = a.limbs[1] >> 16;
    result[14] = a.limbs[1] >> 8;
    result[15] = a.limbs[1];

    result[16] = a.limbs[2] >> 56;
    result[17] = a.limbs[2] >> 48;
    result[18] = a.limbs[2] >> 40;
    result[19] = a.limbs[2] >> 32;
    result[20] = a.limbs[2] >> 24;
    result[21] = a.limbs[2] >> 16;
    result[22] = a.limbs[2] >> 8;
    result[23] = a.limbs[2];

    result[24] = a.limbs[3] >> 56;
    result[25] = a.limbs[3] >> 48;
    result[26] = a.limbs[3] >> 40;
    result[27] = a.limbs[3] >> 32;
    result[28] = a.limbs[3] >> 24;
    result[29] = a.limbs[3] >> 16;
    result[30] = a.limbs[3] >> 8;
    result[31] = a.limbs[3];
}

Uint256 uint256_from_bytes(const unsigned char *input)
{
    return (Uint256){(((unsigned long)(input[0]) << 56) | ((unsigned long)(input[1]) << 48) |
                      ((unsigned long)(input[2]) << 40) | ((unsigned long)(input[3]) << 32) |
                      ((unsigned long)(input[4]) << 24) | ((unsigned long)(input[5]) << 16) |
                      ((unsigned long)(input[6]) << 8) | ((unsigned long)(input[7]))),
                     (((unsigned long)(input[8]) << 56) | ((unsigned long)(input[9]) << 48) |
                      ((unsigned long)(input[10]) << 40) | ((unsigned long)(input[11]) << 32) |
                      ((unsigned long)(input[12]) << 24) | ((unsigned long)(input[13]) << 16) |
                      ((unsigned long)(input[14]) << 8) | ((unsigned long)(input[15]))),
                     (((unsigned long)(input[16]) << 56) | ((unsigned long)(input[17]) << 48) |
                      ((unsigned long)(input[18]) << 40) | ((unsigned long)(input[19]) << 32) |
                      ((unsigned long)(input[20]) << 24) | ((unsigned long)(input[21]) << 16) |
                      ((unsigned long)(input[22]) << 8) | ((unsigned long)(input[23]))),
                     (((unsigned long)(input[24]) << 56) | ((unsigned long)(input[25]) << 48) |
                      ((unsigned long)(input[26]) << 40) | ((unsigned long)(input[27]) << 32) |
                      ((unsigned long)(input[28]) << 24) | ((unsigned long)(input[29]) << 16) |
                      ((unsigned long)(input[30]) << 8) | ((unsigned long)(input[31])))};
};

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
