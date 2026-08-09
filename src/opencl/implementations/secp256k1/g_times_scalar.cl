#include "src/opencl/definitions/secp256k1.cl.h"
#include "src/opencl/headers/secp256k1/g_times_scalar.cl.h"
#include "src/opencl/headers/secp256k1/jacobian_point_affine_point_addition.cl.h"

// Fixed-base scalar multiplication using precomputed tables.
//
// g_times_tables is a flat array of 32 tables with 256 points each (512 KB):
//   g_times_tables[w * 256 + d] = d * 256^w * G
// where w is the 8-bit window position (0 = least significant byte)
// and d is the digit value (0..255). The d = 0 entries are the point at
// infinity, encoded as x = P, y = 0 (an impossible affine x coordinate).
//
// The scalar is decomposed into 32 bytes and the result is the sum of
// one table point per byte: no point doubling is needed. Zero digits are
// handled by the infinity branches inside the point addition; those
// branches are warp-uniform only per-thread, but cheap compared to the
// eliminated doublings.
inline JacobianPoint g_times_scalar(const Uint256 scalar, __global const Point *g_times_tables)
{
    JacobianPoint result = {
        {.limbs = {0, 0, 0, 0, 0, 0, 0, 0}},
        {.limbs = {0, 0, 0, 0, 0, 0, 0, 0}},
        {.limbs = {0, 0, 0, 0, 0, 0, 0, 0}}}; // z = 0: point at infinity

    // Walk the tables from the most significant window (w = 31) down.
    // The loops are kept rolled (#pragma unroll 1): fully unrolling 32
    // copies of the branchy point addition crashes NVIDIA's NVVM
    // compiler (SIGSEGV in NvCliCompileBitcode, driver 580.95).
    __global const Point *window = g_times_tables + 31 * 256;

#pragma unroll 1
    for (int limb_index = 0; limb_index < 8; limb_index++)
    {
        const uint limb = scalar.limbs[limb_index];
#pragma unroll 1
        for (int shift = 24; shift >= 0; shift -= 8)
        {
            const uint digit = (limb >> shift) & 255;
            result = jacobian_point_affine_point_addition(result, window[digit]);
            window -= 256;
        }
    }

    return result;
}
