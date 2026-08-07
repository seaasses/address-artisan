#include "src/opencl/structs/structs.cl.h"

void ckdpub(
    const XPub parent,
    uint index,
    uchar *restrict result,
    __global const Point *g_times_tables
);
