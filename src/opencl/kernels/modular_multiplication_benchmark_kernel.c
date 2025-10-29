#include "src/opencl/headers/modular_operations/modular_multiplication.h"
#include "src/opencl/headers/big_uint/big_uint_from_bytes.h"

__kernel void modular_multiplication_benchmark_kernel(
    __constant unsigned char *a_buffer,
    __constant unsigned char *b_buffer,
    unsigned int max_threads,
    unsigned int iterations,
    __global volatile unsigned int *anti_optimization_counter)
{
    unsigned int thread_id = get_global_id(0);

    // Early exit for threads beyond max_threads
    if (thread_id >= max_threads) {
        return;
    }

    // Convert from constant memory to Uint256
    Uint256 a = UINT256_FROM_BYTES(a_buffer);
    Uint256 b = UINT256_FROM_BYTES(b_buffer);

    // Perform iterations of modular multiplication
    Uint256 result = a;
    for (unsigned int i = 0; i < iterations; i++) {
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
