#include "src/opencl/headers/uint256/addition.h"

#pragma inline
void uint256Addition(const UInt256 *a, const UInt256 *b, UInt256 *result)
{ //inplace unsafe

        // TODO: we can use result->limbs[0] as the carry and save a variable. Test if this is faster
        result->limbs[3] = a->limbs[3] + b->limbs[3];
        unsigned int carry = result->limbs[3] < a->limbs[3];

        result->limbs[2] = a->limbs[2] + b->limbs[2] + carry;
        carry = (result->limbs[2] < a->limbs[2]) | ((result->limbs[2] == a->limbs[2]) & carry);

        result->limbs[1] = a->limbs[1] + b->limbs[1] + carry;
        carry = (result->limbs[1] < a->limbs[1]) | ((result->limbs[1] == a->limbs[1]) & carry);

        result->limbs[0] = a->limbs[0] + b->limbs[0] + carry;
}