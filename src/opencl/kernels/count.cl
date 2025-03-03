uint return6();

__kernel void count(uint work_items, __global uchar *output) {

  const uint work_item_id = get_global_id(0);

  if (work_item_id >= work_items) {
    return;
  }

  output[work_item_id] = return6();
}