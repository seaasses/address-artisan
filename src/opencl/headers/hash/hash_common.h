// Common hash functions and macros shared between SHA256 and SHA512

#define CH(x, y, z) (((x) & (y)) | ((~(x)) & (z)))
#define MAJ(x, y, z) (((x) & ((y) | (z))) | ((y) & (z)))

#define ROTR(x, n) (((x) >> (n)) | ((x) << (sizeof(x) * 8 - (n))))
#define SHR(x, n) ((x) >> (n))
