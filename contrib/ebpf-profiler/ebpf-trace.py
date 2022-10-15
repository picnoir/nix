#!/usr/bin/env nix-shell
#!nix-shell -i python3 -p python3 bcc --pure

from bcc import BPF, USDT
import ctypes as ct
import os
import sys
import io


if len(sys.argv) < 1:
    print("USAGE: ebpf-trace LIBRARY_PATH")
    exit()

debug = False

lib_path = sys.argv[1]

script_dir = os.path.dirname(os.path.realpath(__file__))
with open(os.path.join(script_dir, 'ebpf-trace.c')) as bpf_file:
    bpf_text = bpf_file.read()

def generate_bpf_function_for_probe_in(probe_name):
    return f"""
int usdt_trace_in_{probe_name} (struct pt_regs *ctx) {{
  uint64_t addr;
  struct trace_event trace_event = {{0}};

  trace_event.ts = bpf_ktime_get_ns();

  bpf_usdt_readarg(1, ctx, &trace_event.expr_id);

  bpf_usdt_readarg(2, ctx, &trace_event.line);

  bpf_usdt_readarg(3, ctx, &trace_event.column);

  bpf_usdt_readarg(4, ctx, &addr);
  bpf_probe_read_user_str(&trace_event.file, sizeof(char) * 128, (void *)addr);

  strcpy(trace_event.probe_name, "{probe_name}");

  events.perf_submit(ctx, &trace_event, sizeof(struct trace_event));
  return 0;
}};
"""

def generate_bpf_function_for_probe_out(probe_name):
    return f"""
int usdt_trace_out_{probe_name} (struct pt_regs *ctx) {{
  uint64_t addr;
  struct trace_event trace_event = {{0}};

  trace_event.ts = bpf_ktime_get_ns();

  bpf_usdt_readarg(1, ctx, &trace_event.expr_id);

  events.perf_submit(ctx, &trace_event, sizeof(struct trace_event));

  return 0;
}};
"""

probes = [ "top_level", "attrs", "let", "list", "var", "select",
           "lambda", "with", "if", "assert",
           "op_update", "call", "has_attr" ]
probes_in_out = []

probes_in = [f"{probe}__in" for probe in probes]

probes_out = [f"{probe}__out" for probe in probes] + [
    "call_throwned__out", "has_attr_failed__out", "op_update_empty1__out",
    "op_update_empty2__out", "select_short__out"
]

u = USDT(path=lib_path)
out_file = open('ebpf_trace.txt', 'wb', 100*(2**20))

def print_event(cpu, data, size):
    event = b["events"].event(data)
    out_file.write(f"{event.ts} {event.expr_id} {event.probe_name.decode('ascii', errors='ignore')} {event.line}:{event.column} {event.file.decode('ascii', errors='ignore')}\n".encode("utf-8"))

for probe in probes_in:
    bpf_text += generate_bpf_function_for_probe_in(probe)
    u.enable_probe(probe=probe, fn_name=f"usdt_trace_in_{probe}")

for probe in probes_out:
    bpf_text += generate_bpf_function_for_probe_out(probe)
    u.enable_probe(probe=probe, fn_name=f"usdt_trace_out_{probe}")

if debug:
    print(bpf_text)
    print(probes_in_out)

b = BPF(text=bpf_text, usdt_contexts=[u])

b["events"].open_perf_buffer(print_event, page_cnt=1024)

print("Ready", file=sys.stderr)
while True:
    try:
        b.perf_buffer_poll()
    except KeyboardInterrupt:
        out_file.close()
        exit()
