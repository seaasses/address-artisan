#include "src/opencl/structs/structs.cl.h"

#define SECP256K1_P                                                        \
    {.limbs = {0xFFFFFFFFFFFFFFFF, 0xFFFFFFFFFFFFFFFF, 0xFFFFFFFFFFFFFFFF, \
               0xFFFFFFFEFFFFFC2F}}

#define SECP256K1_P_MINUS_1                                                \
    {.limbs = {0xFFFFFFFFFFFFFFFF, 0xFFFFFFFFFFFFFFFF, 0xFFFFFFFFFFFFFFFF, \
               0xFFFFFFFEFFFFFC2E}}

#define SECP256K1_P_MINUS_2                                                \
    {.limbs = {0xFFFFFFFFFFFFFFFF, 0xFFFFFFFFFFFFFFFF, 0xFFFFFFFFFFFFFFFF, \
               0xFFFFFFFEFFFFFC2D}}

#define P_FOLDING_CONSTANT 0x00000001000003d1ULL

#define SECP256K1_P_0 0xFFFFFFFFFFFFFFFF
#define SECP256K1_P_1 0xFFFFFFFFFFFFFFFF
#define SECP256K1_P_2 0xFFFFFFFFFFFFFFFF
#define SECP256K1_P_3 0xFFFFFFFEFFFFFC2F

#define SECP256K1_G {                                                           \
    .x = {.limbs = {0x79BE667EF9DCBBAC, 0x55A06295CE870B07, 0x029BFCDB2DCE28D9, \
                    0x59F2815B16F81798}},                                       \
    .y = {.limbs = {0x483ADA7726A3C465, 0x5DA4FBFC0E1108A8, 0xFD17B448A6855419, \
                    0x9C47D08FFB10D4B8}},                                       \
};

#define SECP256K1_G_X_0 0x79BE667EF9DCBBAC
#define SECP256K1_G_X_1 0x55A06295CE870B07
#define SECP256K1_G_X_2 0x029BFCDB2DCE28D9
#define SECP256K1_G_X_3 0x59F2815B16F81798
#define SECP256K1_G_Y_0 0x483ADA7726A3C465
#define SECP256K1_G_Y_1 0x5DA4FBFC0E1108A8
#define SECP256K1_G_Y_2 0xFD17B448A6855419
#define SECP256K1_G_Y_3 0x9C47D08FFB10D4B8


