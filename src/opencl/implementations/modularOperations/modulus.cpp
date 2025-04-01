#include "src/opencl/headers/modularOperations/modulus.h"
#include "src/opencl/definitions/secp256k1.h"
#include "src/opencl/headers/uint256/subtraction.h"

#pragma inline
const unsigned char is_outside_secp256k1_space(const UInt256 a)
{
  const UInt256 p = SECP256K1_P;
#pragma unroll
  for (unsigned char i = 0; i < 4; ++i)
  {
    if (a.limbs[i] > p.limbs[i])
    {
      return 1; // a is greater than p
    }
    else if (a.limbs[i] < p.limbs[i])
    {
      return 0; // a is less than p
    }
  }

  return 1; // a is equal to p
}

#pragma inline
const UInt256 modulus(const UInt256 a)
{
  if (is_outside_secp256k1_space(a))
  {
    return uint256Subtraction(a, SECP256K1_P);
  }

  return a;
}
