#pragma inline
const UInt256 modularAddition(const UInt256 a, const UInt256 b) {
  bool overflowFlag;
  const UInt256 result;

  uint256AdditionWithOverflowFlag(&a, &b, &result, &overflowFlag);

  if (overflowFlag) {
    return uint256Subtraction(result, SECP256K1_P);
  }

  return modulus(result);
}