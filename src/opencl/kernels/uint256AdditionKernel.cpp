#include "src/opencl/headers/uint256/fromBytes.h"
#include "src/opencl/headers/uint256/toBytes.h"
#include "src/opencl/headers/uint256/addition.h"

__kernel void uint256AdditionKernel(
    __global unsigned char *input_a,
    __global unsigned char *input_b,
    __global unsigned char *result)
{

  unsigned char local_a[32];
  unsigned char local_b[32];
  unsigned char local_result[32];

  for (unsigned char i = 0; i < 32; i++)
  {
    local_a[i] = input_a[i];
    local_b[i] = input_b[i];
  }

  const UInt256 a = uint256FromBytes(local_a);
  const UInt256 b = uint256FromBytes(local_b);

  UInt256 local_class_result;

  uint256Addition(&a, &b, &local_class_result); // inplace unsafe

  uint256ToBytes(local_class_result, local_result);

  for (unsigned char i = 0; i < 32; i++)
  {
    result[i] = local_result[i];
  }
}
