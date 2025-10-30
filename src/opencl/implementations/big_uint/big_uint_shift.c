#include "src/opencl/headers/big_uint/big_uint_shift.h"

inline Uint256 uint256_shift_left(const Uint256 x)
{
  Uint256 result;
  result.limbs[0] = (x.limbs[0] << 1) | (x.limbs[1] >> 63);
  result.limbs[1] = (x.limbs[1] << 1) | (x.limbs[2] >> 63);
  result.limbs[2] = (x.limbs[2] << 1) | (x.limbs[3] >> 63);
  result.limbs[3] = x.limbs[3] << 1;
  return result;
}

inline Uint256 uint256_shift_right(const Uint256 x)
{
  Uint256 result;
  result.limbs[3] = (x.limbs[3] >> 1) | (x.limbs[2] << 63);
  result.limbs[2] = (x.limbs[2] >> 1) | (x.limbs[1] << 63);
  result.limbs[1] = (x.limbs[1] >> 1) | (x.limbs[0] << 63);
  result.limbs[0] = x.limbs[0] >> 1;
  return result;
}