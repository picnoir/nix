#include <linux/string.h>

struct trace_event {
  u64 ts;
  u64 expr_id;
  u32 line;
  u32 column;
  char probe_name[25];
  char file[128];
};

BPF_PERF_OUTPUT(events);
