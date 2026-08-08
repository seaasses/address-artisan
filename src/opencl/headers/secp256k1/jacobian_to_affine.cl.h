#include "src/opencl/structs/structs.cl.h"

Point jacobian_to_affine(const JacobianPoint point_jac);
Point jacobian_to_affine_with_z_inverse(const JacobianPoint point_jac, const Uint256 z_inverse);
