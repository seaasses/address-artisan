typedef struct
{
  unsigned long limbs[4];
} Uint256;

typedef struct
{
  unsigned long limbs[5];
} Uint320;

typedef struct
{
  unsigned long limbs[8];
} Uint512;

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