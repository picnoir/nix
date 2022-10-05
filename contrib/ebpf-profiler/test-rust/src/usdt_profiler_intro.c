#include <linux/string.h>

struct trace_event_in {
  u64 ts;
  u64 expr_id;
  u32 line;
  u32 column;
  char probe_name[25];
  char file[128];
};

struct trace_event_out {
  u64 ts;
  u64 expr_id;
};

BPF_PERF_OUTPUT(events);
