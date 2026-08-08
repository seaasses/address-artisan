#include "src/opencl/headers/secp256k1/jacobian_to_affine.cl.h"
#include "src/opencl/structs/structs.cl.h"
#include "src/opencl/headers/modular_operations/modular_multiplication.cl.h"
#include "src/opencl/headers/modular_operations/modular_inverse.cl.h"
#include "src/opencl/definitions/secp256k1.cl.h"

// Convert to affine given a precomputed z_inverse (= point_jac.z^-1 mod p).
// Lets callers batch the expensive inversion across many points instead of
// paying one modular_inverse per point.
inline Point jacobian_to_affine_with_z_inverse(const JacobianPoint point_jac, const Uint256 z_inverse)
{
    Point point;

    point.y = z_inverse;

    point.x = modular_multiplication(point.y, point.y); // z^-2
    point.y = modular_multiplication(point.x, point.y); // z^-3

    point.x = modular_multiplication(point.x, point_jac.x); // X / z^2
    point.y = modular_multiplication(point.y, point_jac.y); // Y / z^3

    return point;
}

inline Point jacobian_to_affine(const JacobianPoint point_jac)
{
    return jacobian_to_affine_with_z_inverse(point_jac, modular_inverse(point_jac.z));
}