#include "src/opencl/structs/structs.cl.h"
#include "src/opencl/headers/secp256k1/ckdpub.cl.h"
#include "src/opencl/headers/secp256k1/compress_point.cl.h"
#include "src/opencl/headers/hash/hmac_sha512.cl.h"
#include "src/opencl/headers/big_uint/big_uint_from_bytes.cl.h"
#include "src/opencl/headers/big_uint/big_uint_to_bytes.cl.h"
#include "src/opencl/headers/secp256k1/g_times_scalar.cl.h"
#include "src/opencl/headers/secp256k1/jacobian_point_affine_point_addition.cl.h"
#include "src/opencl/headers/secp256k1/jacobian_to_affine.cl.h"

// CKDpub up to (but not including) the jacobian -> affine conversion.
// Callers that process several children can batch the expensive modular
// inversions of the Z coordinates (see modular_inverse_batch) instead of
// paying one full inversion per child.
inline JacobianPoint ckdpub_jacobian(
    const XPub parent,
    uint index,
    __global const Point *g_times_tables)
{
    uchar compressed_key[33];
    compressed_key[0] = (uchar)(0x02 | (((uchar)(parent.k_par.y.limbs[3])) & 1));
    uint256_to_bytes(parent.k_par.x, &compressed_key[1]);

    uchar hmac_message[37];
    for (uchar i = 0; i < 33; i++)
    {
        hmac_message[i] = compressed_key[i];
    }

    hmac_message[33] = (uchar)(index >> 24);
    hmac_message[34] = (uchar)(index >> 16);
    hmac_message[35] = (uchar)(index >> 8);
    hmac_message[36] = (uchar)(index);

    uchar hmac_hash[64];
    hmac_sha512_key32_msg37(parent.chain_code, hmac_message, hmac_hash);

    return jacobian_point_affine_point_addition(
        g_times_scalar(
            UINT256_FROM_BYTES(hmac_hash), g_times_tables),
        parent.k_par);
}

inline void ckdpub(
    const XPub parent,
    uint index,
    uchar *restrict result,
    __global const Point *g_times_tables)
{
    Point k_child = jacobian_to_affine(ckdpub_jacobian(parent, index, g_times_tables));

    result[0] = (uchar)(0x02 | (((uchar)(k_child.y.limbs[3])) & 1));
    uint256_to_bytes(k_child.x, &result[1]);
}
