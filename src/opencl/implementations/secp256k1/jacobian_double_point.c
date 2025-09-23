#include "src/opencl/definitions/secp256k1.h"
#include "src/opencl/headers/modular_operations/modular_addition.h"
#include "src/opencl/headers/modular_operations/modular_subtraction.h"
#include "src/opencl/headers/modular_operations/modular_multiplication.h"
#include "src/opencl/headers/modular_operations/modular_double.h"
#include "src/opencl/headers/secp256k1/jacobian_double_point.h"

inline void jacobian_double_point(const JacobianPoint *point, JacobianPoint *result_point)
{

    Uint256 tmp_0, tmp_1;

    modular_multiplication(&point->y, &point->y, &tmp_1);
    modular_multiplication(&point->x, &tmp_1, &result_point->y);

    // TODO: maybe test like a modular_multiply_by_2_power_n that so I can do this in one go and use
    // less uint256 shifts and sums/subtractions. but I think this is OK for now
    modular_double(&result_point->y, &result_point->y);
    modular_double(&result_point->y, &result_point->y);

    modular_multiplication(&tmp_1, &tmp_1, &tmp_0);                               // --------------
    modular_multiplication(&point->x, &point->x, &result_point->z);               // ----------------------
    modular_double(&result_point->z, &result_point->x);                           // ---------------
    modular_addition(&result_point->z, &result_point->x, &result_point->z);       // ---------------
    modular_multiplication(&result_point->z, &result_point->z, &result_point->x); // --------------------

    modular_double(&result_point->y, &tmp_1);                        // -------------
    modular_subtraction(&result_point->x, &tmp_1, &result_point->x); // result_point.x = X3 :D

    modular_subtraction(&result_point->y, &result_point->x, &tmp_1);
    modular_multiplication(&result_point->z, &tmp_1, &result_point->z);

    // TODO: same thing, this could be one that use a shift 3
    modular_double(&tmp_0, &tmp_0);
    modular_double(&tmp_0, &tmp_0);
    modular_double(&tmp_0, &tmp_0);

    modular_subtraction(&result_point->z, &tmp_0, &result_point->y);

    modular_multiplication(&point->y, &point->z, &result_point->z);
    modular_double(&result_point->z, &result_point->z); // result.z = Z3 :D
}
