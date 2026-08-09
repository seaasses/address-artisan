#include "src/opencl/headers/big_uint/big_uint_shift.cl.h"

inline Uint256 uint256_shift_left(const Uint256 x)
{
  Uint256 result;
#pragma unroll
  for (int i = 0; i < 7; i++)
  {
    result.limbs[i] = (x.limbs[i] << 1) | (x.limbs[i + 1] >> 31);
  }
  result.limbs[7] = x.limbs[7] << 1;
  return result;
}

inline Uint256 uint256_shift_right(const Uint256 x)
{
  Uint256 result;
#pragma unroll
  for (int i = 7; i > 0; i--)
  {
    result.limbs[i] = (x.limbs[i] >> 1) | (x.limbs[i - 1] << 31);
  }
  result.limbs[0] = x.limbs[0] >> 1;
  return result;
}
