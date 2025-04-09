#include "src/opencl/headers/modularOperations/modularAddition.h"
#include "src/opencl/headers/modularOperations/modulus.h"
#include "src/opencl/headers/uint256/additionWithOverflowFlag.h"
#include "src/opencl/headers/uint256/subtraction.h"
#include "src/opencl/definitions/secp256k1.h"

#pragma inline
void modularAddition(const UInt256 *a, const UInt256 *b, UInt256 *result)
{
  // inplace safe
  unsigned int overflowFlag;
  UInt256 tmp;

  uint256AdditionWithOverflowFlag(a, b, &tmp, &overflowFlag);

  // cases:
  // 1. less than p : subtract 0 
  // 2. more than p-1 and less than 2^256 : subtract p
  // 3. more than 2^256: subtract p
  // I really do not need to call modulus and can save a subtraction - and turns the function inplace safe without using a temporary variable
  // SO: subtract p when outside secp256k1 space OR overflowFlag is true

  // TODO: create a function to do this. modulus and this (and probably others will) use this
  unsigned long isOutsideSecp256k1Space = 0;
  isOutsideSecp256k1Space |= (tmp.limbs[0] > SECP256K1_P_0);
  isOutsideSecp256k1Space |= ((tmp.limbs[0] == SECP256K1_P_0) & (tmp.limbs[1] > SECP256K1_P_1));
  isOutsideSecp256k1Space |= ((tmp.limbs[0] == SECP256K1_P_0) & (tmp.limbs[1] == SECP256K1_P_1) & (tmp.limbs[2] > SECP256K1_P_2));
  isOutsideSecp256k1Space |= ((tmp.limbs[0] == SECP256K1_P_0) & (tmp.limbs[1] == SECP256K1_P_1) & (tmp.limbs[2] == SECP256K1_P_2) & (tmp.limbs[3] >= SECP256K1_P_3));

  const unsigned long toSubtractMask = -(isOutsideSecp256k1Space | ((unsigned long) overflowFlag));

  const UInt256 toSubtract = (UInt256){.limbs = {
                                           SECP256K1_P_0 & toSubtractMask,
                                           SECP256K1_P_1 & toSubtractMask,
                                           SECP256K1_P_2 & toSubtractMask,
                                           SECP256K1_P_3 & toSubtractMask,
                                       }};

  uint256Subtraction(&tmp, &toSubtract, result);
}