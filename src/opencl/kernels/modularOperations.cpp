#include "src/opencl/structs/structs.h"
#include "src/opencl/headers/uint256/fromBytes.h"
#include "src/opencl/headers/uint256/toBytes.h"
#include "src/opencl/headers/modularOperations/modularAddition.h"
#include "src/opencl/headers/modularOperations/modularMultiplication.h"
#include "src/opencl/headers/modularOperations/modularShiftLeft.h"
// #iinclude "src/opencl/headers/modularOperations/modularExponentiation.h"
#include "src/opencl/headers/modularOperations/modularSubtraction.h"

__kernel void modularOperations(__global unsigned char *a, __global unsigned char *b, unsigned char operation,
                                __global unsigned char *result)
{

  // THIS DOES NOT NEED TO BE FAST, IT IS ONLY USED FOR TESTING

  unsigned char local_a[32];
  unsigned char local_b[32];
  unsigned char local_result[32];
  UInt256 localResultUint256;

#pragma unroll
  for (unsigned char i = 0; i < 32; i++)
  {
    local_a[i] = a[i];
    local_b[i] = b[i];
  }
  const UInt256 a_as_uint256 = uint256FromBytes(local_a);
  const UInt256 b_as_uint256 = uint256FromBytes(local_b);

  //////////////////////////////////////
  if (operation == 0)
  {
    // simple integer modular addition between x1 and y1
    modularAddition(&a_as_uint256, &b_as_uint256, &localResultUint256);
  }
  else if (operation == 1)
  {
    //  simple integer modular multiplication between x1 and y1 using the
    //  russian peasant algorithm
    modularMultiplicationUsingRussianPeasant(&a_as_uint256, &b_as_uint256, &localResultUint256);
  }
  // else if (operation == 2)
  // {
  //   // modular exponentiation between x1 (base) and y1 (exponent)
  //   localResultUint256 = modularExponentiation(a_as_uint256, b_as_uint256);
  // }
  else if (operation == 3)
  {
    // modular subtraction between x1 and y1
    modularSubtraction(&a_as_uint256, &b_as_uint256, &localResultUint256);
  }
  else if (operation == 4)
  {
    // modulus operation
    modulus(&a_as_uint256, &localResultUint256);
  }
  else if (operation == 5)
  {
    // modular double
    modularShiftLeft(&a_as_uint256, &a_as_uint256);
    localResultUint256 = a_as_uint256;
  }

  uint256ToBytes(localResultUint256, local_result);

  // send result to the host
#pragma unroll
  for (unsigned char i = 0; i < 32; i++)
  {
    result[i] = local_result[i];
  }
}