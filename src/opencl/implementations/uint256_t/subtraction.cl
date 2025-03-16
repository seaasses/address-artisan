#pragma inline
const UInt256 uint256Subtraction(const UInt256 a, const UInt256 b) {
  UInt256 result;

  ulong temp;

  temp = a.limbs[3] - b.limbs[3];
  ulong borrow = (a.limbs[3] < b.limbs[3]);
  result.limbs[3] = temp;

  temp = a.limbs[2] - b.limbs[2] - borrow;
  borrow = (a.limbs[2] < b.limbs[2] || (a.limbs[2] == b.limbs[2] && borrow));
  result.limbs[2] = temp;

  temp = a.limbs[1] - b.limbs[1] - borrow;
  borrow = (a.limbs[1] < b.limbs[1] || (a.limbs[1] == b.limbs[1] && borrow));
  result.limbs[1] = temp;

  result.limbs[0] = a.limbs[0] - b.limbs[0] - borrow;

  return result;
}
