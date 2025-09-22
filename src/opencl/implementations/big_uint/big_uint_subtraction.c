#include "src/opencl/headers/big_uint/big_uint_subtraction.h"

inline void uint256_subtraction(const Uint256 *a, const Uint256 *b, Uint256 *result)
{
    // inplace unsafe

    result->limbs[3] = a->limbs[3] - b->limbs[3];
    unsigned int borrow = (a->limbs[3] < b->limbs[3]);

    result->limbs[2] = a->limbs[2] - b->limbs[2] - borrow;
    borrow = (a->limbs[2] < b->limbs[2]) | ((a->limbs[2] == b->limbs[2]) & borrow);

    result->limbs[1] = a->limbs[1] - b->limbs[1] - borrow;
    borrow = (a->limbs[1] < b->limbs[1]) | ((a->limbs[1] == b->limbs[1]) & borrow);

    result->limbs[0] = a->limbs[0] - b->limbs[0] - borrow;
}

inline void uint256_subtraction_with_underflow_flag(const Uint256 *a, const Uint256 *b,
                                                    Uint256 *result, unsigned int *underflowFlag)
{
    // inplace unsafe
    // The underflow flag is set basically if a < b

    result->limbs[3] = a->limbs[3] - b->limbs[3];
    unsigned int borrow = (a->limbs[3] < b->limbs[3]);

    result->limbs[2] = a->limbs[2] - b->limbs[2] - borrow;
    borrow = (a->limbs[2] < b->limbs[2]) | ((a->limbs[2] == b->limbs[2]) & borrow);

    result->limbs[1] = a->limbs[1] - b->limbs[1] - borrow;
    borrow = (a->limbs[1] < b->limbs[1]) | ((a->limbs[1] == b->limbs[1]) & borrow);

    result->limbs[0] = a->limbs[0] - b->limbs[0] - borrow;

    *underflowFlag = (a->limbs[0] < b->limbs[0]) | ((a->limbs[0] == b->limbs[0]) & borrow);
}