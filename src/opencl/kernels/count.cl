__kernel void count(uint work_items, __global uchar *output) {

  const uint work_item_id = get_global_id(0);

  if (work_item_id >= work_items) {
    return;
  }

  uchar message[4] = {work_item_id >> 24, work_item_id >> 16, work_item_id >> 8,
                      work_item_id};

  uchar hashedMessage[32];

  sha256(message, 4, hashedMessage);
  if (hashedMessage[0] == 0) {
    output[work_item_id] = 1;
  } else {
    output[work_item_id] = 0;
  }
}