#include "src/opencl/headers/modularOperations/modularAddition.h"
#include "src/opencl/headers/modularOperations/modulus.h"
#include "src/opencl/headers/uint256/additionWithOverflowFlag.h"
#include "src/opencl/headers/uint256/subtraction.h"
#include "src/opencl/definitions/secp256k1.h"

#pragma inline
void modularAddition(const UInt256 *a, const UInt256 *b, UInt256 *result) // inplace unsafe
{
  bool overflowFlag;
  UInt256 tmp;

  uint256AdditionWithOverflowFlag(a, b, result, &overflowFlag);

  const unsigned long toSubtractMask = -((unsigned long)overflowFlag);

  const UInt256 toSubtract = (UInt256){.limbs = {
                                           SECP256K1_P_0 & toSubtractMask,
                                           SECP256K1_P_1 & toSubtractMask,
                                           SECP256K1_P_2 & toSubtractMask,
                                           SECP256K1_P_3 & toSubtractMask,
                                       }};

  uint256Subtraction(result, &toSubtract, &tmp);

  modulus(&tmp, result);
}