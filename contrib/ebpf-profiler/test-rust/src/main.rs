use env;

use bcc::BPF;
use bcc::{trace_parse, trace_read, BccError, Kprobe};

use core::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

#[repr(C)]
struct trace_event {
    ts: u64,
    expr_id: u64,
    line: u32,
    column: u32,
    probe_name: [u8, 25],
    file: [u8, 128]
}

fn create_usdt_code_in(usdt_name: &str) -> String {
    format!("
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

  events.perf_submit(ctx, &trace_event, sizeof(struct trace_event_in));
  return 0;
}};
", probe_name = usdt_name)
}

fn parse_trace_event(x: &[u8]) -> trace_event {
    unsafe { ptr::read_unaligned(x.as_ptr as *const trace_event) }
}

fn create_usdt_code_out(usdt_name: &str) -> String {
    format!("
int usdt_trace_out_{probe_name} (struct pt_regs *ctx) {{
  struct trace_event trace_event = {{0}};

  trace_event.ts = bpf_ktime_get_ns();

  uint64_t addr;
  bpf_usdt_readarg(1, ctx, &trace_event.expr_id);

  events.perf_submit(ctx, &trace_event, sizeof(struct trace_event_out));

  return 0;", probe_name = usdt_name)
}

fn do_main(runnable: Arc<AtomicBool>, Path: String) -> Result<(), BccError> {
    let cpus = bcc::cpuonline::get()?.len() as u32;
    let u = USDTContext::from_binary_path(&path)?;

    let probes = vec! [ "top_level", "attrs", "let", "list", "var",
                        "select", "lambda", "with", "if", "assert",
                        "op_update", "call", "has_attr" ];

    let probes_in = Vec::with_capacity(13);
    for probe in probes {
        probe_in.push(format!("{}_in", probe));
     };

    let probes_out = Vec::with_capacity(20);
    for probe in probes {
        probes_out.push(format!({}_out, probe));
    }
    probes_out.append(vec![
        "call_throwned__out",
        "has_attr_failed__out",
        "op_update_empty1__out",
        "op_update_empty2__out",
        "select_short__out" ]
    );
    let code = format!(
        "{}\n{}\n",
        format!("#define NUM_CPU {}", cpus),
        include_str!("usdt_profiler_intro.c").to_string()
    );
    for probe in probes_in {
        code.push(create_usdt_code_in(probe));
        u.enable_probe(probe, format!("usdt_trace_in_{}", probe));
    }
    for probe in probe_out {
        code.push(create_usdt_code_out(probe))
        u.enable_probe(probe, format!("usdt_trace_out_{}", probe));
    }

    let mut bpf = BPF::new(&code)?;
    while runnable.load(Ordering::SeqCst) {
        let r = trace_read();
        match r {
            Ok(s) => {
                let item = trace_parse(s);
                println!("{:?}", item);
            }
            Err(e) => println!("{:?}", e),
        }
    }
    Ok(())
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let path = args[1];
    let runnable = Arc::new(AtomicBool::new(true));
    let r = runnable.clone();
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })
    .expect("Failed to set handler for SIGINT / SIGTERM");

    if let Err(x) = do_main(runnable, path) {
        eprintln!("Error: {}", x);
        std::process::exit(1);
    }
}
