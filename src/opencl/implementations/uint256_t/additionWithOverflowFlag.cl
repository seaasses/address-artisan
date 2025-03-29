#pragma inline
void uint256AdditionWithOverflowFlag(const UInt256 *a, const UInt256 *b,
                                     UInt256 *result, bool *overflowFlag) {

  // TODO: this function uses logical operations instead of bitwise operations.
  // see if this can be optimized for better warp performance

  ulong carry = 0;

  result->limbs[3] = a->limbs[3] + b->limbs[3];
  carry = result->limbs[3] < a->limbs[3];

  result->limbs[2] = a->limbs[2] + b->limbs[2] + carry;
  carry = (result->limbs[2] < a->limbs[2]) ||
          ((result->limbs[2] == a->limbs[2]) && carry);

  result->limbs[1] = a->limbs[1] + b->limbs[1] + carry;
  carry = (result->limbs[1] < a->limbs[1]) ||
          ((result->limbs[1] == a->limbs[1]) && carry);

  result->limbs[0] = a->limbs[0] + b->limbs[0] + carry;

  *overflowFlag = (result->limbs[0] < a->limbs[0]) ||
                  ((result->limbs[0] == a->limbs[0]) && carry);
}
