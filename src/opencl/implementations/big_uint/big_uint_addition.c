#include "src/opencl/headers/big_uint/big_uint_addition.h"

inline void uint256_addition(const Uint256 *a, const Uint256 *b,
                             Uint256 *result)
{
    // inplace unsafe
    result->limbs[3] = a->limbs[3] + b->limbs[3];
    unsigned int carry = result->limbs[3] < a->limbs[3];

    result->limbs[2] = a->limbs[2] + b->limbs[2] + carry;
    carry = (result->limbs[2] < a->limbs[2]) | ((result->limbs[2] == a->limbs[2]) & carry);

    result->limbs[1] = a->limbs[1] + b->limbs[1] + carry;
    carry = (result->limbs[1] < a->limbs[1]) | ((result->limbs[1] == a->limbs[1]) & carry);

    result->limbs[0] = a->limbs[0] + b->limbs[0] + carry;
}
