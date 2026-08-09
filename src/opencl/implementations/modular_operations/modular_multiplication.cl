#include "src/opencl/headers/modular_operations/modular_multiplication.cl.h"
#include "src/opencl/headers/modular_operations/modular_reduction.cl.h"
#include "src/opencl/headers/big_uint/big_uint_multiplication.cl.h"
#include "src/opencl/structs/structs.cl.h"

inline Uint256 modular_multiplication(const Uint256 a, const Uint256 b)
{
    return modular_reduction(uint256_multiplication(a, b));
}
