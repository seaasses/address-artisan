#pragma inline
uint256_t uint256_t_shift_left(uint256_t x) {

  uint256_t result;
  ulong carry = 0;

  result.limbs[3] = x.limbs[3] << 1;

#pragma unroll
  for (char i = 2; i >= 0; --i) {
    result.limbs[i] = (x.limbs[i] << 1) | (x.limbs[i + 1] >> 63);
  }

  return result;
}