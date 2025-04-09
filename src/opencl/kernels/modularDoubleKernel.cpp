#include "src/opencl/headers/uint256/fromBytes.h"
#include "src/opencl/headers/uint256/toBytes.h"
#include "src/opencl/headers/modularOperations/modularDouble.h"


__kernel void modularDoubleKernel(
    __global unsigned char *inputA,
    __global unsigned char *result)
{

  unsigned char localA[32];
  unsigned char localResult[32];

  for (unsigned char i = 0; i < 32; i++)
  {
    localA[i] = inputA[i];
  }

  const UInt256 a = uint256FromBytes(localA);

  UInt256 localUint256Result;

  modularDouble(&a, &a); // inplace safe
  localUint256Result = a;

  uint256ToBytes(localUint256Result, localResult);

  for (unsigned char i = 0; i < 32; i++)
  {
    result[i] = localResult[i];
  }
}
