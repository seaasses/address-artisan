#include "src/opencl/headers/big_uint/big_uint_multiplication.cl.h"

// 256x256 -> 512 multiply, big-limb-first (limbs[0] most significant).
//
// Two implementations, selected at OpenCL compile time:
//   - NVIDIA (defines __NV_CL_C_VERSION, or forced with -D AA_USE_PTX): an
//     operand-scanning multiply that chains the hardware carry flag through
//     inline PTX (mad.lo.cc / madc.hi.cc / addc). This path is ported from
//     BitCrack (brichard19/BitCrack, cudaMath/secp256k1.cuh + cudaMath/ptx.cuh),
//     adapted to OpenCL inline asm and this codebase's big-limb-first Uint512.
//   - Everything else (Intel Arc, CPU, ...): a portable Comba column-scan with a
//     ulong accumulator. No inline asm.
// Both produce the identical 512-bit product.

#if defined(__NV_CL_C_VERSION) || defined(AA_USE_PTX)

#define AA_mad_lo_cc(d, a, x, b)  asm volatile("mad.lo.cc.u32 %0, %1, %2, %3;" : "=r"(d) : "r"(a), "r"(x), "r"(b))
#define AA_madc_lo_cc(d, a, x, b) asm volatile("madc.lo.cc.u32 %0, %1, %2, %3;" : "=r"(d) : "r"(a), "r"(x), "r"(b))
#define AA_mad_hi_cc(d, a, x, b)  asm volatile("mad.hi.cc.u32 %0, %1, %2, %3;" : "=r"(d) : "r"(a), "r"(x), "r"(b))
#define AA_madc_hi_cc(d, a, x, b) asm volatile("madc.hi.cc.u32 %0, %1, %2, %3;" : "=r"(d) : "r"(a), "r"(x), "r"(b))
#define AA_madc_hi(d, a, x, b)    asm volatile("madc.hi.u32 %0, %1, %2, %3;" : "=r"(d) : "r"(a), "r"(x), "r"(b))
#define AA_addc(d, a, b)          asm volatile("addc.u32 %0, %1, %2;" : "=r"(d) : "r"(a), "r"(b))

inline Uint512 uint256_multiplication(const Uint256 a, const Uint256 b)
{
    // Big-limb-first throughout: index 0 is the most significant 32-bit word.
    // c[] accumulates the low 256 bits, high[] the high 256 bits.
    uint c[8];
    uint high[8] = {0, 0, 0, 0, 0, 0, 0, 0};
    uint t;

    // a[7] * b (low)
    t = a.limbs[7];
    for (int i = 7; i >= 0; i--)
        c[i] = t * b.limbs[i];
    // a[7] * b (high)
    AA_mad_hi_cc(c[6], t, b.limbs[7], c[6]);
    AA_madc_hi_cc(c[5], t, b.limbs[6], c[5]);
    AA_madc_hi_cc(c[4], t, b.limbs[5], c[4]);
    AA_madc_hi_cc(c[3], t, b.limbs[4], c[3]);
    AA_madc_hi_cc(c[2], t, b.limbs[3], c[2]);
    AA_madc_hi_cc(c[1], t, b.limbs[2], c[1]);
    AA_madc_hi_cc(c[0], t, b.limbs[1], c[0]);
    AA_madc_hi(high[7], t, b.limbs[0], high[7]);

    // a[6] * b (low)
    t = a.limbs[6];
    AA_mad_lo_cc(c[6], t, b.limbs[7], c[6]);
    AA_madc_lo_cc(c[5], t, b.limbs[6], c[5]);
    AA_madc_lo_cc(c[4], t, b.limbs[5], c[4]);
    AA_madc_lo_cc(c[3], t, b.limbs[4], c[3]);
    AA_madc_lo_cc(c[2], t, b.limbs[3], c[2]);
    AA_madc_lo_cc(c[1], t, b.limbs[2], c[1]);
    AA_madc_lo_cc(c[0], t, b.limbs[1], c[0]);
    AA_madc_lo_cc(high[7], t, b.limbs[0], high[7]);
    AA_addc(high[6], high[6], 0);
    // a[6] * b (high)
    AA_mad_hi_cc(c[5], t, b.limbs[7], c[5]);
    AA_madc_hi_cc(c[4], t, b.limbs[6], c[4]);
    AA_madc_hi_cc(c[3], t, b.limbs[5], c[3]);
    AA_madc_hi_cc(c[2], t, b.limbs[4], c[2]);
    AA_madc_hi_cc(c[1], t, b.limbs[3], c[1]);
    AA_madc_hi_cc(c[0], t, b.limbs[2], c[0]);
    AA_madc_hi_cc(high[7], t, b.limbs[1], high[7]);
    AA_madc_hi(high[6], t, b.limbs[0], high[6]);

    // a[5] * b (low)
    t = a.limbs[5];
    AA_mad_lo_cc(c[5], t, b.limbs[7], c[5]);
    AA_madc_lo_cc(c[4], t, b.limbs[6], c[4]);
    AA_madc_lo_cc(c[3], t, b.limbs[5], c[3]);
    AA_madc_lo_cc(c[2], t, b.limbs[4], c[2]);
    AA_madc_lo_cc(c[1], t, b.limbs[3], c[1]);
    AA_madc_lo_cc(c[0], t, b.limbs[2], c[0]);
    AA_madc_lo_cc(high[7], t, b.limbs[1], high[7]);
    AA_madc_lo_cc(high[6], t, b.limbs[0], high[6]);
    AA_addc(high[5], high[5], 0);
    // a[5] * b (high)
    AA_mad_hi_cc(c[4], t, b.limbs[7], c[4]);
    AA_madc_hi_cc(c[3], t, b.limbs[6], c[3]);
    AA_madc_hi_cc(c[2], t, b.limbs[5], c[2]);
    AA_madc_hi_cc(c[1], t, b.limbs[4], c[1]);
    AA_madc_hi_cc(c[0], t, b.limbs[3], c[0]);
    AA_madc_hi_cc(high[7], t, b.limbs[2], high[7]);
    AA_madc_hi_cc(high[6], t, b.limbs[1], high[6]);
    AA_madc_hi(high[5], t, b.limbs[0], high[5]);

    // a[4] * b (low)
    t = a.limbs[4];
    AA_mad_lo_cc(c[4], t, b.limbs[7], c[4]);
    AA_madc_lo_cc(c[3], t, b.limbs[6], c[3]);
    AA_madc_lo_cc(c[2], t, b.limbs[5], c[2]);
    AA_madc_lo_cc(c[1], t, b.limbs[4], c[1]);
    AA_madc_lo_cc(c[0], t, b.limbs[3], c[0]);
    AA_madc_lo_cc(high[7], t, b.limbs[2], high[7]);
    AA_madc_lo_cc(high[6], t, b.limbs[1], high[6]);
    AA_madc_lo_cc(high[5], t, b.limbs[0], high[5]);
    AA_addc(high[4], high[4], 0);
    // a[4] * b (high)
    AA_mad_hi_cc(c[3], t, b.limbs[7], c[3]);
    AA_madc_hi_cc(c[2], t, b.limbs[6], c[2]);
    AA_madc_hi_cc(c[1], t, b.limbs[5], c[1]);
    AA_madc_hi_cc(c[0], t, b.limbs[4], c[0]);
    AA_madc_hi_cc(high[7], t, b.limbs[3], high[7]);
    AA_madc_hi_cc(high[6], t, b.limbs[2], high[6]);
    AA_madc_hi_cc(high[5], t, b.limbs[1], high[5]);
    AA_madc_hi(high[4], t, b.limbs[0], high[4]);

    // a[3] * b (low)
    t = a.limbs[3];
    AA_mad_lo_cc(c[3], t, b.limbs[7], c[3]);
    AA_madc_lo_cc(c[2], t, b.limbs[6], c[2]);
    AA_madc_lo_cc(c[1], t, b.limbs[5], c[1]);
    AA_madc_lo_cc(c[0], t, b.limbs[4], c[0]);
    AA_madc_lo_cc(high[7], t, b.limbs[3], high[7]);
    AA_madc_lo_cc(high[6], t, b.limbs[2], high[6]);
    AA_madc_lo_cc(high[5], t, b.limbs[1], high[5]);
    AA_madc_lo_cc(high[4], t, b.limbs[0], high[4]);
    AA_addc(high[3], high[3], 0);
    // a[3] * b (high)
    AA_mad_hi_cc(c[2], t, b.limbs[7], c[2]);
    AA_madc_hi_cc(c[1], t, b.limbs[6], c[1]);
    AA_madc_hi_cc(c[0], t, b.limbs[5], c[0]);
    AA_madc_hi_cc(high[7], t, b.limbs[4], high[7]);
    AA_madc_hi_cc(high[6], t, b.limbs[3], high[6]);
    AA_madc_hi_cc(high[5], t, b.limbs[2], high[5]);
    AA_madc_hi_cc(high[4], t, b.limbs[1], high[4]);
    AA_madc_hi(high[3], t, b.limbs[0], high[3]);

    // a[2] * b (low)
    t = a.limbs[2];
    AA_mad_lo_cc(c[2], t, b.limbs[7], c[2]);
    AA_madc_lo_cc(c[1], t, b.limbs[6], c[1]);
    AA_madc_lo_cc(c[0], t, b.limbs[5], c[0]);
    AA_madc_lo_cc(high[7], t, b.limbs[4], high[7]);
    AA_madc_lo_cc(high[6], t, b.limbs[3], high[6]);
    AA_madc_lo_cc(high[5], t, b.limbs[2], high[5]);
    AA_madc_lo_cc(high[4], t, b.limbs[1], high[4]);
    AA_madc_lo_cc(high[3], t, b.limbs[0], high[3]);
    AA_addc(high[2], high[2], 0);
    // a[2] * b (high)
    AA_mad_hi_cc(c[1], t, b.limbs[7], c[1]);
    AA_madc_hi_cc(c[0], t, b.limbs[6], c[0]);
    AA_madc_hi_cc(high[7], t, b.limbs[5], high[7]);
    AA_madc_hi_cc(high[6], t, b.limbs[4], high[6]);
    AA_madc_hi_cc(high[5], t, b.limbs[3], high[5]);
    AA_madc_hi_cc(high[4], t, b.limbs[2], high[4]);
    AA_madc_hi_cc(high[3], t, b.limbs[1], high[3]);
    AA_madc_hi(high[2], t, b.limbs[0], high[2]);

    // a[1] * b (low)
    t = a.limbs[1];
    AA_mad_lo_cc(c[1], t, b.limbs[7], c[1]);
    AA_madc_lo_cc(c[0], t, b.limbs[6], c[0]);
    AA_madc_lo_cc(high[7], t, b.limbs[5], high[7]);
    AA_madc_lo_cc(high[6], t, b.limbs[4], high[6]);
    AA_madc_lo_cc(high[5], t, b.limbs[3], high[5]);
    AA_madc_lo_cc(high[4], t, b.limbs[2], high[4]);
    AA_madc_lo_cc(high[3], t, b.limbs[1], high[3]);
    AA_madc_lo_cc(high[2], t, b.limbs[0], high[2]);
    AA_addc(high[1], high[1], 0);
    // a[1] * b (high)
    AA_mad_hi_cc(c[0], t, b.limbs[7], c[0]);
    AA_madc_hi_cc(high[7], t, b.limbs[6], high[7]);
    AA_madc_hi_cc(high[6], t, b.limbs[5], high[6]);
    AA_madc_hi_cc(high[5], t, b.limbs[4], high[5]);
    AA_madc_hi_cc(high[4], t, b.limbs[3], high[4]);
    AA_madc_hi_cc(high[3], t, b.limbs[2], high[3]);
    AA_madc_hi_cc(high[2], t, b.limbs[1], high[2]);
    AA_madc_hi(high[1], t, b.limbs[0], high[1]);

    // a[0] * b (low)
    t = a.limbs[0];
    AA_mad_lo_cc(c[0], t, b.limbs[7], c[0]);
    AA_madc_lo_cc(high[7], t, b.limbs[6], high[7]);
    AA_madc_lo_cc(high[6], t, b.limbs[5], high[6]);
    AA_madc_lo_cc(high[5], t, b.limbs[4], high[5]);
    AA_madc_lo_cc(high[4], t, b.limbs[3], high[4]);
    AA_madc_lo_cc(high[3], t, b.limbs[2], high[3]);
    AA_madc_lo_cc(high[2], t, b.limbs[1], high[2]);
    AA_madc_lo_cc(high[1], t, b.limbs[0], high[1]);
    AA_addc(high[0], high[0], 0);
    // a[0] * b (high)
    AA_mad_hi_cc(high[7], t, b.limbs[7], high[7]);
    AA_madc_hi_cc(high[6], t, b.limbs[6], high[6]);
    AA_madc_hi_cc(high[5], t, b.limbs[5], high[5]);
    AA_madc_hi_cc(high[4], t, b.limbs[4], high[4]);
    AA_madc_hi_cc(high[3], t, b.limbs[3], high[3]);
    AA_madc_hi_cc(high[2], t, b.limbs[2], high[2]);
    AA_madc_hi_cc(high[1], t, b.limbs[1], high[1]);
    AA_madc_hi(high[0], t, b.limbs[0], high[0]);

    Uint512 r;
#pragma unroll
    for (int k = 0; k < 8; k++)
    {
        r.limbs[k] = high[k];     // upper 256 bits
        r.limbs[8 + k] = c[k];    // lower 256 bits
    }
    return r;
}

#undef AA_mad_lo_cc
#undef AA_madc_lo_cc
#undef AA_mad_hi_cc
#undef AA_madc_hi_cc
#undef AA_madc_hi
#undef AA_addc

#else // portable path (Intel Arc, CPU, non-NVIDIA)

// Schoolbook (Comba / column-scanning) 256x256 -> 512 multiply with 8x32-bit
// limbs. Each partial product a[i]*b[j] is a native 32x32 -> 64 multiply.
// Term a[i]*b[j] lands in output column s = i+j (LSW-first) -> r.limbs[15 - s].
// A column sums up to 8 products < 2^64 -> < 2^67, held in a 64-bit accumulator
// plus a uint carry counting 2^64 wraps.
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

#endif
