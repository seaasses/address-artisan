#pragma inline
uint256_t uint256_t_shift_right(uint256_t x) {

  uint256_t result;
  ulong carry = 0;

  result.limbs[0] = x.limbs[0] >> 1;
  result.limbs[1] = (x.limbs[1] >> 1) | (x.limbs[0] << 63);
  result.limbs[2] = (x.limbs[2] >> 1) | (x.limbs[1] << 63);
  result.limbs[3] = (x.limbs[3] >> 1) | (x.limbs[2] << 63);

  return result;
}