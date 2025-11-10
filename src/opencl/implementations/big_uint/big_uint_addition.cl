#include "src/opencl/headers/big_uint/big_uint_addition.cl.h"

inline Uint256WithOverflow uint256_addition_with_overflow_flag(const Uint256 a, const Uint256 b)
{
  Uint256WithOverflow ret;

  ret.result.limbs[3] = a.limbs[3] + b.limbs[3];
  uint carry = ret.result.limbs[3] < a.limbs[3];

  ret.result.limbs[2] = a.limbs[2] + b.limbs[2] + carry;
  carry = (ret.result.limbs[2] < a.limbs[2]) | ((ret.result.limbs[2] == a.limbs[2]) & carry);

  ret.result.limbs[1] = a.limbs[1] + b.limbs[1] + carry;
  carry = (ret.result.limbs[1] < a.limbs[1]) | ((ret.result.limbs[1] == a.limbs[1]) & carry);

  ret.result.limbs[0] = a.limbs[0] + b.limbs[0] + carry;
  ret.overflow = (ret.result.limbs[0] < a.limbs[0]) | ((ret.result.limbs[0] == a.limbs[0]) & carry);

  return ret;
}

inline Uint256 uint256_addition(const Uint256 a, const Uint256 b)
{
  Uint256 result;

  result.limbs[3] = a.limbs[3] + b.limbs[3];
  uint carry = result.limbs[3] < a.limbs[3];

  result.limbs[2] = a.limbs[2] + b.limbs[2] + carry;
  carry = (result.limbs[2] < a.limbs[2]) | ((result.limbs[2] == a.limbs[2]) & carry);

  result.limbs[1] = a.limbs[1] + b.limbs[1] + carry;
  carry = (result.limbs[1] < a.limbs[1]) | ((result.limbs[1] == a.limbs[1]) & carry);

  result.limbs[0] = a.limbs[0] + b.limbs[0] + carry;

  return result;
}

inline Uint320 uint320_uint256_addition(const Uint320 a, const Uint256 b)
{
    Uint320 result;

    result.limbs[4] = a.limbs[4] + b.limbs[3];
    uint carry = result.limbs[4] < a.limbs[4];

    result.limbs[3] = a.limbs[3] + b.limbs[2] + carry;
    carry = (result.limbs[3] < a.limbs[3]) | ((result.limbs[3] == a.limbs[3]) & carry);

    result.limbs[2] = a.limbs[2] + b.limbs[1] + carry;
    carry = (result.limbs[2] < a.limbs[2]) | ((result.limbs[2] == a.limbs[2]) & carry);

    result.limbs[1] = a.limbs[1] + b.limbs[0] + carry;
    carry = (result.limbs[1] < a.limbs[1]) | ((result.limbs[1] == a.limbs[1]) & carry);

    result.limbs[0] = a.limbs[0] + carry;

    return result;
}