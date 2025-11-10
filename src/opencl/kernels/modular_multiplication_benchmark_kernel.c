#include "src/opencl/headers/modular_operations/modular_multiplication.h"
#include "src/opencl/headers/big_uint/big_uint_from_bytes.h"

__kernel void modular_multiplication_benchmark_kernel(
    __constant uchar *a_buffer,
    __constant uchar *b_buffer,
    uint max_threads,
    uint iterations,
    __global volatile uint *anti_optimization_counter)
{
    uint thread_id = get_global_id(0);

    // Early exit for threads beyond max_threads
    if (thread_id >= max_threads) {
        return;
    }

    // Convert from constant memory to Uint256
    Uint256 a = UINT256_FROM_BYTES(a_buffer);
    Uint256 b = UINT256_FROM_BYTES(b_buffer);

    // Perform iterations of modular multiplication
    Uint256 result = a;
    for (uint i = 0; i < iterations; i++) {
        result = modular_multiplication(result, b);
    }

    // XOR all limbs together to prevent dead code elimination
    ulong xor_result = result.limbs[0] ^ result.limbs[1] ^
                       result.limbs[2] ^ result.limbs[3];

    // Extremely unlikely condition to prevent optimization
    if (xor_result == 1) {
        atomic_inc(anti_optimization_counter);
    }
}
