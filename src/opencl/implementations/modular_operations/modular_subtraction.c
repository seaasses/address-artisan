#include "src/opencl/headers/modular_operations/modular_subtraction.h"
#include "src/opencl/definitions/secp256k1.h"
#include "src/opencl/headers/big_uint/big_uint_addition.h"
#include "src/opencl/headers/big_uint/big_uint_subtraction.h"

inline void modular_subtraction(const Uint256 *a, const Uint256 *b, Uint256 *result)
{

    unsigned int underflow_flag;
    Uint256 tmp;

    uint256_subtraction_with_underflow_flag(a, b, &tmp, &underflow_flag);

    ulong mask_to_sum = -((ulong)underflow_flag);
    const Uint256 to_sum = {.limbs = {
                                SECP256K1_P_0 & mask_to_sum,
                                SECP256K1_P_1 & mask_to_sum,
                                SECP256K1_P_2 & mask_to_sum,
                                SECP256K1_P_3 & mask_to_sum,
                            }};

    uint256_addition(&tmp, &to_sum, result);
}
