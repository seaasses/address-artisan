__kernel void modularOperations(__global uchar *a, __global uchar *b,
                                uchar operation, __global uchar *result) {

  // THIS DOES NOT NEED TO BE FAST, IT IS ONLY USED FOR TESTING

  uchar local_a[32];
  uchar local_b[32];
  uchar local_result[32];
  UInt256 localResultUint256;

#pragma unroll
  for (uchar i = 0; i < 32; i++) {
    local_a[i] = a[i];
    local_b[i] = b[i];
  }
  const UInt256 a_as_uint256 = uint256FromBytes(local_a);
  const UInt256 b_as_uint256 = uint256FromBytes(local_b);

  //////////////////////////////////////
  if (operation == 0) {
    // simple integer modular addition between x1 and y1
    localResultUint256 = modularAddition(a_as_uint256, b_as_uint256);
  } else if (operation == 1) {
    //  simple integer modular multiplication between x1 and y1 using the
    //  russian peasant algorithm
    localResultUint256 =
        modularMultiplicationUsingRussianPeasant(a_as_uint256, b_as_uint256);
  } else if (operation == 2) {
    // modular exponentiation between x1 (base) and y1 (exponent)
    localResultUint256 = modularExponentiation(a_as_uint256, b_as_uint256);
  } else if (operation == 3) {
    // modular subtraction between x1 and y1
    localResultUint256 = modularSubtraction(a_as_uint256, b_as_uint256);
  }

  uint256ToBytes(localResultUint256, local_result);

  // send result to the host
#pragma unroll
  for (uchar i = 0; i < 32; i++) {
    result[i] = local_result[i];
  }
}