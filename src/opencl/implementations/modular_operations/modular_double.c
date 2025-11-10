#include "src/opencl/headers/modular_operations/modular_double.h"
#include "src/opencl/headers/big_uint/big_uint_shift.h"
#include "src/opencl/headers/big_uint/big_uint_subtraction.h"
#include "src/opencl/definitions/secp256k1.h"

inline Uint256 modular_double(const Uint256 a)
{
  ulong tmp_bool = a.limbs[0] >> 63; // now tmp_bool is 1 if most significant bit is set, 0 otherwise
  Uint256 tmp = uint256_shift_left(a);

  // cases:
  // 1. result is less than p: subtract 0
  // 2. result is equal or greater than p but less than 2^256: subtract p
  // 3. result is greater than or equal to 2^256: subtract 2^256

  // so, if most significant bit is 1 (case 2 or 3), we need to subtract p
  // if result is outside of 0 secp256k1, subtract p too

  ulong is_outside_secp256k1_space = 0;
  is_outside_secp256k1_space |= (tmp.limbs[0] > SECP256K1_P_0);
  is_outside_secp256k1_space |= ((tmp.limbs[0] == SECP256K1_P_0) & (tmp.limbs[1] > SECP256K1_P_1));
  is_outside_secp256k1_space |= ((tmp.limbs[0] == SECP256K1_P_0) & (tmp.limbs[1] == SECP256K1_P_1) & (tmp.limbs[2] > SECP256K1_P_2));
  is_outside_secp256k1_space |= ((tmp.limbs[0] == SECP256K1_P_0) & (tmp.limbs[1] == SECP256K1_P_1) & (tmp.limbs[2] == SECP256K1_P_2) & (tmp.limbs[3] >= SECP256K1_P_3));

  // TODO: test use tmpbool or do the -(|) on the 4 limbs
  tmp_bool = -(tmp_bool | is_outside_secp256k1_space);

  const Uint256 to_subtract = {.limbs = {
                                  SECP256K1_P_0 & tmp_bool,
                                  SECP256K1_P_1 & tmp_bool,
                                  SECP256K1_P_2 & tmp_bool,
                                  SECP256K1_P_3 & tmp_bool,
                              }};

  Uint256 result = uint256_subtraction(tmp, to_subtract);
  return result;
}

