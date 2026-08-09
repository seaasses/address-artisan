#include "src/opencl/headers/modular_operations/modular_addition.cl.h"
#include "src/opencl/headers/big_uint/big_uint_subtraction.cl.h"
#include "src/opencl/headers/big_uint/big_uint_addition.cl.h"
#include "src/opencl/definitions/secp256k1.cl.h"

inline Uint256 modular_addition(const Uint256 a, const Uint256 b)
{
  Uint256WithOverflow addition_result = uint256_addition_with_overflow_flag(a, b);
  Uint256 tmp = addition_result.result;
  uint overflow_flag = addition_result.overflow;

  // Subtract p when the sum is outside the field (>= p) OR overflowed 2^256.
  // Branchless MSW->LSW compare over 8 limbs.
  uint out_of_field = 0;
  uint eq = 1;
#define AA_CMP(i)                                                  \
  do                                                               \
  {                                                                \
    out_of_field |= (eq & (tmp.limbs[i] > (uint)SECP256K1_P_##i)); \
    eq &= (tmp.limbs[i] == (uint)SECP256K1_P_##i);                 \
  } while (0)
  AA_CMP(0); AA_CMP(1); AA_CMP(2); AA_CMP(3);
  AA_CMP(4); AA_CMP(5); AA_CMP(6); AA_CMP(7);
#undef AA_CMP
  // eq==1 means tmp == p exactly -> also subtract.
  uint to_subtract_mask = -(out_of_field | eq | overflow_flag);

  const Uint256 to_subtract = {.limbs = {
                                   (uint)SECP256K1_P_0 & to_subtract_mask,
                                   (uint)SECP256K1_P_1 & to_subtract_mask,
                                   (uint)SECP256K1_P_2 & to_subtract_mask,
                                   (uint)SECP256K1_P_3 & to_subtract_mask,
                                   (uint)SECP256K1_P_4 & to_subtract_mask,
                                   (uint)SECP256K1_P_5 & to_subtract_mask,
                                   (uint)SECP256K1_P_6 & to_subtract_mask,
                                   (uint)SECP256K1_P_7 & to_subtract_mask,
                               }};

  return uint256_subtraction(tmp, to_subtract);
}
