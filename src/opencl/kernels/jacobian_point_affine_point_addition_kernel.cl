#include "src/opencl/headers/secp256k1/jacobian_point_affine_point_addition.cl.h"
#include "src/opencl/headers/big_uint/big_uint_from_bytes.cl.h"
#include "src/opencl/headers/big_uint/big_uint_to_bytes.cl.h"

__kernel void jacobian_point_affine_point_addition_kernel(
    __global uchar *jac_a_x_buffer,
    __global uchar *jac_a_y_buffer,
    __global uchar *jac_a_z_buffer,
    __global uchar *aff_b_x_buffer,
    __global uchar *aff_b_y_buffer,
    __global uchar *result_x_buffer,
    __global uchar *result_y_buffer,
    __global uchar *result_z_buffer)
{
    JacobianPoint jac_a;
    Point aff_b;
    JacobianPoint result;

    // Copy data from global to private memory and convert
    uchar jac_a_x_private[32];
    uchar jac_a_y_private[32];
    uchar jac_a_z_private[32];
    uchar aff_b_x_private[32];
    uchar aff_b_y_private[32];

    for (int i = 0; i < 32; i++) {
        jac_a_x_private[i] = jac_a_x_buffer[i];
        jac_a_y_private[i] = jac_a_y_buffer[i];
        jac_a_z_private[i] = jac_a_z_buffer[i];
        aff_b_x_private[i] = aff_b_x_buffer[i];
        aff_b_y_private[i] = aff_b_y_buffer[i];
    }

    // Convert byte arrays to Uint256
    jac_a.x = UINT256_FROM_BYTES(jac_a_x_private);
    jac_a.y = UINT256_FROM_BYTES(jac_a_y_private);
    jac_a.z = UINT256_FROM_BYTES(jac_a_z_private);
    aff_b.x = UINT256_FROM_BYTES(aff_b_x_private);
    aff_b.y = UINT256_FROM_BYTES(aff_b_y_private);

    // Perform jacobian point + affine point addition
    result = jacobian_point_affine_point_addition(jac_a, aff_b);

    // Convert result back to bytes and copy to global memory
    uchar result_x_private[32];
    uchar result_y_private[32];
    uchar result_z_private[32];
    uint256_to_bytes(result.x, result_x_private);
    uint256_to_bytes(result.y, result_y_private);
    uint256_to_bytes(result.z, result_z_private);

    for (int i = 0; i < 32; i++) {
        result_x_buffer[i] = result_x_private[i];
        result_y_buffer[i] = result_y_private[i];
        result_z_buffer[i] = result_z_private[i];
    }
}