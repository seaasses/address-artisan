#include "src/opencl/headers/secp256k1/doublePoint.h"
#include "src/opencl/headers/modularOperations/modularMultiplication.h"
#include "src/opencl/headers/modularOperations/modularAddition.h"
#include "src/opencl/headers/modularOperations/modularShiftLeft.h"
#include "src/opencl/headers/modularOperations/modularExponentiation.h"
#include "src/opencl/headers/modularOperations/modularSubtraction.h"
#include "src/opencl/definitions/secp256k1P.h"

#pragma inline
const Point doublePoint(const Point p)
{
    // In teory, this ans sumPoints could/should be merged together
    // but in practice, when sumPoints is called, probably the points are not the
    // same. By doing this, we can save a check in sumPoints to see if the points
    // are the same and simplify the code.
    // This will be used primarily to double a point and implement Point * scalar
    // multiplication

    const UInt256 xSquared = modularMultiplicationUsingRussianPeasant(p.x, p.x);

    const UInt256 lambda = modularMultiplicationUsingRussianPeasant(
        modularAddition(modularShiftLeft(xSquared), xSquared),
        modularExponentiation(modularShiftLeft(p.y), SECP256K1_P_MINUS_2));

    const UInt256 xResult = modularSubtraction(
        modularMultiplicationUsingRussianPeasant(lambda, lambda),
        modularShiftLeft(p.x));

    return (Point){
        .x = xResult,
        .y = modularSubtraction(modularMultiplicationUsingRussianPeasant(
                                    modularSubtraction(p.x, xResult), lambda),
                                p.y)};
}