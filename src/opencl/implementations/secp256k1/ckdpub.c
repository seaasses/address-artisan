#include "src/opencl/structs/structs.h"
#include "src/opencl/headers/secp256k1/ckdpub.h"
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
    unsigned char compressed_key[33];
    compressed_key[0] = (unsigned char)(0x02 | (((unsigned char)(k_par.y.limbs[3])) & 1));
    uint256_to_bytes(k_par.x, &compressed_key[1]);

    unsigned char hmac_message[37];
    for (unsigned char i = 0; i < 33; i++)
    {
        hmac_message[i] = compressed_key[i];
    }

    hmac_message[33] = (unsigned char)(index >> 24);
    hmac_message[34] = (unsigned char)(index >> 16);
    hmac_message[35] = (unsigned char)(index >> 8);
    hmac_message[36] = (unsigned char)(index);

    unsigned char hmac_hash[64];
    hmac_sha512_key32_msg37(chain_code, hmac_message, hmac_hash);

    return jacobian_to_affine(
        jacobian_point_affine_point_addition(
            g_times_scalar(
                uint256_from_bytes(hmac_hash)),
            k_par));
}
