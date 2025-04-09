#include "src/opencl/headers/uint256/fromBytes.h"
#include "src/opencl/headers/uint256/toBytes.h"
#include "src/opencl/headers/modularOperations/modularSubtraction.h"

__kernel void modularSubtractionKernel(
    __global unsigned char *inputA,
    __global unsigned char *inputB,
    __global unsigned char *result)
{

  unsigned char local_a[32];
  unsigned char local_b[32];
  unsigned char local_result[32];

  for (unsigned char i = 0; i < 32; i++)
  {
    local_a[i] = inputA[i];
    local_b[i] = inputB[i];
  }

  const UInt256 a = uint256FromBytes(local_a);
  const UInt256 b = uint256FromBytes(local_b);

  UInt256 local_class_result;

  modularSubtraction(&a, &b, &a); // inplace safe
  local_class_result = a;

  uint256ToBytes(local_class_result, local_result);

  for (unsigned char i = 0; i < 32; i++)
  {
    result[i] = local_result[i];
  }
}
