extern crate termion;
extern crate getopts;

use termion::{color, cursor, clear, style, terminal_size};
use termion::input::{TermRead, MouseTerminal};
use termion::event::{Key, Event, MouseButton, MouseEvent};
use termion::raw::IntoRawMode;
use std::io::{self, BufReader};
use std::io::prelude::*;
use std::iter::Iterator;
use std::{env, time, thread, f32};
use std::sync::{Arc, Mutex};
use std::collections::HashSet;
use std::fs::File;
use getopts::Options;

#[derive(PartialEq, Eq, Hash)]
struct Cell {
    x: i64,
    y: i64
}

#[derive(PartialEq, Clone)]
enum Action {
    Update,
    Pause,
    Quit,
    Nothing
}

#[derive(PartialEq, Clone)]
enum InputAction {
    MouseClick(u16, u16),
    KeyDown(char),
    KeyDownLeft,
    KeyDownRight,
    KeyDownUp,
    KeyDownDown,
    None
}

fn check_nearby(x: i64, y: i64, cells: &HashSet<Cell>) -> u32 {

    cells
        .iter()
        .fold(0,
              |sum, v|
              if v.x <= x+1  &&
                  v.x >= x-1 &&
                  v.y <= y+1 &&
                  v.y >= y-1 {
                  if v.y == y && v.x == x {
                      sum
                  } else {
                      sum + 1
                  }
              } else {
                  sum
              }
        )

}

fn process_input(queue: &Arc<Mutex<Vec<InputAction>>>,
                 cells: &mut HashSet<Cell>,
                 action: Action,
                 offset: &mut (i64, i64)) -> Action {

    let mut retval: Action = action;
    let mut q = queue.lock().unwrap();
    
    q.reverse();
    while q.len() != 0 {
        
        match q.pop().unwrap() {
            InputAction::KeyDownLeft => {
                offset.0 -= 1;
                retval = Action::Update;
            }
            InputAction::KeyDownRight => {
                offset.0 += 1;
                retval = Action::Update;
            }
            InputAction::KeyDownUp => {
                offset.1 -= 1;
                retval = Action::Update;
            }
            InputAction::KeyDownDown => {
                offset.1 += 1;
                retval = Action::Update;
            }
            InputAction::KeyDown(' ') => {
                retval = if retval == Action::Nothing {
                    Action::Pause
                } else {
                    Action::Nothing
                };
            },
            InputAction::KeyDown('q') => {
                retval = Action::Quit;
            },
            InputAction::MouseClick(x, y) => {
                let new_cell = Cell { x: (x as i64) - 1 + offset.0, y: (y as i64) - 1 + offset.1 };
                if !cells.remove(&new_cell) { 
                    cells.insert(new_cell);
                }
                retval = Action::Update;
            },
            _ => ()
        }
    }

    retval
    
}

fn step(cells: &mut HashSet<Cell>, cell_lives: &HashSet<u32>, new_cell: &HashSet<u32>) {

    let mut new_cells: HashSet<Cell> = HashSet::new(); 
    
    for cell in cells.iter() {

        let neighbors: u32 = check_nearby(cell.x, cell.y, &cells);
        
        if cell_lives.contains(&neighbors) {
            new_cells.insert(Cell { x: cell.x, y: cell.y });
        }

        let mut zipped: Vec<(i64, i64)> = Vec::new();
        
        for y in [cell.y-1, cell.y, cell.y+1].iter() {
            for x in [cell.x-1, cell.x, cell.x+1].iter() {
                zipped.push((*x, *y));
            }
        }

        for (x, y) in zipped {
            let neighbors: u32 = check_nearby(x, y, &cells);
            if new_cell.contains(&neighbors) {
                new_cells.insert(Cell { x: x, y: y });
            }
        }

    }

    cells.clear();
    for cell in new_cells {
        cells.insert(cell);
    }
    
}

fn display<W: Write>(stdout: &mut W,
                     cells: &HashSet<Cell>,
                     cells_old: &mut HashSet<Cell>,
                     offset: &(i64, i64),
                     gen: &u64,
                     bw: &mut u16,
                     bh: &mut u16) {
    
    let (w, h) = match terminal_size() {
        Ok(a) => a,
        Err(_) => (50, 30)
    };

    if *bw != w && *bh != h {
        print_background(stdout, &w, &h);
        *bw = w;
        *bh = h;
    }
    
    write!(stdout, "{}", cursor::Hide).unwrap();

    for cell in cells_old.iter() {
        write!(stdout,
               "{}{}.{}",
               cursor::Goto(cell.x as u16, cell.y as u16),
               color::Fg(color::Blue),
               style::Reset).unwrap();
    }

    cells_old.clear();
    
    for cell in cells.iter() {
        cells_old.insert(Cell { x: 1 - offset.0 + cell.x , y: 1 - offset.1 + cell.y });
        if cell.x < offset.0 + (w as i64)  &&
            cell.x >= offset.0             &&
            cell.y < offset.1 + (h as i64) &&
            cell.y >= offset.1 {
                let cellx = 1 - offset.0 + cell.x;
                let celly = 1 - offset.1 + cell.y;
                write!(stdout,
                       "{}{}{}@{}",
                       cursor::Goto(cellx as u16, celly as u16),
                       style::Bold,
                       color::Fg(color::Rgb(255, 163, 26)),
                       style::Reset).unwrap();
        }
    }

    write!(stdout, "{}{}{}Generation: {}{}", color::Fg(color::White), cursor::Goto(2, h), style::Bold, gen, style::Reset).unwrap();
    
    stdout.flush().unwrap();
    
}

fn print_background<W: Write>(stdout: &mut W, w: &u16, h: &u16) {

    for y in 0..*h {
        for x in 0..*w {
            write!(stdout,
                   "{}{}.{}",
                   cursor::Goto(1+x, 1+y),
                   color::Fg(color::Blue),
                   style::Reset).unwrap();
        }
    }

}

fn setup<W: Write>(stdout: &mut W,
                   queue: &Arc<Mutex<Vec<InputAction>>>,
                   cells: &HashSet<Cell>,
                   cells_old: &mut HashSet<Cell>,
                   bw: &mut u16,
                   bh: &mut u16) -> thread::JoinHandle<()> {

    let queue = queue.clone();
    let input_thread = thread::spawn(move || {
        
        'outer: loop {
            let stdin = io::stdin();
            for e in stdin.events() {
                let mut pushval: InputAction = InputAction::None;
                match e.unwrap() {
                    Event::Key(k) => {
                        match k {
                            Key::Left => pushval = InputAction::KeyDownLeft,
                            Key::Right => pushval = InputAction::KeyDownRight,
                            Key::Up => pushval = InputAction::KeyDownUp,
                            Key::Down => pushval = InputAction::KeyDownDown,
                            Key::Char(' ') => pushval = InputAction::KeyDown(' '),
                            Key::Char('q') => pushval = InputAction::KeyDown('q'),
                            _ => ()
                        }
                    },
                    Event::Mouse(m) => {
                        match m {
                            MouseEvent::Press(MouseButton::Left, x, y) => pushval = InputAction::MouseClick(x, y),
                            _ => ()
                            }
                    },
                    _ => ()
                }
                if pushval != InputAction::None {
                    let mut q = queue.lock().unwrap();
                    q.push(pushval.clone());
                    if pushval == InputAction::KeyDown('q') {
                         break 'outer;
                    }
                }
                break;
            }
        }
    });
    
    let gen: u64 = 0;
    let offset: (i64, i64) = (0, 0);
    
    display(stdout, cells, cells_old, &offset, &gen, bw, bh);

    input_thread
    
}

fn parse_for_hashset(opt: Option<String>, set: &mut HashSet<u32>) {

    match opt {
        Some(x) => {
            let mut buf: HashSet<u32> = HashSet::new();
            for v in x.trim().split(",") {
                match v.parse::<u32>() {
                    Ok(n) => buf.insert(n),
                    Err(_) => false
                };
            }
            if buf.len() > 0 {
                set.clear();
                for v in buf {
                    set.insert(v);
                }
            }
        }
        None => ()
    }
    
}

fn set_vars_from_opts(tick_time: &mut f32,
                      cells: &mut HashSet<Cell>,
                      new_cell: &mut HashSet<u32>,
                      cell_lives: &mut HashSet<u32>) -> bool {

    let args: Vec<String> = env::args().collect();
    let prog = args[0].clone();
    let mut opts = Options::new();
    
    opts.optflag("h", "help", "print this menu");
    opts.optopt("t",
                "tick-time",
                "changes the time a single tick takes (in seconds) [optional, default = 0.3]",
                "SECONDS");
    opts.optopt("n",
                "new-cell",
                "sets how many cells need to be around to make a new cell (takes a list of possible configurations) [optional, default = 3]",
                "a, b, ..x");
    opts.optopt("s",
                "survives",
                "sets how many neighbors a cell needs to survive (takes a list of possible configurations) [optional, default = 2, 3]",
                "a, b, ..x");
    opts.optopt("l",
                "load",
                "loads a life 1.06 compatable file [optional]",
                "FILE");

    let matches = match opts.parse(&args[1..]) {
        Ok(v) => v,
        Err(e) => {
            println!("{}", e.to_string());
            return true;
        }
    };
    
    if matches.opt_present("h") {
        let whitespace = String::from(" ");
        let usg = format!(
            "Usage: {} [options]\n\nIn game controlls:\n    q{}quit\n    space{}pause\n    mouse{}place cells",
            prog,
            whitespace.chars().cycle().take(19).collect::<String>(),
            whitespace.chars().cycle().take(15).collect::<String>(),
            whitespace.chars().cycle().take(15).collect::<String>()
        );
        println!("{}", opts.usage(&usg));
        return true;
    }

    match matches.opt_str("t") {
        Some(x) => {
            match x.trim().parse::<f32>() {
                Ok(v) => *tick_time = v,
                Err(e) => {
                    println!("{}", e.to_string());
                    return true;
                }
            }
        },
        None => ()
    }

    parse_for_hashset(matches.opt_str("n"), new_cell);

    parse_for_hashset(matches.opt_str("s"), cell_lives);

    match matches.opt_str("l") {
        Some(x) => {
            let fname = x.clone();
            match File::open(x) {
                Ok(f) => {
                    let reader = BufReader::new(f);
                    for line in reader.lines() {
                        match line {
                            Ok(l) => {
                                let vals: Vec<&str> = l
                                    .trim_right()
                                    .split(" ")
                                    .filter(|&v| match v.parse::<i64>() {
                                        Ok(_) => true,
                                        Err(_) => false
                                    })
                                    .collect();
                                if vals.len() >= 2 {
                                    let x: i64;
                                    let y: i64;
                                    match vals[0].parse::<i64>() {
                                        Ok(v) => x = v,
                                        Err(_) => continue
                                    }
                                    match vals[1].parse::<i64>() {
                                        Ok(v) => y = v,
                                        Err(_) => continue
                                    }
                                    cells.insert(Cell { x: x, y: y });
                                }
                            },
                            Err(_) => ()
                        }
                    }
                },
                Err(e) => {
                    println!("Couldn't open file \"{}\", error: {}", fname, e); 
                    return true;
                }
            }
        },
        None => ()
    }
    
    false
    
}

fn main() {
    
    let mut cells: HashSet<Cell> = HashSet::new();
    let mut cells_old: HashSet<Cell> = HashSet::new();
    let mut action: Action = Action::Pause;
    let mut last_action: Action;
    let mut offset: (i64, i64) = (0, 0);
    let mut generation: u64 = 0;
    let mut cell_lives: HashSet<u32> = [2, 3].iter().cloned().collect();
    let mut new_cell: HashSet<u32> = [3].iter().cloned().collect();
    let mut tick_timer = time::SystemTime::now();
    let mut tick_time: f32 = 0.3;
    let queue: Arc<Mutex<Vec<InputAction>>> = Arc::new(Mutex::new(Vec::new()));
    if set_vars_from_opts(&mut tick_time, &mut cells, &mut new_cell, &mut cell_lives) {
        return;
    }
    let mut stdout = MouseTerminal::from(io::stdout().into_raw_mode().unwrap());
    let mut bw: u16 = 0;
    let mut bh: u16 = 0;
    let input_thread = setup(&mut stdout, &queue, &cells, &mut cells_old, &mut bw, &mut bh);
    
    loop {
        last_action = action.clone();
        action = process_input(&queue, &mut cells, action, &mut offset);
        match action {
            Action::Update => {
                display(&mut stdout, &cells, &mut cells_old, &offset, &generation, &mut bw, &mut bh);
                action = last_action;
            }
            Action::Pause => continue,
            Action::Quit => break,
            Action::Nothing => {
                let dur = time::SystemTime::now().duration_since(tick_timer).unwrap();
                if (dur.as_secs() as f32) + (dur.subsec_nanos() as f32) / 1000000000_f32 > tick_time {
                    tick_timer = time::SystemTime::now();
                    step(&mut cells, &cell_lives, &new_cell);
                    generation += 1;
                    display(&mut stdout, &cells, &mut cells_old, &offset, &generation, &mut bw, &mut bh);
                }
            }
        }
    }
    
    let _ = input_thread.join();

    writeln!(stdout,
             "{}{}{}{}##############{}{}## {}Goodbye!{} ##{}{}##############{}{}",
             clear::All,
             cursor::Goto(1+bw/2-7,2),
             style::Bold,
             color::Fg(color::Yellow),
             cursor::Goto(1+bw/2-7,3),
             color::Fg(color::Green),
             color::Fg(color::White),
             color::Fg(color::Green),
             cursor::Goto(1+bw/2-7,4),
             color::Fg(color::Red),
             style::Reset,
             cursor::Show).unwrap();
    
}