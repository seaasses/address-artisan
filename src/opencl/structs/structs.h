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
  unsigned int overflow;
} Uint256WithOverflow;

typedef struct
{
  Uint256 result;
  unsigned int underflow;
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