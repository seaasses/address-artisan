#include "src/opencl/headers/modularOperations/modularMultiplication.h"
#include "src/opencl/headers/modularOperations/modularAddition.h"
#include "src/opencl/headers/modularOperations/modularDouble.h"
#include "src/opencl/headers/uint256/shiftRight.h"

// TODO: implement multiplication and then modulus with 512 bits to test if this
// is faster - I really don't know
#pragma inline
void modularMultiplicationUsingRussianPeasant(const UInt256 *a, const UInt256 *b, UInt256 *result)
{ // inplace semi-safe. safe if result = a
  // I know that a and b are already < P, no need to modules before starting
  UInt256 localA = *a;
  UInt256 toAdd;
  unsigned long limb;
  unsigned long toAddMask;
  *result = (UInt256) {0};
  // TODO: maybe do 256 is faster than see if b is zero? ors are fast, but this
  // can cause warp stalls
  // TODO: maybe see what is smaller and use it to loop? - do not need to do
  // this if the above is true

#pragma unroll
  for (unsigned char limbIndex = 3; limbIndex != 0xFF; --limbIndex) 
  {
    limb = b->limbs[limbIndex];
#pragma unroll
    for (unsigned int i = 0; i != 64; i++) 
    {

      toAddMask = -(limb & 1);

      UInt256 toAdd = (UInt256) {
        .limbs = {
          localA.limbs[0] & toAddMask,
          localA.limbs[1] & toAddMask,
          localA.limbs[2] & toAddMask,
          localA.limbs[3] & toAddMask,
        }
      };

      modularAddition(result, &toAdd, result);
      modularDouble(&localA, &localA);

      limb >>= 1;
    }
  }
}
