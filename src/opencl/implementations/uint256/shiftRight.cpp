#include "src/opencl/headers/uint256/shiftRight.h"

#pragma inline
void uint256ShiftRight(const UInt256 *x, UInt256 *result)
{

  result->limbs[3] = (x->limbs[3] >> 1) | (x->limbs[2] << 63);
  result->limbs[2] = (x->limbs[2] >> 1) | (x->limbs[1] << 63);
  result->limbs[1] = (x->limbs[1] >> 1) | (x->limbs[0] << 63);
  result->limbs[0] = x->limbs[0] >> 1;
}