#include "src/opencl/definitions/secp256k1.cl.h"
#include "src/opencl/headers/secp256k1/g_times_scalar.cl.h"
#include "src/opencl/headers/secp256k1/jacobian_point_affine_point_addition.cl.h"

// Fixed-base scalar multiplication using precomputed tables.
//
// g_times_tables is a flat array of 64 tables with 16 points each:
//   g_times_tables[w * 16 + d] = d * 16^w * G
// where w is the 4-bit window position (0 = least significant nibble)
// and d is the digit value (0..15). The d = 0 entries are the point at
// infinity, encoded as x = P, y = 0 (an impossible affine x coordinate).
//
// The scalar is decomposed into 64 nibbles and the result is the sum of
// one table point per nibble: no point doubling is needed. Zero digits are
// handled by the infinity branches inside the point addition; those
// branches are warp-uniform only per-thread, but cheap compared to the
// eliminated doublings.
inline JacobianPoint g_times_scalar(const Uint256 scalar, __global const Point *g_times_tables)
{
    JacobianPoint result = {
        {.limbs = {0, 0, 0, 0}},
        {.limbs = {0, 0, 0, 0}},
        {.limbs = {0, 0, 0, 0}}}; // z = 0: point at infinity

    int w = 63; // limbs[0] holds the most significant bits
    for (int limb_index = 0; limb_index < 4; limb_index++)
    {
        const ulong limb = scalar.limbs[limb_index];
        for (int shift = 60; shift >= 0; shift -= 4)
        {
            const ulong digit = (limb >> shift) & 15;
            result = jacobian_point_affine_point_addition(result, g_times_tables[(w << 4) + digit]);
            w--;
        }
    }

    return result;
}
