#include "src/opencl/headers/big_uint/big_uint_to_bytes.cl.h"

inline void uint256_to_bytes(const Uint256 a, uchar *result)
{
    for (int k = 0; k < 8; k++)
    {
        result[k * 4 + 0] = a.limbs[k] >> 24;
        result[k * 4 + 1] = a.limbs[k] >> 16;
        result[k * 4 + 2] = a.limbs[k] >> 8;
        result[k * 4 + 3] = a.limbs[k];
    }
}

inline void uint512_to_bytes(const Uint512 a, uchar *result)
{
    for (int k = 0; k < 16; k++)
    {
        result[k * 4 + 0] = a.limbs[k] >> 24;
        result[k * 4 + 1] = a.limbs[k] >> 16;
        result[k * 4 + 2] = a.limbs[k] >> 8;
        result[k * 4 + 3] = a.limbs[k];
    }
}
