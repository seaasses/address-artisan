typedef struct
{
  unsigned long limbs[4];
} UInt256;

typedef struct Point
{
  UInt256 x;
  UInt256 y;
} Point;
