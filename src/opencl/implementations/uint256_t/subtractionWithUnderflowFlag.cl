#pragma inline
void uint256SubtractionWithUnderflowFlag(const UInt256 *a, const UInt256 *b,
                                         UInt256 *result, bool *underflowFlag) {

  // The underflow flag is set basically if a < b
  // TODO: this function uses logical operations instead of bitwise operations.
  // see if this can be optimized for better warp performance

  ulong borrow;

  result->limbs[3] = a->limbs[3] - b->limbs[3];
  borrow = (a->limbs[3] < b->limbs[3]);

  result->limbs[2] = a->limbs[2] - b->limbs[2] - borrow;
  borrow = (a->limbs[2] < b->limbs[2] || (a->limbs[2] == b->limbs[2] && borrow));

  result->limbs[1] = a->limbs[1] - b->limbs[1] - borrow;
  borrow = (a->limbs[1] < b->limbs[1] || (a->limbs[1] == b->limbs[1] && borrow));

  result->limbs[0] = a->limbs[0] - b->limbs[0] - borrow;

  *underflowFlag =
      ((a->limbs[0] < b->limbs[0]) || ((a->limbs[0] == b->limbs[0]) && borrow));
}
