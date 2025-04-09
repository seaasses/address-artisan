#include "src/opencl/headers/modularOperations/modularExponentiation.h"
#include "src/opencl/headers/modularOperations/modularMultiplication.h"
#include "src/opencl/headers/uint256/shiftRight.h"

#pragma inline
void modularExponentiation(const UInt256 *base, const UInt256 *exponent, UInt256 *result)
{
  // inplace semi sami. Safe when result = base

  // base will be < modulus, so no need to modulus before starting
  UInt256 localBase = *base;
  UInt256 tmp;
  UInt256 toMultiply;

  unsigned long limb;
  unsigned long toMultiplyMask;

  *result = (UInt256) {0, 0, 0, 1};

#pragma unroll
  for (int i = 3; i >= 0 ; --i)
  {
    limb = exponent->limbs[i];
  #pragma unroll
    for (unsigned int i = 0; i < 32; ++i)
    {
      toMultiplyMask = -(limb & 1);
      toMultiply = (UInt256) {
        .limbs = {
          localBase.limbs[0] & toMultiplyMask,
          localBase.limbs[1] & toMultiplyMask,
          localBase.limbs[2] & toMultiplyMask,
          (localBase.limbs[3] & toMultiplyMask) | ((~toMultiplyMask) & 1)
        }
      };

      modularMultiplicationUsingRussianPeasant(result, &toMultiply, result);

      modularMultiplicationUsingRussianPeasant(&localBase, &localBase, &tmp); // base = base^2
      // localBase is in tmp now, instead of *localBase = tmp, I will use tmp as localBase
      // inside this iteration and loop half as long

      limb >>= 1;
      toMultiplyMask = -(limb & 1);

      toMultiply = (UInt256) {
        .limbs = {
          tmp.limbs[0] & toMultiplyMask,
          tmp.limbs[1] & toMultiplyMask,
          tmp.limbs[2] & toMultiplyMask,
          (tmp.limbs[3] & toMultiplyMask) | ((~toMultiplyMask) & 1)
        }
      };

      modularMultiplicationUsingRussianPeasant(result, &toMultiply, result);

      modularMultiplicationUsingRussianPeasant(&tmp, &tmp, &localBase); // base = base^2
      // localBase is again in localBase
      limb >>= 1;
    }
  }
}
