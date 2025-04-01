#include "src/opencl/headers/modularOperations/modularAddition.h"
#include "src/opencl/headers/modularOperations/modulus.h"
#include "src/opencl/headers/uint256/additionWithOverflowFlag.h"
#include "src/opencl/headers/uint256/subtraction.h"
#include "src/opencl/definitions/secp256k1.h"

#pragma inline
const UInt256 modularAddition(const UInt256 a, const UInt256 b)
{
  bool overflowFlag;
  const UInt256 result;

  uint256AdditionWithOverflowFlag(&a, &b, &result, &overflowFlag);

  if (overflowFlag)
  {
    return uint256Subtraction(result, SECP256K1_P);
  }

  return modulus(result);
}