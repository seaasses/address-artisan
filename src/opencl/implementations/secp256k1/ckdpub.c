#include "src/opencl/structs/structs.h"
#include "src/opencl/headers/secp256k1/ckdpub.h"
#include "src/opencl/definitions/big_uint.h"
#include "src/opencl/headers/secp256k1/compress_point.h"
#include "src/opencl/headers/hash/hmac_sha512.h"
#include "src/opencl/headers/big_uint/big_uint_from_bytes.h"
#include "src/opencl/headers/big_uint/big_uint_to_bytes.h"
#include "src/opencl/headers/secp256k1/g_times_scalar.h"
#include "src/opencl/headers/secp256k1/jacobian_point_affine_point_addition.h"
#include "src/opencl/headers/secp256k1/jacobian_to_affine.h"

inline Point ckdpub(
    const unsigned char *chain_code,
    const Point k_par,
    unsigned int index)
{
    // 1. Compress parent public key (33 bytes)
    unsigned char compressed_key[33];
    compressed_key[0] = (unsigned char)(0x02 | (((unsigned char)(k_par.y.limbs[3])) & 1));
    uint256_to_bytes(k_par.x, &compressed_key[1]);

    // 2. Create HMAC input: compressed_key || index (big-endian)
    unsigned char hmac_message[37];
    for (unsigned char i = 0; i < 33; i++)
    {
        hmac_message[i] = compressed_key[i];
    }

    // Convert index to big-endian (4 bytes)
    hmac_message[33] = (unsigned char)(index >> 24);
    hmac_message[34] = (unsigned char)(index >> 16);
    hmac_message[35] = (unsigned char)(index >> 8);
    hmac_message[36] = (unsigned char)(index);

    // 3. Compute HMAC-SHA512(chain_code, hmac_message)
    unsigned char hmac_hash[64];
    hmac_sha512_key32_msg37(chain_code, hmac_message, hmac_hash);

    // 4. Parse IL (first 32 bytes) and compute IL * G
    Uint256 IL = uint256_from_bytes(hmac_hash);
    Point IL_G = g_times_scalar(IL);

    // 5. Convert IL*G to Jacobian and add k_par
    // Result = IL*G + k_par
    JacobianPoint IL_G_jacobian = {
        .x = IL_G.x,
        .y = IL_G.y,
        .z = UINT256_ONE
    };

    JacobianPoint result_jacobian = jacobian_point_affine_point_addition(IL_G_jacobian, k_par);

    // 6. Convert back to affine coordinates
    return jacobian_to_affine(result_jacobian);
}
