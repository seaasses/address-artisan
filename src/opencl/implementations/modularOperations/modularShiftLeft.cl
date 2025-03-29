// TODO: test if this is faster than modularAddition(a,a) - I think it is
#pragma inline
const UInt256 modularShiftLeft(const UInt256 a) {
  const UInt256 result = uint256ShiftLeft(a);

  if (a.limbs[0] & 0x8000000000000000ull) {
    return uint256Subtraction(result, SECP256K1_P);
  }
  return modulus(result);
}
