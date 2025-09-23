#define ULONG_OPERATIONS_H

#define UINT64_MULTIPLICATION(a, b, high, low)                              \
    do                                                                      \
    {                                                                       \
        ulong a_low = (a) & 0xFFFFFFFFUL;                                   \
        ulong a_high = (a) >> 32;                                           \
        ulong b_low = (b) & 0xFFFFFFFFUL;                                   \
        ulong b_high = (b) >> 32;                                           \
                                                                            \
        ulong ll = a_low * b_low;                                           \
        ulong lh = a_low * b_high;                                          \
        ulong hl = a_high * b_low;                                          \
        ulong hh = a_high * b_high;                                         \
                                                                            \
        ulong mid = (ll >> 32) + (lh & 0xFFFFFFFFUL) + (hl & 0xFFFFFFFFUL); \
        ulong carry = (mid >> 32);                                          \
                                                                            \
        (low) = (mid << 32) | (ll & 0xFFFFFFFFUL);                          \
        (high) = hh + (lh >> 32) + (hl >> 32) + carry;                      \
    } while (0)

#define UINT64_SUM_WITH_OVERFLOW_FLAG(a, b, result, overflow_flag) \
    do                                                             \
    {                                                              \
        (result) = (a) + (b);                                      \
        (overflow_flag) = ((result) < (b));                        \
    } while (0)
