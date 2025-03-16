#pragma inline
const uchar is_outside_secp256k1_space(const uint256_t a) {
  const uint256_t p = getP();
#pragma unroll
  for (uchar i = 0; i < 4; ++i) {
    if (a.limbs[i] > p.limbs[i]) {
      return 1; // a is greater than p
    } else if (a.limbs[i] < p.limbs[i]) {
      return 0; // a is less than p
    }
  }

  return 1; // a is equal to p
}

#pragma inline
const uint256_t modulus(const uint256_t a) {
  const uint256_t p = getP();

  if (is_outside_secp256k1_space(a)) {
    return uint256_t_sub(a, p);
  }

  return a;
}
