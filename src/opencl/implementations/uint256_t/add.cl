
#pragma inline
uint256_t uint256_t_add(uint256_t a, uint256_t b) {
  uint256_t result;
  ulong carry = 0;

  result.limbs[3] = a.limbs[3] + b.limbs[3] + carry;
  carry = (result.limbs[3] < a.limbs[3] || result.limbs[3] < b.limbs[3]);

  result.limbs[2] = a.limbs[2] + b.limbs[2] + carry;
  carry = (result.limbs[2] < a.limbs[2] || result.limbs[2] < b.limbs[2]);

  result.limbs[1] = a.limbs[1] + b.limbs[1] + carry;
  carry = (result.limbs[1] < a.limbs[1] || result.limbs[1] < b.limbs[1]);

  result.limbs[0] = a.limbs[0] + b.limbs[0] + carry;
  return result;
}