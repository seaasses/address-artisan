#include "src/opencl/headers/modularOperations/modularSubtraction.h"
#include "src/opencl/headers/uint256/addition.h"
#include "src/opencl/definitions/secp256k1.h"
#include "src/opencl/headers/uint256/subtractionWithUnderflowFlag.h"

#pragma inline
void modularSubtraction(const UInt256 *a, const UInt256 *b, UInt256 *result)
{ // inplace safe
  unsigned int underflowFlag;
  UInt256 tmp;

  uint256SubtractionWithUnderflowFlag(a, b, &tmp, &underflowFlag);

  unsigned long maskToSum = -((unsigned long)underflowFlag);
  const UInt256 toSum = (UInt256){.limbs = {
                                SECP256K1_P_0 & maskToSum,
                                SECP256K1_P_1 & maskToSum,
                                SECP256K1_P_2 & maskToSum,
                                SECP256K1_P_3 & maskToSum,
                            }};

  uint256Addition(&tmp, &toSum, result);
}