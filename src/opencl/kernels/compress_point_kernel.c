#include "src/opencl/headers/secp256k1/compress_point.h"
#include "src/opencl/headers/big_uint/big_uint_from_bytes.h"

__kernel void compress_point_kernel(
    __global unsigned char *point_x_buffer,
    __global unsigned char *point_y_buffer,
    __global unsigned char *compressed_buffer)
{
    Point point;

    // Copy data from global to private memory
    unsigned char point_x_private[32];
    unsigned char point_y_private[32];

    for (int i = 0; i < 32; i++) {
        point_x_private[i] = point_x_buffer[i];
        point_y_private[i] = point_y_buffer[i];
    }

    // Convert byte arrays to Uint256
    bytes_to_uint256(point_x_private, &point.x);
    bytes_to_uint256(point_y_private, &point.y);

    // Compress the point to 33 bytes
    unsigned char compressed_private[33];
    COMPRESS_POINT(point, compressed_private);

    // Copy result to global memory
    for (int i = 0; i < 33; i++) {
        compressed_buffer[i] = compressed_private[i];
    }
}
