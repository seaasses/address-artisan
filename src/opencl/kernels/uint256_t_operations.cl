__kernel void uint256Operations(__global uchar *input_a,
                                __global uchar *input_b, uchar operation,
                                __global uchar *result) {

  // THIS DOES NOT NEED TO BE FAST, IT IS ONLY USED FOR TESTING

  uchar local_a[32];
  uchar local_b[32];
  uchar local_result[32];

  for (uchar i = 0; i < 32; i++) {
    local_a[i] = input_a[i];
    local_b[i] = input_b[i];
  }

  const UInt256 a = uint256FromBytes(local_a);
  const UInt256 b = uint256FromBytes(local_b);

  UInt256 local_class_result;

  if (operation == 0) {
    local_class_result = uint256Addition(a, b);
  } else if (operation == 1) {
    local_class_result = uint256Subtraction(a, b);
  } else if (operation == 2) {
    local_class_result = uint256ShiftLeft(a);
  } else if (operation == 3) {
    local_class_result = uint256ShiftRight(a);
  }

  uint256ToBytes(local_class_result, local_result);

  for (uchar i = 0; i < 32; i++) {
    result[i] = local_result[i];
  }
}