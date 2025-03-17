#pragma inline
const UInt256 uint256ShiftLeft(const UInt256 x) {

  UInt256 result;

  result.limbs[3] = x.limbs[3] << 1;
  result.limbs[2] = (x.limbs[2] << 1) | (x.limbs[3] >> 63);
  result.limbs[1] = (x.limbs[1] << 1) | (x.limbs[2] >> 63);
  result.limbs[0] = (x.limbs[0] << 1) | (x.limbs[1] >> 63);

  return result;
}