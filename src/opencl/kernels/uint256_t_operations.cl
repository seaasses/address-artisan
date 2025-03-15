
__kernel void uint256_t_operations(__global uchar *input_a,
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

  uint256_t a = uint256_t_from_bytes(local_a);
  uint256_t b = uint256_t_from_bytes(local_b);

  uint256_t local_class_result;

  if (operation == 0) {
    local_class_result = uint256_t_add(a, b);
  } else if (operation == 1) {
    local_class_result = uint256_t_sub(a, b);
  } else if (operation == 2) {
    local_class_result = uint256_t_shift_left(a);
  }

  uint256_t_to_bytes(local_class_result, local_result);

  for (uchar i = 0; i < 32; i++) {
    result[i] = local_result[i];
  }
}