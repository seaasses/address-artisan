#include "src/opencl/headers/secp256k1/jacobian_double_point.h"
#include "src/opencl/headers/big_uint/big_uint_from_bytes.h"
#include "src/opencl/headers/big_uint/big_uint_to_bytes.h"

__kernel void jacobian_double_point_kernel(
    __global uchar *point_x_buffer,
    __global uchar *point_y_buffer,
    __global uchar *point_z_buffer,
    __global uchar *result_x_buffer,
    __global uchar *result_y_buffer,
    __global uchar *result_z_buffer)
{
    JacobianPoint point;
    JacobianPoint result;

    // Copy data from global to private memory and convert
    uchar point_x_private[32];
    uchar point_y_private[32];
    uchar point_z_private[32];

    for (int i = 0; i < 32; i++) {
        point_x_private[i] = point_x_buffer[i];
        point_y_private[i] = point_y_buffer[i];
        point_z_private[i] = point_z_buffer[i];
    }

    // Convert byte arrays to Uint256
    point.x = UINT256_FROM_BYTES(point_x_private);
    point.y = UINT256_FROM_BYTES(point_y_private);
    point.z = UINT256_FROM_BYTES(point_z_private);

    // Perform jacobian double point
    result = jacobian_double_point(point);

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