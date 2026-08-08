#include "src/opencl/structs/structs.cl.h"

JacobianPoint ckdpub_jacobian(
    const XPub parent,
    uint index,
    __global const Point *g_times_tables
);

void ckdpub(
    const XPub parent,
    uint index,
    uchar *restrict result,
    __global const Point *g_times_tables
);
