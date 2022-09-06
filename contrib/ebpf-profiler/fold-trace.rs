use std::io;
use std::io::BufRead;

#[derive (PartialEq, Debug)]
enum Direction {
    In,
    Out
}

#[derive (PartialEq, Debug)]
struct Event {
    ts: u64,
    line: u64,
    id: u64,
    probe_name: String,
    probe_direction: Direction,
    line_col: String,
    filename: String
}

fn main ()  {
    let mut buf = String::new();
    let mut stdin_h = io::stdin().lock();
    let mut done = false;
    let mut stack: Vec<Event> = Vec::new();
    let mut line_nb: u64 = 1;
    while !done {
        match stdin_h.read_line(&mut buf) {
            Ok(0) => done = true,
            Ok(_) => process_line(&buf, &mut stack, &line_nb),
            Err(_err) => {
                panic!("Error while reading from stdin.");
            }
        }
        line_nb += 1;
        buf.clear();
    }
    eprintln!("NB lines: {}", line_nb);
}

fn print_stack_names (stack: &Vec<Event>) -> String {
    let mut names_str = String::new();
    for event in stack {
        names_str.push_str(format!(";{}:{}:{}", &event.probe_name, &event.filename, &event.line_col).as_str());
    };
    names_str
}

fn process_line(line: &str, stack: &mut Vec<Event>, line_nb: &u64) {
    let elems: Vec<&str> = line.split(' ').collect();
    let probe_elems: Vec<&str> = elems[2].split("__").collect();
    let probe_direction = match probe_elems[1] {
        "in" => Direction::In,
        "out" => Direction::Out,
        x => panic!("Unknown probe direction {}", x)
    };
    let mut filename = String::from(elems[4]);
    filename.truncate(filename.len() - 1);
    let event = Event {
        ts: elems[0].parse().expect(format!("Cannot parse timestamp for line {}", line_nb).as_str()),
        id: elems[1].parse().expect(format!("Cannot parse probe direction for line {}", line_nb).as_str()),
        line: line_nb.clone(),
        probe_name: String::from(probe_elems[0]),
        probe_direction,
        line_col: String::from(elems[3]),
        filename
    };

    if event.probe_direction == Direction::In {
        stack.push(event);
    } else {
        let in_event = stack.pop().expect("Error: cannot pop stack, we lack a in event.");
        if !same_frame(&event, &in_event) {
            eprintln!("Weird trace!! We found a unmatched out event for");
            eprintln!("{:?}", in_event);
            eprintln!("{:?}", event);
            stack.push(in_event);
            panic!();
        }
        let dur = event.ts - in_event.ts;
        println!("{} {}", print_stack_names(&stack), dur);
    }
}

fn same_frame(a: &Event, b: &Event) -> bool {
    a.id == b.id
}
