#include "src/opencl/headers/uint256/fromBytes.h"
#include "src/opencl/headers/uint256/toBytes.h"
#include "src/opencl/headers/uint256/subtraction.h"

__kernel void uint256SubtractionKernel(
    __global unsigned char *inputA,
    __global unsigned char *inputB,
    __global unsigned char *result)
{

  unsigned char localA[32];
  unsigned char localB[32];
  unsigned char localResult[32];

  for (unsigned char i = 0; i < 32; i++)
  {
    localA[i] = inputA[i];
    localB[i] = inputB[i];
  }

  const UInt256 a = uint256FromBytes(localA);
  const UInt256 b = uint256FromBytes(localB);

  UInt256 localUint256Result;

  uint256Subtraction(&a, &b, &localUint256Result); // inplace unsafe

  uint256ToBytes(localUint256Result, localResult);

  for (unsigned char i = 0; i < 32; i++)
  {
    result[i] = localResult[i];
  }

}
