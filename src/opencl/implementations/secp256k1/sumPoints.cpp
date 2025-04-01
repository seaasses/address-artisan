#include "src/opencl/definitions/secp256k1.h"
#include "src/opencl/headers/secp256k1/sumPoints.h"
#include "src/opencl/headers/modularOperations/modularSubtraction.h"
#include "src/opencl/headers/modularOperations/modularMultiplication.h"
#include "src/opencl/headers/modularOperations/modularExponentiation.h"


#pragma inline
const Point sumPoints(const Point a, const Point b) {
  // This implementation will only work for a + b with a != b and a != -b.
  // This implementation also assumes that none of the points are the point at
  // infinity. In our use case, it is practically impossible to have a == -b or
  // a == b or the points are the point at infinity. so we save these checks

  const UInt256 inverseXDiff =
      modularExponentiation(modularSubtraction(b.x, a.x), SECP256K1_P_MINUS_2);
  const UInt256 yDiff = modularSubtraction(b.y, a.y);

  const UInt256 lambda =
      modularMultiplicationUsingRussianPeasant(yDiff, inverseXDiff);

  const UInt256 xResult = modularSubtraction(
      modularSubtraction(
          modularMultiplicationUsingRussianPeasant(lambda, lambda), a.x),
      b.x);

  return (Point){
      .x = xResult,
      .y = modularSubtraction(modularMultiplicationUsingRussianPeasant(
                                  modularSubtraction(a.x, xResult), lambda),
                              a.y)};
}