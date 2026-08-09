#include "src/opencl/headers/big_uint/big_uint_square.cl.h"
#include "src/opencl/headers/big_uint/big_uint_multiplication.cl.h"
#include "src/opencl/headers/big_uint/ulong_operations.cl.h"

// Like add_component_to_limb, but adds 2 * (a_limb * b_limb). Used for the
// off-diagonal terms of a square: a_i*a_j appears twice for i != j, so the
// 128-bit product is doubled before being accumulated. The bit shifted out
// of the doubled product lands two limbs up, in carry_high.
inline void add_component_to_limb_doubled(ulong a_limb, ulong b_limb,
                                          ulong *carry_high, ulong *carry_low, ulong *result_limb)
{
    ulong carry_tmp_low, carry_tmp_high, result_tmp;

    UINT64_MULTIPLICATION(a_limb, b_limb, carry_tmp_low, result_tmp);

    *carry_high += carry_tmp_low >> 63;
    carry_tmp_low = (carry_tmp_low << 1) | (result_tmp >> 63);
    result_tmp <<= 1;

    UINT64_SUM_WITH_OVERFLOW_FLAG(*carry_low, carry_tmp_low, *carry_low, carry_tmp_high);
    *carry_high += carry_tmp_high;
    UINT64_SUM_WITH_OVERFLOW_FLAG(*result_limb, result_tmp, *result_limb, carry_tmp_low);
    UINT64_SUM_WITH_OVERFLOW_FLAG(*carry_low, carry_tmp_low, *carry_low, carry_tmp_high);
    *carry_high += carry_tmp_high;
}

// Squaring needs only 10 unique 64x64 limb products instead of the 16 a
// general multiplication pays: 4 diagonal (a_i^2, added once) and 6
// off-diagonal (a_i*a_j, i != j, added doubled). Same accumulation scheme
// as uint256_multiplication, limb by limb from least significant (limbs[7])
// to most significant (limbs[0]).
inline Uint512 uint256_square(const Uint256 a)
{
    Uint512 result;
    ulong carry_high = 0;
    ulong carry_low;

    // limb 7: a3^2
    UINT64_MULTIPLICATION(a.limbs[3], a.limbs[3], carry_low, result.limbs[7]); // first limb set (OK)

    // limb 6: 2*a3*a2
    ////////////////////////////////////////////////////////////////////////////////

    result.limbs[6] = carry_low; // start with carry low
    carry_low = carry_high;
    carry_high = 0;
    add_component_to_limb_doubled(a.limbs[3], a.limbs[2], &carry_high, &carry_low, &result.limbs[6]);

    // limb 5: a2^2 + 2*a3*a1
    ////////////////////////////////////////////////////////////////////////////////

    result.limbs[5] = carry_low; // start with carry low
    carry_low = carry_high;
    carry_high = 0;
    add_component_to_limb(a.limbs[2], a.limbs[2], &carry_high, &carry_low, &result.limbs[5]);
    add_component_to_limb_doubled(a.limbs[3], a.limbs[1], &carry_high, &carry_low, &result.limbs[5]);

    // limb 4: 2*a3*a0 + 2*a2*a1
    result.limbs[4] = carry_low; // start with carry low
    carry_low = carry_high;
    carry_high = 0;
    add_component_to_limb_doubled(a.limbs[3], a.limbs[0], &carry_high, &carry_low, &result.limbs[4]);
    add_component_to_limb_doubled(a.limbs[2], a.limbs[1], &carry_high, &carry_low, &result.limbs[4]);

    // limb 3: a1^2 + 2*a2*a0
    result.limbs[3] = carry_low; // start with carry low
    carry_low = carry_high;
    carry_high = 0;
    add_component_to_limb(a.limbs[1], a.limbs[1], &carry_high, &carry_low, &result.limbs[3]);
    add_component_to_limb_doubled(a.limbs[2], a.limbs[0], &carry_high, &carry_low, &result.limbs[3]);

    // limb 2: 2*a1*a0
    result.limbs[2] = carry_low; // start with carry low
    carry_low = carry_high;
    carry_high = 0;
    add_component_to_limb_doubled(a.limbs[1], a.limbs[0], &carry_high, &carry_low, &result.limbs[2]);

    // limb 1: a0^2
    result.limbs[1] = carry_low; // start with carry low
    carry_low = carry_high;
    carry_high = 0;
    add_component_to_limb(a.limbs[0], a.limbs[0], &carry_high, &carry_low, &result.limbs[1]);

    // most significant limb
    result.limbs[0] = carry_low;

    return result;
}
