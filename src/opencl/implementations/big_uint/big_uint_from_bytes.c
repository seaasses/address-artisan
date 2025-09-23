#include "src/opencl/headers/big_uint/big_uint_from_bytes.h"

Uint256 uint256_from_bytes(const unsigned char *input)
{
    return (Uint256){
        .limbs = {
            (((ulong)(input[0]) << 56) | ((ulong)(input[1]) << 48) |
             ((ulong)(input[2]) << 40) | ((ulong)(input[3]) << 32) |
             ((ulong)(input[4]) << 24) | ((ulong)(input[5]) << 16) |
             ((ulong)(input[6]) << 8) | ((ulong)(input[7]))),
            (((ulong)(input[8]) << 56) | ((ulong)(input[9]) << 48) |
             ((ulong)(input[10]) << 40) | ((ulong)(input[11]) << 32) |
             ((ulong)(input[12]) << 24) | ((ulong)(input[13]) << 16) |
             ((ulong)(input[14]) << 8) | ((ulong)(input[15]))),
            (((ulong)(input[16]) << 56) | ((ulong)(input[17]) << 48) |
             ((ulong)(input[18]) << 40) | ((ulong)(input[19]) << 32) |
             ((ulong)(input[20]) << 24) | ((ulong)(input[21]) << 16) |
             ((ulong)(input[22]) << 8) | ((ulong)(input[23]))),
            (((ulong)(input[24]) << 56) | ((ulong)(input[25]) << 48) |
             ((ulong)(input[26]) << 40) | ((ulong)(input[27]) << 32) |
             ((ulong)(input[28]) << 24) | ((ulong)(input[29]) << 16) |
             ((ulong)(input[30]) << 8) | ((ulong)(input[31])))}

    };
};

Uint320 uint320_from_bytes(const unsigned char *input)
{
    return (Uint320){
        .limbs = {
            (((ulong)(input[0]) << 56) | ((ulong)(input[1]) << 48) |
             ((ulong)(input[2]) << 40) | ((ulong)(input[3]) << 32) |
             ((ulong)(input[4]) << 24) | ((ulong)(input[5]) << 16) |
             ((ulong)(input[6]) << 8) | ((ulong)(input[7]))),

            (((ulong)(input[8]) << 56) | ((ulong)(input[9]) << 48) |
             ((ulong)(input[10]) << 40) | ((ulong)(input[11]) << 32) |
             ((ulong)(input[12]) << 24) | ((ulong)(input[13]) << 16) |
             ((ulong)(input[14]) << 8) | ((ulong)(input[15]))),
            (((ulong)(input[16]) << 56) | ((ulong)(input[17]) << 48) |
             ((ulong)(input[18]) << 40) | ((ulong)(input[19]) << 32) |
             ((ulong)(input[20]) << 24) | ((ulong)(input[21]) << 16) |
             ((ulong)(input[22]) << 8) | ((ulong)(input[23]))),
            (((ulong)(input[24]) << 56) | ((ulong)(input[25]) << 48) |
             ((ulong)(input[26]) << 40) | ((ulong)(input[27]) << 32) |
             ((ulong)(input[28]) << 24) | ((ulong)(input[29]) << 16) |
             ((ulong)(input[30]) << 8) | ((ulong)(input[31]))),
            (((ulong)(input[32]) << 56) | ((ulong)(input[33]) << 48) |
             ((ulong)(input[34]) << 40) | ((ulong)(input[35]) << 32) |
             ((ulong)(input[36]) << 24) | ((ulong)(input[37]) << 16) |
             ((ulong)(input[38]) << 8) | ((ulong)(input[39])))

        }

    };
};

ulong ulong_from_bytes(const unsigned char *input)
{
    return (((ulong)(input[0]) << 56) | ((ulong)(input[1]) << 48) |
            ((ulong)(input[2]) << 40) | ((ulong)(input[3]) << 32) |
            ((ulong)(input[4]) << 24) | ((ulong)(input[5]) << 16) |
            ((ulong)(input[6]) << 8) | ((ulong)(input[7])));
}

void bytes_to_uint256(const unsigned char *input, Uint256 *result)
{
    *result = uint256_from_bytes(input);
}

ulong bytes_to_ulong(const unsigned char *input)
{
    return ulong_from_bytes(input);
}