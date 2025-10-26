#include "src/opencl/headers/big_uint/big_uint_subtraction.h"

inline Uint256WithUnderflow uint256_subtraction_with_underflow_flag(Uint256 a, Uint256 b)
{
    Uint256WithUnderflow result_with_underflow;

    result_with_underflow.result.limbs[3] = a.limbs[3] - b.limbs[3];
    unsigned int borrow = (a.limbs[3] < b.limbs[3]);

    result_with_underflow.result.limbs[2] = a.limbs[2] - b.limbs[2] - borrow;
    borrow = (a.limbs[2] < b.limbs[2]) | ((a.limbs[2] == b.limbs[2]) & borrow);

    result_with_underflow.result.limbs[1] = a.limbs[1] - b.limbs[1] - borrow;
    borrow = (a.limbs[1] < b.limbs[1]) | ((a.limbs[1] == b.limbs[1]) & borrow);

    result_with_underflow.result.limbs[0] = a.limbs[0] - b.limbs[0] - borrow;

    result_with_underflow.underflow = (a.limbs[0] < b.limbs[0]) | ((a.limbs[0] == b.limbs[0]) & borrow);

    return result_with_underflow;
}

inline Uint256 uint256_subtraction(Uint256 a, Uint256 b)
{
    Uint256 result;

    result.limbs[3] = a.limbs[3] - b.limbs[3];
    unsigned int borrow = (a.limbs[3] < b.limbs[3]);

    result.limbs[2] = a.limbs[2] - b.limbs[2] - borrow;
    borrow = (a.limbs[2] < b.limbs[2]) | ((a.limbs[2] == b.limbs[2]) & borrow);

    result.limbs[1] = a.limbs[1] - b.limbs[1] - borrow;
    borrow = (a.limbs[1] < b.limbs[1]) | ((a.limbs[1] == b.limbs[1]) & borrow);

    result.limbs[0] = a.limbs[0] - b.limbs[0] - borrow;

    return result;
}