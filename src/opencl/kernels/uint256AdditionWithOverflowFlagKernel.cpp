#include "src/opencl/headers/uint256/fromBytes.h"
#include "src/opencl/headers/uint256/toBytes.h"
#include "src/opencl/headers/uint256/additionWithOverflowFlag.h"

__kernel void uint256AdditionWithOverflowFlagKernel(
    __global unsigned char *inputA,
    __global unsigned char *inputB,
    __global unsigned char *overflowFlag,
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
  unsigned int localOverflowFlag;

  uint256AdditionWithOverflowFlag(&a, &b, &localUint256Result, &localOverflowFlag); // inplace unsafe

  uint256ToBytes(localUint256Result, localResult);

  for (unsigned char i = 0; i < 32; i++)
  {
    result[i] = localResult[i];
  }
  *overflowFlag = localOverflowFlag;

}
