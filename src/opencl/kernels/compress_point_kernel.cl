#include "src/opencl/headers/secp256k1/compress_point.cl.h"
#include "src/opencl/headers/big_uint/big_uint_from_bytes.cl.h"

__kernel void compress_point_kernel(
    __global uchar *point_x_buffer,
    __global uchar *point_y_buffer,
    __global uchar *compressed_buffer)
{
    Point point;

    // Copy data from global to private memory
    uchar point_x_private[32];
    uchar point_y_private[32];

    for (int i = 0; i < 32; i++) {
        point_x_private[i] = point_x_buffer[i];
        point_y_private[i] = point_y_buffer[i];
    }

    // Convert byte arrays to Uint256
    point.x = UINT256_FROM_BYTES(point_x_private);
    point.y = UINT256_FROM_BYTES(point_y_private);

    // Compress the point to 33 bytes
    uchar compressed_private[33];
    COMPRESS_POINT(point, compressed_private);

    // Copy result to global memory
    for (int i = 0; i < 33; i++) {
        compressed_buffer[i] = compressed_private[i];
    }
}
