__kernel void uint256Operations(__global uchar *input_a,
                                __global uchar *input_b, uchar operation,
                                __global uchar *result,
                                __global uchar *booleanFlag) {

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
  bool localBooleanFlag;

  if (operation == 0) {
    uint256AdditionWithOverflowFlag(&a, &b, &local_class_result,
                                    &localBooleanFlag);
  } else if (operation == 1) {
    local_class_result = uint256Subtraction(a, b);
  } else if (operation == 2) {
    local_class_result = uint256ShiftLeft(a);
  } else if (operation == 3) {
    local_class_result = uint256ShiftRight(a);
  } else if (operation == 4) {
    uint256SubtractionWithUnderflowFlag(&a, &b, &local_class_result,
                                        &localBooleanFlag);
  }

  uint256ToBytes(local_class_result, local_result);

  for (uchar i = 0; i < 32; i++) {
    result[i] = local_result[i];
  }
  *booleanFlag = (localBooleanFlag == true) ? 1 : 0;
}