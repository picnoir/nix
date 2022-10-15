use bcc::{BccError, BPFBuilder, USDTContext};
use bcc::perf_event::PerfMapBuilder;

use core::sync::atomic::{AtomicBool, Ordering};

use std::sync::Arc;
use std::ptr;

use std::env;
use std::fs::File;
use std::io::Write;

#[repr(C)]
struct trace_event {
    ts: u32,
    expr_id: u32,
    line: u32,
    column: u32,
    probe_name: [u8; 25],
    file: [u8; 128]
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

  strcpy(trace_event.probe_name, \"{probe_name}\");

  events.perf_submit(ctx, &trace_event, sizeof(struct trace_event));
  return 0;
}}
", probe_name = usdt_name)
}

fn parse_trace_event(x: &[u8]) -> trace_event {
    unsafe { ptr::read_unaligned(x.as_ptr() as *const trace_event) }
}

fn create_usdt_code_out(usdt_name: &str) -> String {
    format!("
int usdt_trace_out_{probe_name} (struct pt_regs *ctx) {{
  uint64_t addr;
  struct trace_event trace_event = {{0}};

  trace_event.ts = bpf_ktime_get_ns();

  bpf_usdt_readarg(1, ctx, &trace_event.expr_id);

  bpf_usdt_readarg(2, ctx, &trace_event.line);

  bpf_usdt_readarg(3, ctx, &trace_event.column);

  bpf_usdt_readarg(4, ctx, &addr);
  bpf_probe_read_user_str(&trace_event.file, sizeof(char) * 128, (void *)addr);

  strcpy(trace_event.probe_name, \"{probe_name}\");

  events.perf_submit(ctx, &trace_event, sizeof(struct trace_event));
  return 0;
}}
", probe_name = usdt_name)
}

fn profiler_data_callback () -> Box<dyn FnMut(&[u8]) + Send> {
    let f = File::create("/tmp/out.txt").unwrap();
    Box::new(move |x| {
        let data = parse_trace_event(x);
        f.write_all(data.ts);
        //println!("{}", data.ts);
    })
}


fn do_main(runnable: Arc<AtomicBool>, path: &str) -> Result<(), BccError> {
    let cpus = bcc::cpuonline::get()?.len() as u32;
    let mut u = USDTContext::from_binary_path(path)?;

    let probes =
        vec! [
            "top_level".to_string(),
            "attrs".to_string(),
            "let".to_string(),
            "list".to_string(),
            "var".to_string(),
            "select".to_string(),
            "lambda".to_string(),
            "with".to_string(),
            "if".to_string(),
            "assert".to_string(),
            "op_update".to_string(),
            "call".to_string(),
            "has_attr".to_string()
        ];

    let mut probes_in = Vec::with_capacity(13);
    for probe in &probes {
        probes_in.push(format!("{}__in", probe));
     };

    let mut probes_out = Vec::with_capacity(20);
    for probe in &probes {
        probes_out.push(format!("{}__out", probe));
    }
    probes_out.append(&mut vec![
        "call_throwned__out".to_string(),
        "has_attr_failed__out".to_string(),
        "op_update_empty1__out".to_string(),
        "op_update_empty2__out".to_string(),
        "select_short__out".to_string()
    ]);
    let mut code = format!(
        "{}\n{}\n",
        format!("#define NUM_CPU {}", cpus),
        include_str!("usdt_profiler_intro.c").to_string()
    );
    for probe in &probes_in {
        code.push_str(&create_usdt_code_in(probe));
        u.enable_probe(probe, format!("usdt_trace_in_{}", probe))?;
    }
//  for probe in &probes_out {
//      code.push_str(&create_usdt_code_out(probe));
//      u.enable_probe(probe, format!("usdt_trace_out_{}", probe))?;
//  }

    println!("Starting with code:\n{}", &code);
    let bpf = BPFBuilder::new(&code)?
        .add_usdt_context(u)?
        .build()?;
    let table = bpf.table("events")?;
    let mut perf_map = PerfMapBuilder::new(table, profiler_data_callback).page_count(128).build()?;
    println!("{}", probes_in.len() + probes_out.len());
    println!("Ready");
    while runnable.load(Ordering::SeqCst) {
        perf_map.poll(200);
    };
    return Ok(())
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let path = &args[1];
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
