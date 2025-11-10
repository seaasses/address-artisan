#include "src/opencl/headers/secp256k1/g_times_scalar.h"
#include "src/opencl/headers/secp256k1/jacobian_to_affine.h"
#include "src/opencl/headers/big_uint/big_uint_from_bytes.h"
#include "src/opencl/headers/big_uint/big_uint_to_bytes.h"

__kernel void g_times_scalar_kernel(
    __global uchar *scalar_buffer,
    __global uchar *result_x_buffer,
    __global uchar *result_y_buffer)
{
    Uint256 scalar;
    JacobianPoint jacobian_result;
    Point result;

    // Copy data from global to private memory and convert
    uchar scalar_private[32];

    for (int i = 0; i < 32; i++) {
        scalar_private[i] = scalar_buffer[i];
    }

    // Convert byte arrays to Uint256
    scalar = UINT256_FROM_BYTES(scalar_private);

    // Perform g times scalar multiplication (returns Jacobian point)
    jacobian_result = g_times_scalar(scalar);

    // Convert to affine coordinates
    result = jacobian_to_affine(jacobian_result);

    // Convert result back to bytes and copy to global memory
    uchar result_x_private[32];
    uchar result_y_private[32];
    uint256_to_bytes(result.x, result_x_private);
    uint256_to_bytes(result.y, result_y_private);

    for (int i = 0; i < 32; i++) {
        result_x_buffer[i] = result_x_private[i];
        result_y_buffer[i] = result_y_private[i];
    }
}