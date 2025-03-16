#pragma inline
const uchar is_outside_secp256k1_space(const UInt256 a) {
  const UInt256 p = getP();
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
const UInt256 modulus(const UInt256 a) {
  const UInt256 p = getP();

  if (is_outside_secp256k1_space(a)) {
    return uint256Subtraction(a, p);
  }

  return a;
}
