#include "src/opencl/structs/structs.cl.h"

// Reduction constant: 2^256 mod p = 2^32 + 977, so the "small" fold factor is 977.
#define SECP256K1_REDUCE_C_977 977

// secp256k1 field prime p, big-limb-first (limbs[0] most significant), 8x32 bits.
#define SECP256K1_P_0 0xFFFFFFFF
#define SECP256K1_P_1 0xFFFFFFFF
#define SECP256K1_P_2 0xFFFFFFFF
#define SECP256K1_P_3 0xFFFFFFFF
#define SECP256K1_P_4 0xFFFFFFFF
#define SECP256K1_P_5 0xFFFFFFFF
#define SECP256K1_P_6 0xFFFFFFFE
#define SECP256K1_P_7 0xFFFFFC2F
