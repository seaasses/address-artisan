#include "src/opencl/headers/modular_operations/modular_subtraction.h"
#include "src/opencl/definitions/secp256k1.h"
#include "src/opencl/headers/big_uint/big_uint_addition.h"
#include "src/opencl/headers/big_uint/big_uint_subtraction.h"

inline Uint256 modular_subtraction(const Uint256 a, const Uint256 b)
{
    Uint256WithUnderflow subtraction_result = uint256_subtraction_with_underflow_flag(a, b);
    Uint256 tmp = subtraction_result.result;
    unsigned int underflow_flag = subtraction_result.underflow;

    ulong mask_to_sum = -((ulong)underflow_flag);
    const Uint256 to_sum = {.limbs = {
                                SECP256K1_P_0 & mask_to_sum,
                                SECP256K1_P_1 & mask_to_sum,
                                SECP256K1_P_2 & mask_to_sum,
                                SECP256K1_P_3 & mask_to_sum,
                            }};

    Uint256 result = uint256_addition(tmp, to_sum);
    return result;
}
