#include "src/opencl/headers/modularOperations/modularSubtraction.h"
#include "src/opencl/headers/uint256/addition.h"
#include "src/opencl/definitions/secp256k1P.h"
#include "src/opencl/headers/uint256/subtractionWithUnderflowFlag.h"

#pragma inline
const UInt256 modularSubtraction(const UInt256 a, const UInt256 b)
{
  const UInt256 result;
  bool underflowFlag;
  uint256SubtractionWithUnderflowFlag(&a, &b, &result, &underflowFlag);

  if (underflowFlag)
  {
    return uint256Addition(result, SECP256K1_P);
  }

  return result; // no need to modulus here
}