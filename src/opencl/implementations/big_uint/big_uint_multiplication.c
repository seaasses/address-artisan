#include "src/opencl/headers/big_uint/big_uint_multiplication.h"
#include "src/opencl/headers/big_uint/ulong_operations.h"

inline void add_component_to_limb(ulong a_limb, ulong b_limb,
                                  ulong *carry_high, ulong *carry_low, ulong *result_limb)
{
    ulong carry_tmp_low, carry_tmp_high, result_tmp;

    UINT64_MULTIPLICATION(a_limb, b_limb, carry_tmp_low, result_tmp);

    UINT64_SUM_WITH_OVERFLOW_FLAG(*carry_low, carry_tmp_low, *carry_low, carry_tmp_high);
    *carry_high += carry_tmp_high;
    UINT64_SUM_WITH_OVERFLOW_FLAG(*result_limb, result_tmp, *result_limb, carry_tmp_low);
    UINT64_SUM_WITH_OVERFLOW_FLAG(*carry_low, carry_tmp_low, *carry_low, carry_tmp_high);
    *carry_high += carry_tmp_high;
}

inline Uint512 uint256_multiplication(Uint256 a, Uint256 b)
{
    Uint512 result;
    ulong carry_high = 0;
    ulong carry_low;

    // limb 7
    UINT64_MULTIPLICATION(a.limbs[3], b.limbs[3], carry_low, result.limbs[7]); // first limb set (OK)

    // limb 6
    ////////////////////////////////////////////////////////////////////////////////

    result.limbs[6] = carry_low; // start with carry low
    carry_low = carry_high;
    carry_high = 0;
    add_component_to_limb(a.limbs[3], b.limbs[2], &carry_high, &carry_low, &result.limbs[6]);
    add_component_to_limb(a.limbs[2], b.limbs[3], &carry_high, &carry_low, &result.limbs[6]);

    // limb 5
    ////////////////////////////////////////////////////////////////////////////////

    result.limbs[5] = carry_low; // start with carry low
    carry_low = carry_high;
    carry_high = 0;
    add_component_to_limb(a.limbs[1], b.limbs[3], &carry_high, &carry_low, &result.limbs[5]);
    add_component_to_limb(a.limbs[2], b.limbs[2], &carry_high, &carry_low, &result.limbs[5]);
    add_component_to_limb(a.limbs[3], b.limbs[1], &carry_high, &carry_low, &result.limbs[5]);

    // limb 4
    result.limbs[4] = carry_low; // start with carry low
    carry_low = carry_high;
    carry_high = 0;
    add_component_to_limb(a.limbs[3], b.limbs[0], &carry_high, &carry_low, &result.limbs[4]);
    add_component_to_limb(a.limbs[0], b.limbs[3], &carry_high, &carry_low, &result.limbs[4]);
    add_component_to_limb(a.limbs[1], b.limbs[2], &carry_high, &carry_low, &result.limbs[4]);
    add_component_to_limb(a.limbs[2], b.limbs[1], &carry_high, &carry_low, &result.limbs[4]);

    // limb 3
    result.limbs[3] = carry_low; // start with carry low
    carry_low = carry_high;
    carry_high = 0;
    add_component_to_limb(a.limbs[2], b.limbs[0], &carry_high, &carry_low, &result.limbs[3]);
    add_component_to_limb(a.limbs[0], b.limbs[2], &carry_high, &carry_low, &result.limbs[3]);
    add_component_to_limb(a.limbs[1], b.limbs[1], &carry_high, &carry_low, &result.limbs[3]);

    // limb 2
    result.limbs[2] = carry_low; // start with carry low
    carry_low = carry_high;
    carry_high = 0;
    add_component_to_limb(a.limbs[0], b.limbs[1], &carry_high, &carry_low, &result.limbs[2]);
    add_component_to_limb(a.limbs[1], b.limbs[0], &carry_high, &carry_low, &result.limbs[2]);

    // limb 1
    result.limbs[1] = carry_low; // start with carry low
    carry_low = carry_high;
    carry_high = 0;
    add_component_to_limb(a.limbs[0], b.limbs[0], &carry_high, &carry_low, &result.limbs[1]);

    // most significant limb
    result.limbs[0] = carry_low;

    return result;
}

inline Uint320 uint256_ulong_multiplication(Uint256 a, ulong b)
{
    Uint320 result;
    ulong carry_high = 0;
    ulong carry_low;

    UINT64_MULTIPLICATION(a.limbs[3], b, carry_low, result.limbs[4]); // first limb set (OK)

    result.limbs[3] = carry_low;
    carry_low = carry_high;
    carry_high = 0;
    add_component_to_limb(a.limbs[2], b, &carry_high, &carry_low, &result.limbs[3]);

    result.limbs[2] = carry_low;
    carry_low = carry_high;
    carry_high = 0;
    add_component_to_limb(a.limbs[1], b, &carry_high, &carry_low, &result.limbs[2]);

    result.limbs[1] = carry_low;
    carry_low = carry_high;
    carry_high = 0;
    add_component_to_limb(a.limbs[0], b, &carry_high, &carry_low, &result.limbs[1]);

    result.limbs[0] = carry_low;

    return result;
}