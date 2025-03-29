#pragma inline
const Point sumPoints(const Point a, const Point b) {
  // This implementation will only work for a + b with a != b and a != -b
  // In our use case, it is practically impossible to have a == -b or a == b
  // so we save these checks

  const UInt256 inverseXDiff =
      modularExponentiation(modularSubtraction(b.x, a.x), SECP256K1_P_MINUS_2);
  const UInt256 yDiff = modularSubtraction(b.y, a.y);

  const UInt256 lambda =
      modularMultiplicationUsingRussianPeasant(yDiff, inverseXDiff);

  const UInt256 xResult = modularSubtraction(
      modularSubtraction(
          modularMultiplicationUsingRussianPeasant(lambda, lambda), a.x),
      b.x);

  const UInt256 yResult =
      modularSubtraction(modularMultiplicationUsingRussianPeasant(
                             modularSubtraction(a.x, xResult), lambda),
                         a.y);
  const Point result = {.x = xResult, .y = yResult};
  return result;
}