#include "src/opencl/headers/secp256k1/jacobian_to_affine.h"
#include "src/opencl/structs/structs.h"
#include "src/opencl/headers/modular_operations/modular_multiplication.h"
#include "src/opencl/headers/modular_operations/modular_inverse.h"
#include "src/opencl/definitions/secp256k1.h"

inline void jacobian_to_affine(const JacobianPoint *point_jac, Point *point)
{
    modular_inverse(&point_jac->z, &point->y);

    modular_multiplication(&point->y, &point->y, &point->x);
    modular_multiplication(&point->x, &point->y, &point->y);

    modular_multiplication(&point->x, &point_jac->x, &point->x);
    modular_multiplication(&point->y, &point_jac->y, &point->y);
}