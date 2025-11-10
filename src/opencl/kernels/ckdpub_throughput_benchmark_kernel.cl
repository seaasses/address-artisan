#include "src/opencl/headers/secp256k1/ckdpub.cl.h"
#include "src/opencl/headers/big_uint/big_uint_from_bytes.cl.h"

__kernel void ckdpub_throughput_benchmark_kernel(
    __constant uchar *chain_code_buffer,
    __constant uchar *k_par_x_buffer,
    __constant uchar *k_par_y_buffer,
    uint max_threads,
    __global volatile uint *anti_optimization_counter)
{
    uint thread_id = get_global_id(0);

    if (thread_id >= max_threads) {
        return;
    }

    XPub parent;
    for (int i = 0; i < 32; i++) {
        parent.chain_code[i] = chain_code_buffer[i];
    }
    parent.k_par.x = UINT256_FROM_BYTES(k_par_x_buffer);
    parent.k_par.y = UINT256_FROM_BYTES(k_par_y_buffer);

    uchar compressed_key[33];
    ckdpub(parent, thread_id, compressed_key);

    uchar xor_result = 0;
    for (int i = 0; i < 33; i++) {
        xor_result ^= compressed_key[i];
    }

    if (xor_result == 1) {
        atomic_inc(anti_optimization_counter);
    }
}
