#include "src/opencl/headers/uint256/fromBytes.h"
#include "src/opencl/headers/uint256/toBytes.h"
#include "src/opencl/headers/uint256/additionWithOverflowFlag.h"
#include "src/opencl/headers/uint256/subtractionWithUnderflowFlag.h"
#include "src/opencl/headers/uint256/subtraction.h"
#include "src/opencl/headers/uint256/shiftLeft.h"
#include "src/opencl/headers/uint256/shiftRight.h"

__kernel void uint256Operations(
    __global unsigned char *input_a,
    __global unsigned char *input_b,
    unsigned char operation,
    __global unsigned char *result,
    __global unsigned char *booleanFlag)
{

  // THIS DOES NOT NEED TO BE FAST, IT IS ONLY USED FOR TESTING

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
  bool localBooleanFlag;

  if (operation == 0)
  {
    uint256AdditionWithOverflowFlag(&a, &b, &local_class_result, // inplace unsafe
                                    &localBooleanFlag);
  }
  else if (operation == 1)
  {
    uint256Subtraction(&a, &b, &local_class_result); // inplace unsafe
  }
  else if (operation == 2)
  {
    uint256ShiftLeft(&a, &a); // inplace safe
    local_class_result = a;
  }
  else if (operation == 3)
  {
    uint256ShiftRight(&a, &a); // inplace safe
    local_class_result = a;
  }
  else if (operation == 4)
  {
    uint256SubtractionWithUnderflowFlag(&a, &b, &local_class_result,
                                        &localBooleanFlag);
  }

  uint256ToBytes(local_class_result, local_result);

  for (unsigned char i = 0; i < 32; i++)
  {
    result[i] = local_result[i];
  }
  *booleanFlag = (localBooleanFlag == true) ? 1 : 0;
}
