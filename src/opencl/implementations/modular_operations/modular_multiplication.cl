#include "src/opencl/headers/modular_operations/modular_multiplication.cl.h"
#include "src/opencl/definitions/secp256k1.cl.h"
#include "src/opencl/headers/big_uint/big_uint_multiplication.cl.h"
#include "src/opencl/headers/big_uint/big_uint_subtraction.cl.h"
#include "src/opencl/headers/big_uint/ulong_operations.cl.h"
#include "src/opencl/headers/big_uint/big_uint_addition.cl.h"
#include "src/opencl/structs/structs.cl.h"

inline Uint256 modular_multiplication(const Uint256 a, const Uint256 b)
{
    Uint256 result;
    Uint512 tmp_0;
    Uint320 tmp_1;
    Uint320 tmp_2;
    uint final_carry;

    // MULTIPLICATION = 512 bits. Normal uint multiplication
    tmp_0 = uint256_multiplication(a, b);
    // NEED TO REDUCE

    // FIRST FOLD
    // X = tmp_0_high << 256 + tmp_0_low
    Uint256 tmp_0_low_as_uint256 = {.limbs = {tmp_0.limbs[0], tmp_0.limbs[1], tmp_0.limbs[2], tmp_0.limbs[3]}};
    tmp_1 = uint256_ulong_multiplication(tmp_0_low_as_uint256, P_FOLDING_CONSTANT);
    // tmp_1 = tmp_0_high * P_FOLDING_CONSTANT
    tmp_2 = uint320_uint256_addition(tmp_1, *((Uint256 *)(&tmp_0.limbs[4])));
    // tmp_2 = X = 290 bits (on uint320)
    // X = 34 bits high (ms limb)  << 256 + tmp_2_low

    // SECOND FOLD
    // TODO: add 256 bits + 128 bits addition

    UINT64_MULTIPLICATION(P_FOLDING_CONSTANT, tmp_2.limbs[0], tmp_0.limbs[2], tmp_0.limbs[3]);
    tmp_0.limbs[0] = 0; // and i will not need to do this
    tmp_0.limbs[1] = 0; // and this

    Uint256 tmp_2_as_uint256 = {.limbs = {tmp_2.limbs[1], tmp_2.limbs[2], tmp_2.limbs[3], tmp_2.limbs[4]}};
    Uint256 tmp_0_as_uint256 = {.limbs = {tmp_0.limbs[0], tmp_0.limbs[1], tmp_0.limbs[2], tmp_0.limbs[3]}};

    Uint256WithOverflow addition_result = uint256_addition_with_overflow_flag(tmp_2_as_uint256, tmp_0_as_uint256);

    tmp_1.limbs[0] = addition_result.result.limbs[0];
    tmp_1.limbs[1] = addition_result.result.limbs[1];
    tmp_1.limbs[2] = addition_result.result.limbs[2];
    tmp_1.limbs[3] = addition_result.result.limbs[3];
    final_carry = addition_result.overflow;

    // CONDITIONAL SUBTRACTION

    ulong to_subtract_mask = 0;
    to_subtract_mask |= (tmp_1.limbs[0] > SECP256K1_P_0);
    to_subtract_mask |= ((tmp_1.limbs[0] == SECP256K1_P_0) & (tmp_1.limbs[1] > SECP256K1_P_1));
    to_subtract_mask |= ((tmp_1.limbs[0] == SECP256K1_P_0) & (tmp_1.limbs[1] == SECP256K1_P_1) & (tmp_1.limbs[2] > SECP256K1_P_2));
    to_subtract_mask |= ((tmp_1.limbs[0] == SECP256K1_P_0) & (tmp_1.limbs[1] == SECP256K1_P_1) & (tmp_1.limbs[2] == SECP256K1_P_2) & (tmp_1.limbs[3] >= SECP256K1_P_3));

    to_subtract_mask = -((ulong)final_carry | to_subtract_mask);

    Uint256 tmp_1_as_uint256 = {.limbs = {tmp_1.limbs[0], tmp_1.limbs[1], tmp_1.limbs[2], tmp_1.limbs[3]}};
    Uint256 to_subtract = {.limbs = {
        SECP256K1_P_0 & to_subtract_mask,
        SECP256K1_P_1 & to_subtract_mask,
        SECP256K1_P_2 & to_subtract_mask,
        SECP256K1_P_3 & to_subtract_mask
    }};

    result = uint256_subtraction(tmp_1_as_uint256, to_subtract);

    return result;
}
