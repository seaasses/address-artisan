__kernel void modularOperations(__global uchar *x1, __global uchar *y1,
                                 __global uchar *x2, __global uchar *y2,
                                 uchar operation, __global uchar *result_x,
                                 __global uchar *result_y) {

  // THIS DOES NOT NEED TO BE FAST, IT IS ONLY USED FOR TESTING

  uchar local_x1[32];
  uchar local_y1[32];
  uchar local_x2[32];
  uchar local_y2[32];
  uchar local_result_x[32];
  uchar local_result_y[32];

#pragma unroll
  for (uchar i = 0; i < 32; i++) {
    local_x1[i] = x1[i];
    local_y1[i] = y1[i];
    local_x2[i] = x2[i];
    local_y2[i] = y2[i];
  }
  //////////////////////////////////////
  if (operation == 0) {
    // simple integer modular addition between x1 and y1
    const UInt256 x1_as_uint256 = uint256FromBytes(local_x1);
    const UInt256 y1_as_uint256 = uint256FromBytes(local_y1);

    const UInt256 result = modularAddition(x1_as_uint256, y1_as_uint256);

    uint256ToBytes(result, local_result_x);
  } else if (operation == 1) {
    //  simple integer modular multiplication between x1 and y1 using the
    //  russian peasant algorithm

    const UInt256 x1_as_uint256 = uint256FromBytes(local_x1);
    const UInt256 y1_as_uint256 = uint256FromBytes(local_y1);

    const UInt256 result =
        modularMultiplicationUsingRussianPeasant(x1_as_uint256, y1_as_uint256);

    uint256ToBytes(result, local_result_x);
  } else if (operation == 2) {
    // modular exponentiation between x1 (base) and y1 (exponent)
    const UInt256 x1_as_uint256 = uint256FromBytes(local_x1);
    const UInt256 y1_as_uint256 = uint256FromBytes(local_y1);

    const UInt256 result = modularExponentiation(x1_as_uint256, y1_as_uint256);

    uint256ToBytes(result, local_result_x);
  }

  // send result to the host
#pragma unroll
  for (uchar i = 0; i < 32; i++) {
    result_x[i] = local_result_x[i];
    result_y[i] = local_result_y[i];
  }
}