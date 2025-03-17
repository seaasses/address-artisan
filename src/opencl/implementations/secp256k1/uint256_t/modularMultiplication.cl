#pragma inline
const ulong isDifferentFromZero(const UInt256 a) {
  return a.limbs[0] | a.limbs[1] | a.limbs[2] | a.limbs[3];
}

// TODO: implement multiplication and then modulus with 512 bits to test
#pragma inline
const UInt256 modularMultiplicationUsingRussianPeasant(UInt256 a, UInt256 b) {
  UInt256 result = {0};

  a = modulus(a);

  // TODO: maybe do 256 is faster than see if b is zero?
  // TODO: maybe see what is smaller and use it to loop?
  while (isDifferentFromZero(b)) {

    if (b.limbs[3] & 0x0000000000000001) { // if is odd
      result = modularAddition(result, a);
      return result;
    }
    a = uint256ShiftLeft(a);
    b = uint256ShiftRight(b);
  }

  return result;
}
