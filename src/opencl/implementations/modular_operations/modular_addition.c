#include "src/opencl/headers/modular_operations/modular_addition.h"
#include "src/opencl/headers/big_uint/big_uint_subtraction.h"
#include "src/opencl/headers/big_uint/big_uint_addition.h"
#include "src/opencl/definitions/secp256k1.h"

inline void modular_addition(const Uint256 *a, const Uint256 *b, Uint256 *result)
{

  // inplace safe
  unsigned int overflow_flag;
  Uint256 tmp;

  uint256_addition_with_overflow_flag(a, b, &tmp, &overflow_flag);

  // cases:
  // 1. less than p : subtract 0 
  // 2. more than p-1 and less than 2^256 : subtract p
  // 3. more than 2^256: subtract p
  // I really do not need to call modulus and can save a subtraction - and turns the function inplace safe without using a temporary variable
  // SO: subtract p when outside secp256k1 space OR overflow_flag is true

  // TODO: create a function to do this. modulus and this (and probably others will) use this
  ulong to_subtract_mask = 0;
  to_subtract_mask |= (tmp.limbs[0] > SECP256K1_P_0);
  to_subtract_mask |= ((tmp.limbs[0] == SECP256K1_P_0) & (tmp.limbs[1] > SECP256K1_P_1));
  to_subtract_mask |= ((tmp.limbs[0] == SECP256K1_P_0) & (tmp.limbs[1] == SECP256K1_P_1) & (tmp.limbs[2] > SECP256K1_P_2));
  to_subtract_mask |= ((tmp.limbs[0] == SECP256K1_P_0) & (tmp.limbs[1] == SECP256K1_P_1) & (tmp.limbs[2] == SECP256K1_P_2) & (tmp.limbs[3] >= SECP256K1_P_3));

  to_subtract_mask = -(to_subtract_mask | ((ulong) overflow_flag));

  const Uint256 to_subtract = {.limbs = {
                                           SECP256K1_P_0 & to_subtract_mask,
                                           SECP256K1_P_1 & to_subtract_mask,
                                           SECP256K1_P_2 & to_subtract_mask,
                                           SECP256K1_P_3 & to_subtract_mask,
                                       }};

  uint256_subtraction(&tmp, &to_subtract, result);
}
