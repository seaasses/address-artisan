#include "src/opencl/headers/big_uint/big_uint_addition.cl.h"

inline Uint256WithOverflow uint256_addition_with_overflow_flag(const Uint256 a, const Uint256 b)
{
  Uint256WithOverflow ret;
  uint carry = 0;
  for (int i = 7; i >= 0; i--)
  {
    uint sum = a.limbs[i] + b.limbs[i];
    uint carry1 = sum < a.limbs[i];
    uint out = sum + carry;
    uint carry2 = out < sum;
    ret.result.limbs[i] = out;
    carry = carry1 | carry2;
  }
  ret.overflow = carry;
  return ret;
}

inline Uint256 uint256_addition(const Uint256 a, const Uint256 b)
{
  Uint256 result;
  uint carry = 0;
  for (int i = 7; i >= 0; i--)
  {
    uint sum = a.limbs[i] + b.limbs[i];
    uint carry1 = sum < a.limbs[i];
    uint out = sum + carry;
    uint carry2 = out < sum;
    result.limbs[i] = out;
    carry = carry1 | carry2;
  }
  return result;
}
