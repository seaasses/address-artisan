#include "src/opencl/structs/structs.cl.h"

#define UINT256_FROM_BYTES(input) \
    ((Uint256){ \
        .limbs = { \
            (((uint)((input)[0]) << 24) | ((uint)((input)[1]) << 16) | ((uint)((input)[2]) << 8) | ((uint)((input)[3]))), \
            (((uint)((input)[4]) << 24) | ((uint)((input)[5]) << 16) | ((uint)((input)[6]) << 8) | ((uint)((input)[7]))), \
            (((uint)((input)[8]) << 24) | ((uint)((input)[9]) << 16) | ((uint)((input)[10]) << 8) | ((uint)((input)[11]))), \
            (((uint)((input)[12]) << 24) | ((uint)((input)[13]) << 16) | ((uint)((input)[14]) << 8) | ((uint)((input)[15]))), \
            (((uint)((input)[16]) << 24) | ((uint)((input)[17]) << 16) | ((uint)((input)[18]) << 8) | ((uint)((input)[19]))), \
            (((uint)((input)[20]) << 24) | ((uint)((input)[21]) << 16) | ((uint)((input)[22]) << 8) | ((uint)((input)[23]))), \
            (((uint)((input)[24]) << 24) | ((uint)((input)[25]) << 16) | ((uint)((input)[26]) << 8) | ((uint)((input)[27]))), \
            (((uint)((input)[28]) << 24) | ((uint)((input)[29]) << 16) | ((uint)((input)[30]) << 8) | ((uint)((input)[31]))) \
        } \
    })

#define UINT512_FROM_BYTES(input) \
    ((Uint512){ \
        .limbs = { \
            (((uint)((input)[0]) << 24) | ((uint)((input)[1]) << 16) | ((uint)((input)[2]) << 8) | ((uint)((input)[3]))), \
            (((uint)((input)[4]) << 24) | ((uint)((input)[5]) << 16) | ((uint)((input)[6]) << 8) | ((uint)((input)[7]))), \
            (((uint)((input)[8]) << 24) | ((uint)((input)[9]) << 16) | ((uint)((input)[10]) << 8) | ((uint)((input)[11]))), \
            (((uint)((input)[12]) << 24) | ((uint)((input)[13]) << 16) | ((uint)((input)[14]) << 8) | ((uint)((input)[15]))), \
            (((uint)((input)[16]) << 24) | ((uint)((input)[17]) << 16) | ((uint)((input)[18]) << 8) | ((uint)((input)[19]))), \
            (((uint)((input)[20]) << 24) | ((uint)((input)[21]) << 16) | ((uint)((input)[22]) << 8) | ((uint)((input)[23]))), \
            (((uint)((input)[24]) << 24) | ((uint)((input)[25]) << 16) | ((uint)((input)[26]) << 8) | ((uint)((input)[27]))), \
            (((uint)((input)[28]) << 24) | ((uint)((input)[29]) << 16) | ((uint)((input)[30]) << 8) | ((uint)((input)[31]))), \
            (((uint)((input)[32]) << 24) | ((uint)((input)[33]) << 16) | ((uint)((input)[34]) << 8) | ((uint)((input)[35]))), \
            (((uint)((input)[36]) << 24) | ((uint)((input)[37]) << 16) | ((uint)((input)[38]) << 8) | ((uint)((input)[39]))), \
            (((uint)((input)[40]) << 24) | ((uint)((input)[41]) << 16) | ((uint)((input)[42]) << 8) | ((uint)((input)[43]))), \
            (((uint)((input)[44]) << 24) | ((uint)((input)[45]) << 16) | ((uint)((input)[46]) << 8) | ((uint)((input)[47]))), \
            (((uint)((input)[48]) << 24) | ((uint)((input)[49]) << 16) | ((uint)((input)[50]) << 8) | ((uint)((input)[51]))), \
            (((uint)((input)[52]) << 24) | ((uint)((input)[53]) << 16) | ((uint)((input)[54]) << 8) | ((uint)((input)[55]))), \
            (((uint)((input)[56]) << 24) | ((uint)((input)[57]) << 16) | ((uint)((input)[58]) << 8) | ((uint)((input)[59]))), \
            (((uint)((input)[60]) << 24) | ((uint)((input)[61]) << 16) | ((uint)((input)[62]) << 8) | ((uint)((input)[63]))) \
        } \
    })

#define ULONG_FROM_BYTES(input) \
    (((ulong)((input)[0]) << 56) | ((ulong)((input)[1]) << 48) | \
     ((ulong)((input)[2]) << 40) | ((ulong)((input)[3]) << 32) | \
     ((ulong)((input)[4]) << 24) | ((ulong)((input)[5]) << 16) | \
     ((ulong)((input)[6]) << 8) | ((ulong)((input)[7])))

#define UINT_FROM_BYTES_BE(input) \
    (((uint)((input)[0]) << 24) | \
     ((uint)((input)[1]) << 16) | \
     ((uint)((input)[2]) << 8) | \
     ((uint)((input)[3])))

#define UINT_FROM_BYTES_LE(input) \
    (((uint)((input)[0])) | \
     ((uint)((input)[1]) << 8) | \
     ((uint)((input)[2]) << 16) | \
     ((uint)((input)[3]) << 24))
