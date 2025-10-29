#include "src/opencl/headers/secp256k1/ckdpub.h"
#include "src/opencl/headers/big_uint/big_uint_from_bytes.h"
#include "src/opencl/headers/big_uint/big_uint_to_bytes.h"

__kernel void ckdpub_kernel(
    __global unsigned char *chain_code_buffer,
    __global unsigned char *k_par_x_buffer,
    __global unsigned char *k_par_y_buffer,
    __global unsigned int *index_buffer,
    __global unsigned char *compressed_key_buffer)
{
    // Copy data from global to private memory
    unsigned char chain_code_private[32];
    unsigned char k_par_x_private[32];
    unsigned char k_par_y_private[32];

    for (int i = 0; i < 32; i++) {
        chain_code_private[i] = chain_code_buffer[i];
        k_par_x_private[i] = k_par_x_buffer[i];
        k_par_y_private[i] = k_par_y_buffer[i];
    }

    unsigned int index = index_buffer[0];

    // Convert byte arrays to Point
    Point k_par;
    k_par.x = UINT256_FROM_BYTES(k_par_x_private);
    k_par.y = UINT256_FROM_BYTES(k_par_y_private);

    // Derive child key - result is written to private buffer
    unsigned char compressed_key_private[33];
    ckdpub(chain_code_private, k_par, index, compressed_key_private);

    // Copy result to global memory
    for (int i = 0; i < 33; i++) {
        compressed_key_buffer[i] = compressed_key_private[i];
    }
}
