// TODO: implement multiplication and then modulus with 512 bits to test if this
// is faster - I really don't know
#pragma inline
const UInt256 modularMultiplicationUsingRussianPeasant(UInt256 a, UInt256 b) {
  // I know that a and b are already < P, no need to modules before starting
  UInt256 result = {0};
  UInt256 toSum;
  ulong mask;

  // TODO: maybe do 256 is faster than see if b is zero? ors are fast, but this
  // can cause warp stalls
  // TODO: maybe see what is smaller and use it to loop? - do not need to do
  // this if the above is true
  while (b.limbs[0] | b.limbs[1] | b.limbs[2] | b.limbs[3]) {
    // for (uint i = 0; i != 256; i++) { // TODO: test this

    if (b.limbs[3] & 1) { // if is odd
      result = modularAddition(result, a);
    }

    a = modularShiftLeft(a);
    b = uint256ShiftRight(b);
  }

  return result;
}
