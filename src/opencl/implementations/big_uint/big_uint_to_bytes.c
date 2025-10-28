#include "src/opencl/headers/big_uint/big_uint_to_bytes.h"

inline void uint256_to_bytes(const Uint256 a, unsigned char *restrict result)
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

inline void uint320_to_bytes(const Uint320 a, unsigned char *restrict result)
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

    result[32] = a.limbs[4] >> 56;
    result[33] = a.limbs[4] >> 48;
    result[34] = a.limbs[4] >> 40;
    result[35] = a.limbs[4] >> 32;
    result[36] = a.limbs[4] >> 24;
    result[37] = a.limbs[4] >> 16;
    result[38] = a.limbs[4] >> 8;
    result[39] = a.limbs[4];
}

inline void uint512_to_bytes(const Uint512 a, unsigned char *restrict result)
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

    result[32] = a.limbs[4] >> 56;
    result[33] = a.limbs[4] >> 48;
    result[34] = a.limbs[4] >> 40;
    result[35] = a.limbs[4] >> 32;
    result[36] = a.limbs[4] >> 24;
    result[37] = a.limbs[4] >> 16;
    result[38] = a.limbs[4] >> 8;
    result[39] = a.limbs[4];

    result[40] = a.limbs[5] >> 56;
    result[41] = a.limbs[5] >> 48;
    result[42] = a.limbs[5] >> 40;
    result[43] = a.limbs[5] >> 32;
    result[44] = a.limbs[5] >> 24;
    result[45] = a.limbs[5] >> 16;
    result[46] = a.limbs[5] >> 8;
    result[47] = a.limbs[5];

    result[48] = a.limbs[6] >> 56;
    result[49] = a.limbs[6] >> 48;
    result[50] = a.limbs[6] >> 40;
    result[51] = a.limbs[6] >> 32;
    result[52] = a.limbs[6] >> 24;
    result[53] = a.limbs[6] >> 16;
    result[54] = a.limbs[6] >> 8;
    result[55] = a.limbs[6];

    result[56] = a.limbs[7] >> 56;
    result[57] = a.limbs[7] >> 48;
    result[58] = a.limbs[7] >> 40;
    result[59] = a.limbs[7] >> 32;
    result[60] = a.limbs[7] >> 24;
    result[61] = a.limbs[7] >> 16;
    result[62] = a.limbs[7] >> 8;
    result[63] = a.limbs[7];
}