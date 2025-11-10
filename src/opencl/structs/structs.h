#ifndef STRUCTS_H
#define STRUCTS_H

typedef struct
{
  ulong limbs[4];
} Uint256;

typedef struct
{
  ulong limbs[5];
} Uint320;

typedef struct
{
  ulong limbs[8];
} Uint512;

typedef struct
{
  Uint256 result;
  uint overflow;
} Uint256WithOverflow;

typedef struct
{
  Uint256 result;
  uint underflow;
} Uint256WithUnderflow;

typedef struct
{
  Uint256 x;
  Uint256 y;
} Point;

typedef struct
{
  Uint256 x;
  Uint256 y;
  Uint256 z;
} JacobianPoint;

typedef struct
{
  uint b;
  uint a;
} CacheKey;

typedef struct
{
  uchar chain_code[32];
  Point k_par;
} XPub;

#endif // STRUCTS_H