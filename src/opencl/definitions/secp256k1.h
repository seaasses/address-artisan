#include "src/opencl/structs/structs.h"

#define SECP256K1_P                                                     \
  (UInt256){0xFFFFFFFFFFFFFFFF, 0xFFFFFFFFFFFFFFFF, 0xFFFFFFFFFFFFFFFF, \
            0xFFFFFFFEFFFFFC2F}
#define SECP256K1_P_MINUS_2                                             \
  (UInt256){0xFFFFFFFFFFFFFFFF, 0xFFFFFFFFFFFFFFFF, 0xFFFFFFFFFFFFFFFF, \
            0xFFFFFFFEFFFFFC2D}
#define SECP256K1_P_0 0xFFFFFFFFFFFFFFFF
#define SECP256K1_P_1 0xFFFFFFFFFFFFFFFF
#define SECP256K1_P_2 0xFFFFFFFFFFFFFFFF
#define SECP256K1_P_3 0xFFFFFFFEFFFFFC2F

#define SECP256K1_G (Point){                                         \
    .x = {0x79BE667EF9DCBBAC, 0x55A06295CE870B07, 0x029BFCDB2DCE28D9, \
          0x59F2815B16F81798},                                       \
    .y = {0x483ADA7726A3C465, 0x5DA4FBFC0E1108A8, 0xFD17B448A6855419, \
          0x9C47D08FFB10D4B8},                                       \
};
