#include "src/opencl/structs/structs.cl.h"


Uint512 uint256_multiplication(const Uint256 a, const Uint256 b);

Uint320 uint256_ulong_multiplication(const Uint256 a, const ulong b);

void add_component_to_limb(ulong a_limb, ulong b_limb,
                           ulong *carry_high, ulong *carry_low, ulong *result_limb);