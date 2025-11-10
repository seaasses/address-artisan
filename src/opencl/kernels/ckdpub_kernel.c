#include "src/opencl/headers/secp256k1/ckdpub.h"
#include "src/opencl/headers/big_uint/big_uint_from_bytes.h"
#include "src/opencl/headers/big_uint/big_uint_to_bytes.h"

__kernel void ckdpub_kernel(
    __global uchar *chain_code_buffer,
    __global uchar *k_par_x_buffer,
    __global uchar *k_par_y_buffer,
    __global uint *index_buffer,
    __global uchar *compressed_key_buffer)
{
    uchar chain_code_private[32];
    uchar k_par_x_private[32];
    uchar k_par_y_private[32];

    for (int i = 0; i < 32; i++) {
        chain_code_private[i] = chain_code_buffer[i];
        k_par_x_private[i] = k_par_x_buffer[i];
        k_par_y_private[i] = k_par_y_buffer[i];
    }

    uint index = index_buffer[0];

    XPub parent;
    for (int i = 0; i < 32; i++) {
        parent.chain_code[i] = chain_code_private[i];
    }
    parent.k_par.x = UINT256_FROM_BYTES(k_par_x_private);
    parent.k_par.y = UINT256_FROM_BYTES(k_par_y_private);

    uchar compressed_key_private[33];
    ckdpub(parent, index, compressed_key_private);

    for (int i = 0; i < 33; i++) {
        compressed_key_buffer[i] = compressed_key_private[i];
    }
}
