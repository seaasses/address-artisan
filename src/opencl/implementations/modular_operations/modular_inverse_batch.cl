#include "src/opencl/headers/modular_operations/modular_inverse_batch.cl.h"
#include "src/opencl/headers/modular_operations/modular_inverse.cl.h"
#include "src/opencl/headers/modular_operations/modular_multiplication.cl.h"

// Montgomery's trick: invert `count` values with a SINGLE modular inverse
// (the ~270-multiplication addition chain) plus 3 * (count - 1) cheap
// multiplications, instead of one full inverse per value.
//
// In-place: values[i] is replaced by values[i]^-1 mod p. prefix_scratch
// must have room for `count` elements. count must be >= 1.
//
// A zero value poisons the whole batch (the running product becomes zero),
// just like inverting zero poisons a single inversion today: Z = 0 cannot
// happen for the valid curve points this is used on.
inline void modular_inverse_batch(Uint256 *values, Uint256 *prefix_scratch, const uint count)
{
    // prefix_scratch[i] = values[0] * values[1] * ... * values[i]
    prefix_scratch[0] = values[0];
#pragma unroll 1
    for (uint i = 1; i < count; i++)
    {
        prefix_scratch[i] = modular_multiplication(prefix_scratch[i - 1], values[i]);
    }

    // One single expensive inversion for the whole batch
    Uint256 inverse = modular_inverse(prefix_scratch[count - 1]);

    // Walk backwards, peeling one value off the running inverse at a time:
    //   inverse == (values[0] * ... * values[i])^-1
    //   values[i]^-1 = inverse * prefix_scratch[i - 1]
#pragma unroll 1
    for (uint i = count - 1; i > 0; i--)
    {
        const Uint256 original = values[i];
        values[i] = modular_multiplication(inverse, prefix_scratch[i - 1]);
        inverse = modular_multiplication(inverse, original);
    }
    values[0] = inverse;
}
