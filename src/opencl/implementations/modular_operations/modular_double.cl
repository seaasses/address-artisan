#include "src/opencl/headers/modular_operations/modular_double.cl.h"
#include "src/opencl/headers/big_uint/big_uint_shift.cl.h"
#include "src/opencl/headers/big_uint/big_uint_subtraction.cl.h"
#include "src/opencl/definitions/secp256k1.cl.h"

inline Uint256 modular_double(const Uint256 a)
{
  uint msb = a.limbs[0] >> 31; // top bit of the 256-bit value (MSW is 32 bits)
  Uint256 tmp = uint256_shift_left(a);

  // Subtract p if the doubled value overflowed 2^256 (msb set) OR landed
  // outside the field (>= p). Branchless MSW->LSW compare over 8 limbs.
  uint is_outside = 0;
  uint eq = 1;
#define AA_CMP(i)                                              \
  do                                                           \
  {                                                            \
    is_outside |= (eq & (tmp.limbs[i] > (uint)SECP256K1_P_##i)); \
    eq &= (tmp.limbs[i] == (uint)SECP256K1_P_##i);             \
  } while (0)
  AA_CMP(0); AA_CMP(1); AA_CMP(2); AA_CMP(3);
  AA_CMP(4); AA_CMP(5); AA_CMP(6); AA_CMP(7);
#undef AA_CMP

  uint mask = -(msb | is_outside | eq);

  const Uint256 to_subtract = {.limbs = {
                                  (uint)SECP256K1_P_0 & mask,
                                  (uint)SECP256K1_P_1 & mask,
                                  (uint)SECP256K1_P_2 & mask,
                                  (uint)SECP256K1_P_3 & mask,
                                  (uint)SECP256K1_P_4 & mask,
                                  (uint)SECP256K1_P_5 & mask,
                                  (uint)SECP256K1_P_6 & mask,
                                  (uint)SECP256K1_P_7 & mask,
                              }};

  return uint256_subtraction(tmp, to_subtract);
}
