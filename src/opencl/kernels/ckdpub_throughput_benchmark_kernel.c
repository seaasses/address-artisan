#include "src/opencl/headers/secp256k1/ckdpub.h"
#include "src/opencl/headers/big_uint/big_uint_from_bytes.h"

__kernel void ckdpub_throughput_benchmark_kernel(
    __constant unsigned char *chain_code_buffer,
    __constant unsigned char *k_par_x_buffer,
    __constant unsigned char *k_par_y_buffer,
    unsigned int max_threads,
    __global volatile unsigned int *anti_optimization_counter)
{
    unsigned int thread_id = get_global_id(0);

    // Early exit for threads beyond max_threads
    if (thread_id >= max_threads) {
        return;
    }

    // Convert from constant memory to Point struct
    Point k_par;
    k_par.x = UINT256_FROM_BYTES(k_par_x_buffer);
    k_par.y = UINT256_FROM_BYTES(k_par_y_buffer);

    // Copy chain_code from constant to private for ckdpub call
    unsigned char chain_code_private[32];
    for (int i = 0; i < 32; i++) {
        chain_code_private[i] = chain_code_buffer[i];
    }

    // Use thread_id as the index for derivation
    unsigned char compressed_key[33];
    ckdpub(chain_code_private, k_par, thread_id, compressed_key);

    // XOR all bytes together to create a checksum
    // This prevents dead code optimization while rarely being true
    unsigned char xor_result = 0;
    for (int i = 0; i < 33; i++) {
        xor_result ^= compressed_key[i];
    }

    // Extremely unlikely condition to prevent dead code elimination
    // If XOR of all 33 bytes equals exactly 1, increment counter
    // This ensures the computation can't be optimized away
    if (xor_result == 1) {
        atomic_inc(anti_optimization_counter);
    }
}
