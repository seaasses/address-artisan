#define ROTR64(x, n) (((x) >> (n)) | ((x) << (64 - (n))))
#define maj(x, y, z) (((x) & (y)) | ((x) & (z)) | ((y) & (z)))
#define ch(x, y, z) (((x) & (y)) | ((~x) & (z)))
#define sha512SmallSigma0(x) (ROTR64(x, 1) ^ ROTR64(x, 8) ^ (x >> 7))
#define sha512SmallSigma1(x) (ROTR64(x, 19) ^ ROTR64(x, 61) ^ (x >> 6))
#define sha512BigSigma0(x) (ROTR64(x, 28) ^ ROTR64(x, 34) ^ ROTR64(x, 39))
#define sha512BigSigma1(x) (ROTR64(x, 14) ^ ROTR64(x, 18) ^ ROTR64(x, 41))

void sha512For165BytesMessage(uchar *message, uchar *hashedMessage) {

  const ulong k[80] = {
      0x428a2f98d728ae22, 0x7137449123ef65cd, 0xb5c0fbcfec4d3b2f,
      0xe9b5dba58189dbbc, 0x3956c25bf348b538, 0x59f111f1b605d019,
      0x923f82a4af194f9b, 0xab1c5ed5da6d8118, 0xd807aa98a3030242,
      0x12835b0145706fbe, 0x243185be4ee4b28c, 0x550c7dc3d5ffb4e2,
      0x72be5d74f27b896f, 0x80deb1fe3b1696b1, 0x9bdc06a725c71235,
      0xc19bf174cf692694, 0xe49b69c19ef14ad2, 0xefbe4786384f25e3,
      0x0fc19dc68b8cd5b5, 0x240ca1cc77ac9c65, 0x2de92c6f592b0275,
      0x4a7484aa6ea6e483, 0x5cb0a9dcbd41fbd4, 0x76f988da831153b5,
      0x983e5152ee66dfab, 0xa831c66d2db43210, 0xb00327c898fb213f,
      0xbf597fc7beef0ee4, 0xc6e00bf33da88fc2, 0xd5a79147930aa725,
      0x06ca6351e003826f, 0x142929670a0e6e70, 0x27b70a8546d22ffc,
      0x2e1b21385c26c926, 0x4d2c6dfc5ac42aed, 0x53380d139d95b3df,
      0x650a73548baf63de, 0x766a0abb3c77b2a8, 0x81c2c92e47edaee6,
      0x92722c851482353b, 0xa2bfe8a14cf10364, 0xa81a664bbc423001,
      0xc24b8b70d0f89791, 0xc76c51a30654be30, 0xd192e819d6ef5218,
      0xd69906245565a910, 0xf40e35855771202a, 0x106aa07032bbd1b8,
      0x19a4c116b8d2d0c8, 0x1e376c085141ab53, 0x2748774cdf8eeb99,
      0x34b0bcb5e19b48a8, 0x391c0cb3c5c95a63, 0x4ed8aa4ae3418acb,
      0x5b9cca4f7763e373, 0x682e6ff3d6b2b8a3, 0x748f82ee5defb2fc,
      0x78a5636f43172f60, 0x84c87814a1f0ab72, 0x8cc702081a6439ec,
      0x90befffa23631e28, 0xa4506cebde82bde9, 0xbef9a3f7b2c67915,
      0xc67178f2e372532b, 0xca273eceea26619c, 0xd186b8c721c0c207,
      0xeada7dd6cde0eb1e, 0xf57d4f7fee6ed178, 0x06f067aa72176fba,
      0x0a637dc5a2c898a6, 0x113f9804bef90dae, 0x1b710b35131c471b,
      0x28db77f523047d84, 0x32caab7b40c72493, 0x3c9ebe0a15c9bebc,
      0x431d67c49c100d4c, 0x4cc5d4becb3e42b6, 0x597f299cfc657e2a,
      0x5fcb6fab3ad6faec, 0x6c44198c4a475817};

  ulong hs[8] = {0x6a09e667f3bcc908, 0xbb67ae8584caa73b, 0x3c6ef372fe94f82b,
                 0xa54ff53a5f1d36f1, 0x510e527fade682d1, 0x9b05688c2b3e6c1f,
                 0x1f83d9abfb41bd6b, 0x5be0cd19137e2179};

  ulong letters[8] = {0x6a09e667f3bcc908, 0xbb67ae8584caa73b,
                      0x3c6ef372fe94f82b, 0xa54ff53a5f1d36f1,
                      0x510e527fade682d1, 0x9b05688c2b3e6c1f,
                      0x1f83d9abfb41bd6b, 0x5be0cd19137e2179};

  ulong t1, t2;

  // first message schedule
  ulong ws[80] = {

      (((ulong)message[0]) << 56) | (((ulong)message[1]) << 48) |
          (((ulong)message[2]) << 40) | (((ulong)message[3]) << 32) |
          (((ulong)message[4]) << 24) | (((ulong)message[5]) << 16) |
          (((ulong)message[6]) << 8) | message[7], // 0
      (((ulong)message[8]) << 56) | (((ulong)message[9]) << 48) |
          (((ulong)message[10]) << 40) | (((ulong)message[11]) << 32) |
          (((ulong)message[12]) << 24) | (((ulong)message[13]) << 16) |
          (((ulong)message[14]) << 8) | message[15], // 1
      (((ulong)message[16]) << 56) | (((ulong)message[17]) << 48) |
          (((ulong)message[18]) << 40) | (((ulong)message[19]) << 32) |
          (((ulong)message[20]) << 24) | (((ulong)message[21]) << 16) |
          (((ulong)message[22]) << 8) | message[23], // 2
      (((ulong)message[24]) << 56) | (((ulong)message[25]) << 48) |
          (((ulong)message[26]) << 40) | (((ulong)message[27]) << 32) |
          (((ulong)message[28]) << 24) | (((ulong)message[29]) << 16) |
          (((ulong)message[30]) << 8) | message[31], // 3
      (((ulong)message[32]) << 56) | (((ulong)message[33]) << 48) |
          (((ulong)message[34]) << 40) | (((ulong)message[35]) << 32) |
          (((ulong)message[36]) << 24) | (((ulong)message[37]) << 16) |
          (((ulong)message[38]) << 8) | message[39], // 4
      (((ulong)message[40]) << 56) | (((ulong)message[41]) << 48) |
          (((ulong)message[42]) << 40) | (((ulong)message[43]) << 32) |
          (((ulong)message[44]) << 24) | (((ulong)message[45]) << 16) |
          (((ulong)message[46]) << 8) | message[47], // 5
      (((ulong)message[48]) << 56) | (((ulong)message[49]) << 48) |
          (((ulong)message[50]) << 40) | (((ulong)message[51]) << 32) |
          (((ulong)message[52]) << 24) | (((ulong)message[53]) << 16) |
          (((ulong)message[54]) << 8) | message[55], // 6
      (((ulong)message[56]) << 56) | (((ulong)message[57]) << 48) |
          (((ulong)message[58]) << 40) | (((ulong)message[59]) << 32) |
          (((ulong)message[60]) << 24) | (((ulong)message[61]) << 16) |
          (((ulong)message[62]) << 8) | message[63], // 7
      (((ulong)message[64]) << 56) | (((ulong)message[65]) << 48) |
          (((ulong)message[66]) << 40) | (((ulong)message[67]) << 32) |
          (((ulong)message[68]) << 24) | (((ulong)message[69]) << 16) |
          (((ulong)message[70]) << 8) | message[71], // 8
      (((ulong)message[72]) << 56) | (((ulong)message[73]) << 48) |
          (((ulong)message[74]) << 40) | (((ulong)message[75]) << 32) |
          (((ulong)message[76]) << 24) | (((ulong)message[77]) << 16) |
          (((ulong)message[78]) << 8) | message[79], // 9
      (((ulong)message[80]) << 56) | (((ulong)message[81]) << 48) |
          (((ulong)message[82]) << 40) | (((ulong)message[83]) << 32) |
          (((ulong)message[84]) << 24) | (((ulong)message[85]) << 16) |
          (((ulong)message[86]) << 8) | message[87], // 10
      (((ulong)message[88]) << 56) | (((ulong)message[89]) << 48) |
          (((ulong)message[90]) << 40) | (((ulong)message[91]) << 32) |
          (((ulong)message[92]) << 24) | (((ulong)message[93]) << 16) |
          (((ulong)message[94]) << 8) | message[95], // 11
      (((ulong)message[96]) << 56) | (((ulong)message[97]) << 48) |
          (((ulong)message[98]) << 40) | (((ulong)message[99]) << 32) |
          (((ulong)message[100]) << 24) | (((ulong)message[101]) << 16) |
          (((ulong)message[102]) << 8) | message[103], // 12
      (((ulong)message[104]) << 56) | (((ulong)message[105]) << 48) |
          (((ulong)message[106]) << 40) | (((ulong)message[107]) << 32) |
          (((ulong)message[108]) << 24) | (((ulong)message[109]) << 16) |
          (((ulong)message[110]) << 8) | message[111], // 13
      (((ulong)message[112]) << 56) | (((ulong)message[113]) << 48) |
          (((ulong)message[114]) << 40) | (((ulong)message[115]) << 32) |
          (((ulong)message[116]) << 24) | (((ulong)message[117]) << 16) |
          (((ulong)message[118]) << 8) | message[119], // 14
      (((ulong)message[120]) << 56) | (((ulong)message[121]) << 48) |
          (((ulong)message[122]) << 40) | (((ulong)message[123]) << 32) |
          (((ulong)message[124]) << 24) | (((ulong)message[125]) << 16) |
          (((ulong)message[126]) << 8) |
          message[127], // 15 ---- final of the first block

  };

#pragma unroll
  for (short i = 16; i < 80; ++i) {
    ws[i] = sha512SmallSigma1(ws[i - 2]) + ws[i - 7] +
            sha512SmallSigma0(ws[i - 15]) + ws[i - 16];
  }
  //////////////////////////////////////////////////////////////
  // compression function

#pragma unroll
  for (short t = 0; t < 80; ++t) {
    t1 = letters[7] + sha512BigSigma1(letters[4]) +
         ch(letters[4], letters[5], letters[6]) + k[t] + ws[t];
    t2 = sha512BigSigma0(letters[0]) + maj(letters[0], letters[1], letters[2]);
    letters[7] = letters[6];
    letters[6] = letters[5];
    letters[5] = letters[4];
    letters[4] = letters[3] + t1;
    letters[3] = letters[2];
    letters[2] = letters[1];
    letters[1] = letters[0];
    letters[0] = t1 + t2;
  }

#pragma unroll
  for (uchar i = 0; i < 8; ++i) {
    letters[i] += hs[i];
    hs[i] = letters[i];
  }

  // second message schedule

  ws[0] = (((ulong)message[128]) << 56) | (((ulong)message[129]) << 48) |
          (((ulong)message[130]) << 40) | (((ulong)message[131]) << 32) |
          (((ulong)message[132]) << 24) | (((ulong)message[133]) << 16) |
          (((ulong)message[134]) << 8) | message[135];

  ws[1] = (((ulong)message[136]) << 56) | (((ulong)message[137]) << 48) |
          (((ulong)message[138]) << 40) | (((ulong)message[139]) << 32) |
          (((ulong)message[140]) << 24) | (((ulong)message[141]) << 16) |
          (((ulong)message[142]) << 8) | message[143];

  ws[2] = (((ulong)message[144]) << 56) | (((ulong)message[145]) << 48) |
          (((ulong)message[146]) << 40) | (((ulong)message[147]) << 32) |
          (((ulong)message[148]) << 24) | (((ulong)message[149]) << 16) |
          (((ulong)message[150]) << 8) | message[151];

  ws[3] = (((ulong)message[152]) << 56) | (((ulong)message[153]) << 48) |
          (((ulong)message[154]) << 40) | (((ulong)message[155]) << 32) |
          (((ulong)message[156]) << 24) | (((ulong)message[157]) << 16) |
          (((ulong)message[158]) << 8) | message[159];

  ws[4] = (((ulong)message[160]) << 56) | (((ulong)message[161]) << 48) |
          (((ulong)message[162]) << 40) | (((ulong)message[163]) << 32) |
          (((ulong)message[164]) << 24) | 0x800000;

  ws[5] = 0;
  ws[6] = 0;
  ws[7] = 0;
  ws[8] = 0;
  ws[9] = 0;
  ws[10] = 0;
  ws[11] = 0;
  ws[12] = 0;
  ws[13] = 0;
  ws[14] = 0;
  ws[15] = 0x0000000000000528;

#pragma unroll
  for (short i = 16; i < 80; ++i) {
    ws[i] = sha512SmallSigma1(ws[i - 2]) + ws[i - 7] +
            sha512SmallSigma0(ws[i - 15]) + ws[i - 16];
  }

  //////////////////////////////////////////////////////////////
  // second round of compression function

#pragma unroll
  for (short t = 0; t < 80; ++t) {
    t1 = letters[7] + sha512BigSigma1(letters[4]) +
         ch(letters[4], letters[5], letters[6]) + k[t] + ws[t];
    t2 = sha512BigSigma0(letters[0]) + maj(letters[0], letters[1], letters[2]);
    letters[7] = letters[6];
    letters[6] = letters[5];
    letters[5] = letters[4];
    letters[4] = letters[3] + t1;
    letters[3] = letters[2];
    letters[2] = letters[1];
    letters[1] = letters[0];
    letters[0] = t1 + t2;
  }

#pragma unroll
  for (uchar i = 0; i < 8; ++i) {
    letters[i] += hs[i];
    hashedMessage[i << 3] = letters[i] >> 56;
    hashedMessage[(i << 3) + 1] = letters[i] >> 48;
    hashedMessage[(i << 3) + 2] = letters[i] >> 40;
    hashedMessage[(i << 3) + 3] = letters[i] >> 32;
    hashedMessage[(i << 3) + 4] = letters[i] >> 24;
    hashedMessage[(i << 3) + 5] = letters[i] >> 16;
    hashedMessage[(i << 3) + 6] = letters[i] >> 8;
    hashedMessage[(i << 3) + 7] = letters[i];
  }
}