#include "src/opencl/headers/big_uint/big_uint_square.cl.h"
#include "src/opencl/headers/big_uint/big_uint_multiplication.cl.h"

// Squaring via the general multiply (like hashcat/BitCrack, which have no
// dedicated square). A dedicated 8x32 square (8 diagonal + 28 doubled
// off-diagonal products, ~44% fewer partial products) is a future optimization
// that can replace this body without touching any caller.
inline Uint512 uint256_square(const Uint256 a)
{
    return uint256_multiplication(a, a);
}
