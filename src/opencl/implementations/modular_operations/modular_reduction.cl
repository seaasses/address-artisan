#include "src/opencl/headers/modular_operations/modular_reduction.cl.h"
#include "src/opencl/definitions/secp256k1.cl.h"
#include "src/opencl/headers/big_uint/big_uint_subtraction.cl.h"
#include "src/opencl/structs/structs.cl.h"

// Reduces a full 512-bit value modulo the secp256k1 prime p = 2^256 - 2^32 - 977.
// Since 2^256 = c (mod p) with c = 2^32 + 977, the reduction folds the high half
// H via H*c = (H << 32) + H*977, adds it to the low half L, folds the small
// (~33-bit) remainder once more, then does one conditional subtraction of p.
// Valid for ANY Uint512 input; the result is fully reduced (< p).
//
// Bounds: first fold each word accumulates carry + lo (<2^32) + 977*hi (<2^42) +
// hi<<32 word (<2^32) < 2^43. Second fold folds H2 < 2^34, H2*977 < 2^44,
// H2<<32 < 2^66 -> at most one bit-256 carry, folded as +c. Final: one subtract.
inline Uint256 modular_reduction(const Uint512 x)
{
    // LSW-first views: hi = high 256 bits, lo = low 256 bits.
    uint hi[8];
    uint lo[8];
#pragma unroll
    for (int k = 0; k < 8; k++)
    {
        hi[k] = x.limbs[7 - k];
        lo[k] = x.limbs[15 - k];
    }

    // FIRST FOLD: r10 = lo + hi*977 + (hi << 32), 10 words LSW-first.
    uint r10[10];
    ulong acc = 0;
#pragma unroll
    for (int k = 0; k < 10; k++)
    {
        if (k < 8)
            acc += (ulong)lo[k];
        if (k < 8)
            acc += (ulong)SECP256K1_REDUCE_C_977 * (ulong)hi[k];
        if (k >= 1 && (k - 1) < 8)
            acc += (ulong)hi[k - 1];
        r10[k] = (uint)acc;
        acc >>= 32;
    }

    // SECOND FOLD: r10[8],r10[9] hold the part >= 2^256 (~33 bits). Fold H2*c.
    ulong h2 = (ulong)r10[8] | ((ulong)r10[9] << 32);
    ulong m977 = h2 * (ulong)SECP256K1_REDUCE_C_977;
    uint res[8];
    acc = (ulong)r10[0] + (m977 & 0xFFFFFFFFUL);
    res[0] = (uint)acc;
    acc >>= 32;
    acc += (ulong)r10[1] + (m977 >> 32) + (h2 & 0xFFFFFFFFUL); // (H2 << 32) low word
    res[1] = (uint)acc;
    acc >>= 32;
    acc += (ulong)r10[2] + (h2 >> 32); // (H2 << 32) high word
    res[2] = (uint)acc;
    acc >>= 32;
#pragma unroll
    for (int k = 3; k < 8; k++)
    {
        acc += (ulong)r10[k];
        res[k] = (uint)acc;
        acc >>= 32;
    }
    uint carry256 = (uint)acc; // 0 or 1

    // FOLD the bit-256 carry: 2^256 = c (mod p). Add c*carry256 into res.
    ulong cc = (ulong)carry256;
    acc = (ulong)res[0] + cc * (ulong)SECP256K1_REDUCE_C_977;
    res[0] = (uint)acc;
    acc >>= 32;
    acc += (ulong)res[1] + cc; // c's 2^32 term
    res[1] = (uint)acc;
    acc >>= 32;
#pragma unroll
    for (int k = 2; k < 8; k++)
    {
        acc += (ulong)res[k];
        res[k] = (uint)acc;
        acc >>= 32;
    }

    // Rebuild big-limb-first, then subtract p if result >= p (branchless).
    Uint256 out;
#pragma unroll
    for (int k = 0; k < 8; k++)
        out.limbs[7 - k] = res[k];

    uint ge = 0;
    uint eq = 1;
#define AA_CMP(i)                                                       \
    do                                                                  \
    {                                                                   \
        ge |= (eq & (out.limbs[i] > (uint)SECP256K1_P_##i));            \
        eq &= (out.limbs[i] == (uint)SECP256K1_P_##i);                  \
    } while (0)
    AA_CMP(0);
    AA_CMP(1);
    AA_CMP(2);
    AA_CMP(3);
    AA_CMP(4);
    AA_CMP(5);
    AA_CMP(6);
    AA_CMP(7);
#undef AA_CMP

    uint sub_mask = -(ge | eq); // all-ones if out >= p, else 0
    Uint256 p_masked;
    p_masked.limbs[0] = (uint)SECP256K1_P_0 & sub_mask;
    p_masked.limbs[1] = (uint)SECP256K1_P_1 & sub_mask;
    p_masked.limbs[2] = (uint)SECP256K1_P_2 & sub_mask;
    p_masked.limbs[3] = (uint)SECP256K1_P_3 & sub_mask;
    p_masked.limbs[4] = (uint)SECP256K1_P_4 & sub_mask;
    p_masked.limbs[5] = (uint)SECP256K1_P_5 & sub_mask;
    p_masked.limbs[6] = (uint)SECP256K1_P_6 & sub_mask;
    p_masked.limbs[7] = (uint)SECP256K1_P_7 & sub_mask;

    return uint256_subtraction(out, p_masked);
}
