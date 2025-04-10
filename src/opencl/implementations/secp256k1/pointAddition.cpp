#include "src/opencl/headers/secp256k1/pointAddition.h"
#include "src/opencl/headers/modularOperations/modularMultiplication.h"
#include "src/opencl/headers/modularOperations/modularSubtraction.h"
#include "src/opencl/headers/modularOperations/modularDouble.h"
#include "src/opencl/headers/modularOperations/modularExponentiation.h"
#include "src/opencl/definitions/secp256k1.h"


#pragma inline
void pointAddition(const Point *a, const Point *b, Point *result)
{
    // JUST FOR a != b, a != -b and both points are not the point at infinity

    UInt256 tmp = SECP256K1_P_MINUS_2;

    // LAMBDA = (y2 - y1) / (x2 - x1)

    modularSubtraction(&b->y, &a->y, &result->y); // tmp = p-2, result.y = y2 - y1
    modularSubtraction(&b->x, &a->x, &result->x); // tmp = p-2, result.x = x2 - x1 and result.y = y2 - y1
    modularExponentiation(&result->x, &tmp, &result->x); // tmp = p-2, result.x = (x2 - x1)^(-1) and result.y = y2 - y1
    modularMultiplicationUsingRussianPeasant(&result->x, &result->y, &tmp); // tmp = LAMBDA, result.x = (x2 - x1)^(-1) and result.y = y2 - y1

    // X_RESULT = LAMBDA^2 - x1 - x2

    modularMultiplicationUsingRussianPeasant(&tmp, &tmp, &result->x); // tmp = LAMBDA, result.x = LAMBDA^2, result.y = y2 - y1
    modularSubtraction(&result->x, &a->x, &result->x); // tmp = LAMBDA, result.x = LAMBDA^2 - x1, result.y = y2 - y1 
    modularSubtraction(&result->x, &b->x, &result->x); // tmp = LAMBDA, result.x = X_RESULT, result.y = y2 - y1 

    // // Y_RESULT = LAMBDA * (x1 ​− ​X_RESULT) − y1

    modularSubtraction(&a->x, &result->x, &result->y); // tmp = LAMBDA, result.x = X_RESULT, result.y = x1 - X_RESULT 
    modularMultiplicationUsingRussianPeasant(&result->y, &tmp, &result->y); // tmp = LAMBDA, result.x = X_RESULT, result.y = LAMBDA * (x1 - X_RESULT)
    modularSubtraction(&result->y, &a->y, &result->y); // tmp = LAMBDA, result.x = X_RESULT, result.y = Y_RESULT :D

}