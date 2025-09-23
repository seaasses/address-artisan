#include "src/opencl/headers/secp256k1/jacobian_to_affine.h"
#include "src/opencl/headers/big_uint/big_uint_from_bytes.h"
#include "src/opencl/headers/big_uint/big_uint_to_bytes.h"

__kernel void jacobian_to_affine_kernel(
    __global unsigned char *jac_x_buffer,
    __global unsigned char *jac_y_buffer,
    __global unsigned char *jac_z_buffer,
    __global unsigned char *aff_x_buffer,
    __global unsigned char *aff_y_buffer)
{
    JacobianPoint jac_point;
    Point aff_point;

    // Copy data from global to private memory and convert
    unsigned char jac_x_private[32];
    unsigned char jac_y_private[32];
    unsigned char jac_z_private[32];

    for (int i = 0; i < 32; i++) {
        jac_x_private[i] = jac_x_buffer[i];
        jac_y_private[i] = jac_y_buffer[i];
        jac_z_private[i] = jac_z_buffer[i];
    }

    // Convert byte arrays to Uint256
    bytes_to_uint256(jac_x_private, &jac_point.x);
    bytes_to_uint256(jac_y_private, &jac_point.y);
    bytes_to_uint256(jac_z_private, &jac_point.z);

    // Perform jacobian to affine conversion
    jacobian_to_affine(&jac_point, &aff_point);

    // Convert result back to bytes and copy to global memory
    unsigned char aff_x_private[32];
    unsigned char aff_y_private[32];
    uint256_to_bytes(aff_point.x, aff_x_private);
    uint256_to_bytes(aff_point.y, aff_y_private);

    for (int i = 0; i < 32; i++) {
        aff_x_buffer[i] = aff_x_private[i];
        aff_y_buffer[i] = aff_y_private[i];
    }
}