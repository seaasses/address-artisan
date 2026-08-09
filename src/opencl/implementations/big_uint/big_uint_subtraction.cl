#include "src/opencl/headers/big_uint/big_uint_subtraction.cl.h"

inline Uint256WithUnderflow uint256_subtraction_with_underflow_flag(const Uint256 a, const Uint256 b)
{
    Uint256WithUnderflow result_with_underflow;
    uint borrow = 0;
#pragma unroll
    for (int i = 7; i >= 0; i--)
    {
        uint diff = a.limbs[i] - b.limbs[i];
        uint borrow1 = a.limbs[i] < b.limbs[i];
        uint out = diff - borrow;
        uint borrow2 = diff < borrow;
        result_with_underflow.result.limbs[i] = out;
        borrow = borrow1 | borrow2;
    }
    result_with_underflow.underflow = borrow;
    return result_with_underflow;
}

inline Uint256 uint256_subtraction(const Uint256 a, const Uint256 b)
{
    Uint256 result;
    uint borrow = 0;
#pragma unroll
    for (int i = 7; i >= 0; i--)
    {
        uint diff = a.limbs[i] - b.limbs[i];
        uint borrow1 = a.limbs[i] < b.limbs[i];
        uint out = diff - borrow;
        uint borrow2 = diff < borrow;
        result.limbs[i] = out;
        borrow = borrow1 | borrow2;
    }
    return result;
}
