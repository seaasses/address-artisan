#include "src/opencl/headers/big_uint/big_uint_multiplication.cl.h"

// Schoolbook (Comba / column-scanning) 256x256 -> 512 multiply with 8x32-bit
// limbs. Each partial product a[i]*b[j] is a native 32x32 -> 64 multiply.
//
// Big-limb-first layout: limbs[0] is most significant. Term a[i]*b[j] has
// weight 2^(32*(14-i-j)), so it lands in output column s = i+j (LSW-first),
// i.e. output word r.limbs[15 - s]. Columns run s = 0..14; the top output
// word (r.limbs[0], position 15) only receives the carry-out of column 14.
//
// A column sums up to 8 products, each < 2^64, so the running total is < 2^67.
// It is held in a 64-bit accumulator `acc` plus a `uint acc_carry` counting
// 2^64 wraps (<= 8). Between columns the value is shifted down 32 bits as
// (acc >> 32) | (acc_carry << 32) < 2^35.
inline Uint512 uint256_multiplication(const Uint256 a, const Uint256 b)
{
    Uint512 r;
    ulong acc = 0;
    uint acc_carry = 0;

#pragma unroll
    for (int s = 0; s <= 14; s++)
    {
        int i_lo = (s > 7) ? (s - 7) : 0;
        int i_hi = (s < 7) ? s : 7;
        for (int i = i_lo; i <= i_hi; i++)
        {
            int j = s - i;
            ulong prod = (ulong)a.limbs[7 - i] * (ulong)b.limbs[7 - j];
            ulong old = acc;
            acc += prod;
            acc_carry += (acc < old);
        }
        r.limbs[15 - s] = (uint)acc;
        acc = (acc >> 32) | ((ulong)acc_carry << 32);
        acc_carry = 0;
    }
    r.limbs[0] = (uint)acc;

    return r;
}
