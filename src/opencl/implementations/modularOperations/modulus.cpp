#include "src/opencl/headers/modularOperations/modulus.h"
#include "src/opencl/definitions/secp256k1.h"
#include "src/opencl/headers/uint256/subtraction.h"

#pragma inline
void modulus(const UInt256 *a, UInt256 *result)
{
  // check if the number is outside the secp256k1 space
  unsigned int isOutsideSecp256k1Space = 1;
  isOutsideSecp256k1Space = (a->limbs[0] > SECP256K1_P_0) | (isOutsideSecp256k1Space & ~(a->limbs[0] > SECP256K1_P_0));
  isOutsideSecp256k1Space = (a->limbs[1] > SECP256K1_P_1) | (isOutsideSecp256k1Space & ~(a->limbs[1] > SECP256K1_P_1));
  isOutsideSecp256k1Space = (a->limbs[2] > SECP256K1_P_2) | (isOutsideSecp256k1Space & ~(a->limbs[2] > SECP256K1_P_2));
  isOutsideSecp256k1Space = (a->limbs[3] > SECP256K1_P_3) | (isOutsideSecp256k1Space & ~(a->limbs[3] < SECP256K1_P_3));

  const unsigned long toSubtractMask = -((unsigned long)isOutsideSecp256k1Space);

  const UInt256 toSubtract = (UInt256){.limbs = {
                                           SECP256K1_P_0 & toSubtractMask,
                                           SECP256K1_P_1 & toSubtractMask,
                                           SECP256K1_P_2 & toSubtractMask,
                                           SECP256K1_P_3 & toSubtractMask,
                                       }};

  uint256Subtraction(a, &toSubtract, result);
}