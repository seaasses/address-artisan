#pragma inline
const UInt256 modularAddition(const UInt256 a, const UInt256 b) {

  const UInt256 result = uint256Addition(a, b);

  return modulus(result);
}