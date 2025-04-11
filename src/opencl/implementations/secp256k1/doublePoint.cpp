#include "src/opencl/headers/secp256k1/doublePoint.h"
#include "src/opencl/headers/modularOperations/modularMultiplication.h"
#include "src/opencl/headers/modularOperations/modularSubtraction.h"
#include "src/opencl/headers/modularOperations/modularDouble.h"
#include "src/opencl/headers/modularOperations/modularExponentiation.h"
#include "src/opencl/definitions/secp256k1.h"


#pragma inline
void doublePoint(const Point *p, Point *result)
{
    // In teory, this and sumPoints could/should be merged together
    // but in practice, when sumPoints is called, probably the points are not the
    // same. By doing this, we can save a check in sumPoints to see if the points
    // are the same and simplify the code and be more efficient.

    // This function will be used primarily to double a point and implement Point * scalar
    // multiplication

    UInt256 tmp = SECP256K1_P_MINUS_2;

    // LAMBDA = (3x^2 + a) * (2y)^(-1)

    modularMultiplicationUsingRussianPeasant(&p->x, &p->x, &result->y); // result.y = x^2
    modularDouble(&result->y, &result->x); // result.x = 2x^2 and result.y = x^2
    modularAddition(&result->y, &result->x, &result->y); // result.x = 2x^2 and result.y = 3x^2

    modularDouble(&p->y, &result->x); // result.x = 2y and result.y = 3x^2 

    // TODO: This uses Fermat's little theorem to compute the inverse. Create a function to this and test it vs the 
    // Extended Euclidean Algorithm

    // add comment about montgomery multiplication
    modularExponentiation(&result->x, &tmp, &result->x); // result.x = (2y)^(-1) and result.y = 3x^2

    modularMultiplicationUsingRussianPeasant(&result->y, &result->x, &result->y); // result.x = (2y)^(-1) and result.y = LAMBDA

    // X_RESULT = LAMBDA^2 - 2x

    modularDouble(&p->x, &result->x); // result.x = 2x and result.y = LAMBDA
    modularMultiplicationUsingRussianPeasant(&result->y, &result->y, &tmp); // result.x = 2x, result.y = LAMBDA and tmp = LAMBDA^2 
    modularSubtraction(&tmp, &result->x, &result->x); // result.x = X_RESULT, result.y = LAMBDA and tmp = LAMBDA^2

    // Y_RESULT = LAMBDA * (x - X_RESULT) - y

    modularSubtraction(&p->x, &result->x, &tmp); // result.x = X_RESULT, tmp = x - X_RESULT and result.y = LAMBDA
    modularMultiplicationUsingRussianPeasant(&result->y, &tmp, &result->y); // result.x = X_RESULT, tmp = x - X_RESULT and result.y = lambda * (x - X_RESULT)
    modularSubtraction(&result->y, &p->y, &result->y); // result.x = X_RESULT, result.y = Y_RESULT :D

}