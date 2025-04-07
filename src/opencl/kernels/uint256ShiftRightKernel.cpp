#include "src/opencl/headers/uint256/fromBytes.h"
#include "src/opencl/headers/uint256/toBytes.h"
#include "src/opencl/headers/uint256/shiftRight.h"

__kernel void uint256ShiftRightKernel(
    __global unsigned char *input_a,
    __global unsigned char *result)
{

  unsigned char local_a[32];
  unsigned char local_result[32];

  for (unsigned char i = 0; i < 32; i++)
  {
    local_a[i] = input_a[i];
  }

  const UInt256 a = uint256FromBytes(local_a);

  UInt256 local_class_result;

  uint256ShiftRight(&a, &a); // inplace safe
  local_class_result = a;

  uint256ToBytes(local_class_result, local_result);

  for (unsigned char i = 0; i < 32; i++)
  {
    result[i] = local_result[i];
  }
}
