#include "src/opencl/headers/secp256k1/ckdpub.h"
#include "src/opencl/headers/big_uint/big_uint_from_bytes.h"
#include "src/opencl/headers/big_uint/big_uint_to_bytes.h"

__kernel void ckdpub_kernel(
    __global unsigned char *chain_code_buffer,
    __global unsigned char *k_par_x_buffer,
    __global unsigned char *k_par_y_buffer,
    __global unsigned int *index_buffer,
    __global unsigned char *k_child_x_buffer,
    __global unsigned char *k_child_y_buffer)
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
    bytes_to_uint256(k_par_x_private, &k_par.x);
    bytes_to_uint256(k_par_y_private, &k_par.y);

    // Derive child key
    Point k_child = ckdpub(chain_code_private, k_par, index);

    // Convert result back to bytes and copy to global memory
    unsigned char k_child_x_private[32];
    unsigned char k_child_y_private[32];
    uint256_to_bytes(k_child.x, k_child_x_private);
    uint256_to_bytes(k_child.y, k_child_y_private);

    for (int i = 0; i < 32; i++) {
        k_child_x_buffer[i] = k_child_x_private[i];
        k_child_y_buffer[i] = k_child_y_private[i];
    }
}
