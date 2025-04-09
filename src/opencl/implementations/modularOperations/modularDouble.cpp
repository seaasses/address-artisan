#include "src/opencl/headers/modularOperations/modularDouble.h"
#include "src/opencl/headers/modularOperations/modulus.h"
#include "src/opencl/definitions/secp256k1.h"
#include "src/opencl/headers/uint256/shiftLeft.h"
#include "src/opencl/headers/uint256/subtraction.h"

#pragma inline
void modularDouble(const UInt256 *a, UInt256 *result) 
{
  // inplace safe

  UInt256 tmp;
  unsigned long tmpBool = a->limbs[0] >> 63; // now tmpBool is 1 if most significant bit is set, 0 otherwise
  uint256ShiftLeft(a, &tmp);

  // cases:
  // 1. result is less than p: subtract 0
  // 2. result is equal or greater than p but less than 2^256: subtract p
  // 3. result is greater than or equal to 2^256: subtract 2^256


  // so, if most significant bit is 1 (case 2 or 3), we need to subtract p
  // if result is outside of 0 secp256k1, subtract p too

  unsigned long isOutsideSecp256k1Space = 0;
  isOutsideSecp256k1Space |= (tmp.limbs[0] > SECP256K1_P_0);
  isOutsideSecp256k1Space |= ((tmp.limbs[0] == SECP256K1_P_0) & (tmp.limbs[1] > SECP256K1_P_1));
  isOutsideSecp256k1Space |= ((tmp.limbs[0] == SECP256K1_P_0) & (tmp.limbs[1] == SECP256K1_P_1) & (tmp.limbs[2] > SECP256K1_P_2));
  isOutsideSecp256k1Space |= ((tmp.limbs[0] == SECP256K1_P_0) & (tmp.limbs[1] == SECP256K1_P_1) & (tmp.limbs[2] == SECP256K1_P_2) & (tmp.limbs[3] >= SECP256K1_P_3));

  // TODO: test use tmpbool or do the -(|) on the 4 limbs
  tmpBool = -(tmpBool | isOutsideSecp256k1Space);

  const UInt256 toSubtract = {.limbs = {
                                  SECP256K1_P_0 & tmpBool,
                                  SECP256K1_P_1 & tmpBool,
                                  SECP256K1_P_2 & tmpBool,
                                  SECP256K1_P_3 & tmpBool,
                              }};

  uint256Subtraction(&tmp, &toSubtract, result);
}

