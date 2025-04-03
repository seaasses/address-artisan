#include "src/opencl/headers/modularOperations/modularShiftLeft.h"
#include "src/opencl/headers/modularOperations/modulus.h"
#include "src/opencl/definitions/secp256k1.h"
#include "src/opencl/headers/uint256/shiftLeft.h"
#include "src/opencl/headers/uint256/subtraction.h"

#pragma inline
void modularShiftLeft(const UInt256 *a, UInt256 *result) // inplace safe
{
  const unsigned long toSubtractMask = -((((unsigned long)a->limbs[0]) & 0x8000000000000000ull) >> 63);
  const UInt256 toSubtract = {.limbs = {
                                  SECP256K1_P_0 & toSubtractMask,
                                  SECP256K1_P_1 & toSubtractMask,
                                  SECP256K1_P_2 & toSubtractMask,
                                  SECP256K1_P_3 & toSubtractMask,
                              }};

  UInt256 tmp;
  uint256ShiftLeft(a, &tmp);
  uint256Subtraction(&tmp, &toSubtract, result);
  modulus(result, result);
}
