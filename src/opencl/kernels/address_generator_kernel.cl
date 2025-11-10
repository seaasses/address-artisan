#include "src/opencl/headers/secp256k1/ckdpub.cl.h"
#include "src/opencl/headers/hash/hash160.cl.h"

__kernel void address_generator_kernel(
    __global uchar *chain_code_buffer,
    __global uchar *k_par_x_buffer,
    __global uchar *k_par_y_buffer,
    uint base_index,
    uint quant,
    __global uchar *hash160_output_buffer
)
{
    uint thread_id = get_global_id(0);

    if (thread_id >= quant) {
        return;
    }

    uint index = base_index + thread_id;

    XPub parent;
    for (int i = 0; i < 32; i++) {
        parent.chain_code[i] = chain_code_buffer[i];
    }
    parent.k_par.x = UINT256_FROM_BYTES(k_par_x_buffer);
    parent.k_par.y = UINT256_FROM_BYTES(k_par_y_buffer);

    uchar compressed_key[33];
    ckdpub(parent, index, compressed_key);

    uchar hash160[20];
    hash160_33(compressed_key, hash160);

    for (int i = 0; i < 20; i++) {
        hash160_output_buffer[thread_id * 20 + i] = hash160[i];
    }
}
