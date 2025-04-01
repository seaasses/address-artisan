#include "src/opencl/headers/secp256k1/gTimesScalar.h"
#include "src/opencl/definitions/secp256k1.h"

const Point gTimesScalar(const UInt256 scalar)
{
    return SECP256K1_G;
}