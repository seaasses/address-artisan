#include "src/opencl/headers/modularOperations/modularExponentiation.h"
#include "src/opencl/headers/modularOperations/modularMultiplication.h"
#include "src/opencl/headers/uint256/shiftRight.h"

#pragma inline
const UInt256 modularExponentiation(UInt256 base, UInt256 exponent)
{

  // base will be < modulus, so no need to modulus before starting
  UInt256 result = {.limbs = {
                        0x0000000000000000,
                        0x0000000000000000,
                        0x0000000000000000,
                        0x0000000000000001,
                    }};

  while (exponent.limbs[0] | exponent.limbs[1] | exponent.limbs[2] |
         exponent.limbs[3])
  {
    if (exponent.limbs[3] & 1)
    {
      result = modularMultiplicationUsingRussianPeasant(result, base);
    }

    base =
        modularMultiplicationUsingRussianPeasant(base, base); // base = base^2
    exponent = uint256ShiftRight(exponent);                   // exponent = exponent // 2
  }

  return result;
}
