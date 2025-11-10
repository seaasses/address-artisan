#include "src/opencl/structs/structs.cl.h"

#define UINT256_FROM_BYTES(input) \
    ((Uint256){ \
        .limbs = { \
            (((ulong)((input)[0]) << 56) | ((ulong)((input)[1]) << 48) | \
             ((ulong)((input)[2]) << 40) | ((ulong)((input)[3]) << 32) | \
             ((ulong)((input)[4]) << 24) | ((ulong)((input)[5]) << 16) | \
             ((ulong)((input)[6]) << 8) | ((ulong)((input)[7]))), \
            (((ulong)((input)[8]) << 56) | ((ulong)((input)[9]) << 48) | \
             ((ulong)((input)[10]) << 40) | ((ulong)((input)[11]) << 32) | \
             ((ulong)((input)[12]) << 24) | ((ulong)((input)[13]) << 16) | \
             ((ulong)((input)[14]) << 8) | ((ulong)((input)[15]))), \
            (((ulong)((input)[16]) << 56) | ((ulong)((input)[17]) << 48) | \
             ((ulong)((input)[18]) << 40) | ((ulong)((input)[19]) << 32) | \
             ((ulong)((input)[20]) << 24) | ((ulong)((input)[21]) << 16) | \
             ((ulong)((input)[22]) << 8) | ((ulong)((input)[23]))), \
            (((ulong)((input)[24]) << 56) | ((ulong)((input)[25]) << 48) | \
             ((ulong)((input)[26]) << 40) | ((ulong)((input)[27]) << 32) | \
             ((ulong)((input)[28]) << 24) | ((ulong)((input)[29]) << 16) | \
             ((ulong)((input)[30]) << 8) | ((ulong)((input)[31]))) \
        } \
    })

#define UINT320_FROM_BYTES(input) \
    ((Uint320){ \
        .limbs = { \
            (((ulong)((input)[0]) << 56) | ((ulong)((input)[1]) << 48) | \
             ((ulong)((input)[2]) << 40) | ((ulong)((input)[3]) << 32) | \
             ((ulong)((input)[4]) << 24) | ((ulong)((input)[5]) << 16) | \
             ((ulong)((input)[6]) << 8) | ((ulong)((input)[7]))), \
            (((ulong)((input)[8]) << 56) | ((ulong)((input)[9]) << 48) | \
             ((ulong)((input)[10]) << 40) | ((ulong)((input)[11]) << 32) | \
             ((ulong)((input)[12]) << 24) | ((ulong)((input)[13]) << 16) | \
             ((ulong)((input)[14]) << 8) | ((ulong)((input)[15]))), \
            (((ulong)((input)[16]) << 56) | ((ulong)((input)[17]) << 48) | \
             ((ulong)((input)[18]) << 40) | ((ulong)((input)[19]) << 32) | \
             ((ulong)((input)[20]) << 24) | ((ulong)((input)[21]) << 16) | \
             ((ulong)((input)[22]) << 8) | ((ulong)((input)[23]))), \
            (((ulong)((input)[24]) << 56) | ((ulong)((input)[25]) << 48) | \
             ((ulong)((input)[26]) << 40) | ((ulong)((input)[27]) << 32) | \
             ((ulong)((input)[28]) << 24) | ((ulong)((input)[29]) << 16) | \
             ((ulong)((input)[30]) << 8) | ((ulong)((input)[31]))), \
            (((ulong)((input)[32]) << 56) | ((ulong)((input)[33]) << 48) | \
             ((ulong)((input)[34]) << 40) | ((ulong)((input)[35]) << 32) | \
             ((ulong)((input)[36]) << 24) | ((ulong)((input)[37]) << 16) | \
             ((ulong)((input)[38]) << 8) | ((ulong)((input)[39]))) \
        } \
    })

#define UINT512_FROM_BYTES(input) \
    ((Uint512){ \
        .limbs = { \
            (((ulong)((input)[0]) << 56) | ((ulong)((input)[1]) << 48) | \
             ((ulong)((input)[2]) << 40) | ((ulong)((input)[3]) << 32) | \
             ((ulong)((input)[4]) << 24) | ((ulong)((input)[5]) << 16) | \
             ((ulong)((input)[6]) << 8) | ((ulong)((input)[7]))), \
            (((ulong)((input)[8]) << 56) | ((ulong)((input)[9]) << 48) | \
             ((ulong)((input)[10]) << 40) | ((ulong)((input)[11]) << 32) | \
             ((ulong)((input)[12]) << 24) | ((ulong)((input)[13]) << 16) | \
             ((ulong)((input)[14]) << 8) | ((ulong)((input)[15]))), \
            (((ulong)((input)[16]) << 56) | ((ulong)((input)[17]) << 48) | \
             ((ulong)((input)[18]) << 40) | ((ulong)((input)[19]) << 32) | \
             ((ulong)((input)[20]) << 24) | ((ulong)((input)[21]) << 16) | \
             ((ulong)((input)[22]) << 8) | ((ulong)((input)[23]))), \
            (((ulong)((input)[24]) << 56) | ((ulong)((input)[25]) << 48) | \
             ((ulong)((input)[26]) << 40) | ((ulong)((input)[27]) << 32) | \
             ((ulong)((input)[28]) << 24) | ((ulong)((input)[29]) << 16) | \
             ((ulong)((input)[30]) << 8) | ((ulong)((input)[31]))), \
            (((ulong)((input)[32]) << 56) | ((ulong)((input)[33]) << 48) | \
             ((ulong)((input)[34]) << 40) | ((ulong)((input)[35]) << 32) | \
             ((ulong)((input)[36]) << 24) | ((ulong)((input)[37]) << 16) | \
             ((ulong)((input)[38]) << 8) | ((ulong)((input)[39]))), \
            (((ulong)((input)[40]) << 56) | ((ulong)((input)[41]) << 48) | \
             ((ulong)((input)[42]) << 40) | ((ulong)((input)[43]) << 32) | \
             ((ulong)((input)[44]) << 24) | ((ulong)((input)[45]) << 16) | \
             ((ulong)((input)[46]) << 8) | ((ulong)((input)[47]))), \
            (((ulong)((input)[48]) << 56) | ((ulong)((input)[49]) << 48) | \
             ((ulong)((input)[50]) << 40) | ((ulong)((input)[51]) << 32) | \
             ((ulong)((input)[52]) << 24) | ((ulong)((input)[53]) << 16) | \
             ((ulong)((input)[54]) << 8) | ((ulong)((input)[55]))), \
            (((ulong)((input)[56]) << 56) | ((ulong)((input)[57]) << 48) | \
             ((ulong)((input)[58]) << 40) | ((ulong)((input)[59]) << 32) | \
             ((ulong)((input)[60]) << 24) | ((ulong)((input)[61]) << 16) | \
             ((ulong)((input)[62]) << 8) | ((ulong)((input)[63]))) \
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