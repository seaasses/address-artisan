#pragma inline
const uint256_t uint256_t_modularAddition(const uint256_t a,
                                          const uint256_t b) {

  const uint256_t result = uint256_t_addition(a, b);

  return modulus(result);
}