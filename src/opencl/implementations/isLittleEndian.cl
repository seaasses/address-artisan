__inline uchar isLittleEndian() {
  ushort tmp = 1;
  uchar *tmpBytes = (uchar *)&tmp;
  return tmpBytes[0];
}
