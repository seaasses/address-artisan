
typedef struct {
  ulong limbs[4];
} uint256_t;

uint256_t uint256_t_from_bytes(uchar *input) {
  return (uint256_t){(((ulong)(input[0]) << 56) | ((ulong)(input[1]) << 48) |
                      ((ulong)(input[2]) << 40) | ((ulong)(input[3]) << 32) |
                      ((ulong)(input[4]) << 24) | ((ulong)(input[5]) << 16) |
                      ((ulong)(input[6]) << 8) | ((ulong)(input[7]))),
                     (((ulong)(input[8]) << 56) | ((ulong)(input[9]) << 48) |
                      ((ulong)(input[10]) << 40) | ((ulong)(input[11]) << 32) |
                      ((ulong)(input[12]) << 24) | ((ulong)(input[13]) << 16) |
                      ((ulong)(input[14]) << 8) | ((ulong)(input[15]))),
                     (((ulong)(input[16]) << 56) | ((ulong)(input[17]) << 48) |
                      ((ulong)(input[18]) << 40) | ((ulong)(input[19]) << 32) |
                      ((ulong)(input[20]) << 24) | ((ulong)(input[21]) << 16) |
                      ((ulong)(input[22]) << 8) | ((ulong)(input[23]))),
                     (((ulong)(input[24]) << 56) | ((ulong)(input[25]) << 48) |
                      ((ulong)(input[26]) << 40) | ((ulong)(input[27]) << 32) |
                      ((ulong)(input[28]) << 24) | ((ulong)(input[29]) << 16) |
                      ((ulong)(input[30]) << 8) | ((ulong)(input[31])))};
};

void uint256_t_to_bytes(uint256_t a, uchar *result) {
  result[0] = a.limbs[0] >> 56;
  result[1] = a.limbs[0] >> 48;
  result[2] = a.limbs[0] >> 40;
  result[3] = a.limbs[0] >> 32;
  result[4] = a.limbs[0] >> 24;
  result[5] = a.limbs[0] >> 16;
  result[6] = a.limbs[0] >> 8;
  result[7] = a.limbs[0];

  result[8] = a.limbs[1] >> 56;
  result[9] = a.limbs[1] >> 48;
  result[10] = a.limbs[1] >> 40;
  result[11] = a.limbs[1] >> 32;
  result[12] = a.limbs[1] >> 24;
  result[13] = a.limbs[1] >> 16;
  result[14] = a.limbs[1] >> 8;
  result[15] = a.limbs[1];

  result[16] = a.limbs[2] >> 56;
  result[17] = a.limbs[2] >> 48;
  result[18] = a.limbs[2] >> 40;
  result[19] = a.limbs[2] >> 32;
  result[20] = a.limbs[2] >> 24;
  result[21] = a.limbs[2] >> 16;
  result[22] = a.limbs[2] >> 8;
  result[23] = a.limbs[2];

  result[24] = a.limbs[3] >> 56;
  result[25] = a.limbs[3] >> 48;
  result[26] = a.limbs[3] >> 40;
  result[27] = a.limbs[3] >> 32;
  result[28] = a.limbs[3] >> 24;
  result[29] = a.limbs[3] >> 16;
  result[30] = a.limbs[3] >> 8;
  result[31] = a.limbs[3];
}

#pragma inline
uint256_t uint256_t_sub(uint256_t a, uint256_t b) {
  uint256_t result;

  ulong temp;

  temp = a.limbs[3] - b.limbs[3];
  ulong borrow = (a.limbs[3] < b.limbs[3]) ? 1 : 0;
  result.limbs[3] = temp;

  temp = a.limbs[2] - b.limbs[2] - borrow;
  borrow =
      (a.limbs[2] < b.limbs[2] || (a.limbs[2] == b.limbs[2] && borrow)) ? 1 : 0;
  result.limbs[2] = temp;

  temp = a.limbs[1] - b.limbs[1] - borrow;
  borrow =
      (a.limbs[1] < b.limbs[1] || (a.limbs[1] == b.limbs[1] && borrow)) ? 1 : 0;
  result.limbs[1] = temp;

  result.limbs[0] = a.limbs[0] - b.limbs[0] - borrow;

  return result;
}

#pragma inline
uint256_t uint256_t_add(uint256_t a, uint256_t b) {
  uint256_t result;
  ulong carry = 0;

  result.limbs[3] = a.limbs[3] + b.limbs[3] + carry;
  carry = (result.limbs[3] < a.limbs[3] || result.limbs[3] < b.limbs[3]);

  result.limbs[2] = a.limbs[2] + b.limbs[2] + carry;
  carry = (result.limbs[2] < a.limbs[2] || result.limbs[2] < b.limbs[2]);

  result.limbs[1] = a.limbs[1] + b.limbs[1] + carry;
  carry = (result.limbs[1] < a.limbs[1] || result.limbs[1] < b.limbs[1]);

  result.limbs[0] = a.limbs[0] + b.limbs[0] + carry;
  return result;
}

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
  }

  uint256_t_to_bytes(local_class_result, local_result);

  for (uchar i = 0; i < 32; i++) {
    result[i] = local_result[i];
  }
}