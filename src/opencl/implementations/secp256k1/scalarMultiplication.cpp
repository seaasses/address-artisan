#include "src/opencl/headers/secp256k1/scalarMultiplication.h"
#include "src/opencl/headers/secp256k1/pointAddition.h"
#include "src/opencl/headers/secp256k1/doublePoint.h"
#include "src/opencl/headers/uint256/shiftRight.h"
#include "src/opencl/definitions/secp256k1.h"

#pragma inline
void scalarMultiplication(const Point *point, const UInt256 *scalar, Point *result)
{
    UInt256 scalarCopy = *scalar;
    Point pointCopy = *point;
    Point tmp;

    unsigned int infinity = 1;

    uint aa =0;
    while (scalarCopy.limbs[3] != 0 && aa < 70) {
        if (scalarCopy.limbs[3] & 1) {
            if (infinity) {
                *result = pointCopy;
                infinity = 0;
            } 
            else {
                pointAddition(&pointCopy, result, &tmp);
                *result = tmp;
            }
        }
        aa++;

        // doublePoint(&pointCopy, &tmp);
        pointCopy = tmp;

        // uint256ShiftRight(&scalarCopy, &scalarCopy);
    }

}