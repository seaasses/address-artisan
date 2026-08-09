#include "src/opencl/headers/modular_operations/modular_square.cl.h"
#include "src/opencl/headers/modular_operations/modular_reduction.cl.h"
#include "src/opencl/headers/big_uint/big_uint_square.cl.h"
#include "src/opencl/structs/structs.cl.h"

inline Uint256 modular_square(const Uint256 a)
{
    return modular_reduction(uint256_square(a));
}
