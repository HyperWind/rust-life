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

fn count_neighbors(x: i64, y: i64, cells: &HashSet<Cell>) -> u32 {

    let mut neighbor_counter: u32 = 0;
    
    for iy in y-1..y+2 {
        for ix in x-1..x+2 {
            if ix != x || iy != y {
                let buffer_cell =  Cell { x: ix, y: iy };
                neighbor_counter = if cells.contains(&buffer_cell) { 
                    neighbor_counter + 1
                } else {
                    neighbor_counter
                };
            }
        }
    }

    neighbor_counter

}

fn process_input(queue: &Arc<Mutex<Vec<InputAction>>>,
                 cells: &mut HashSet<Cell>,
                 action: Action,
                 offset: &mut (i64, i64)) -> Action {

    let mut retval: Action = action.clone();
    let mut q = queue.lock().unwrap();
    
    q.reverse();
    while !q.is_empty() {
        
        retval = match q.pop().unwrap() {
            InputAction::KeyDownLeft => {
                offset.0 -= 1;
                Action::Update
            }
            InputAction::KeyDownRight => {
                offset.0 += 1;
                Action::Update
            }
            InputAction::KeyDownUp => {
                offset.1 -= 1;
                Action::Update
            }
            InputAction::KeyDownDown => {
                offset.1 += 1;
                Action::Update
            }
            InputAction::KeyDown(' ') => {
                if retval == Action::Nothing {
                    Action::Pause
                } else {
                    Action::Nothing
                }
            },
            InputAction::KeyDown('q') => {
                Action::Quit
            },
            InputAction::MouseClick(x, y) => {
                let new_cell = Cell { x: (x as i64) - 1 + offset.0, y: (y as i64) - 1 + offset.1 };
                if !cells.remove(&new_cell) { 
                    cells.insert(new_cell);
                }
                Action::Update
            },
            _ => action.clone()
        }
    }

    retval
    
}

fn step(cells: &mut HashSet<Cell>, cell_lives: &HashSet<u32>, new_cell: &HashSet<u32>) {

    let mut new_cells: HashSet<Cell> = HashSet::new(); 
    
    for cell in cells.iter() {

        let neighbors: u32 = count_neighbors(cell.x, cell.y, cells);
        
        if cell_lives.contains(&neighbors) {
            new_cells.insert(Cell { x: cell.x, y: cell.y });
        }

        for y in cell.y-1..cell.y+2 {
            for x in cell.x-1..cell.x+2 {
                let neighbors: u32 = count_neighbors(x, y, cells);
                if new_cell.contains(&neighbors) {
                    new_cells.insert(Cell { x: x, y: y });
                }
            }
        }

    }

    *cells = new_cells.drain().collect();
}

fn display<W: Write>(stdout: &mut W,
                     cells: &HashSet<Cell>,
                     cells_old: &mut HashSet<Cell>,
                     offset: &(i64, i64),
                     gen: &u64,
                     bw: &mut u16,
                     bh: &mut u16) {
    
    let (w, h) = terminal_size().unwrap_or_else(|_| (50,30));

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

    write!(stdout,
           "{}{}{}Generation: {}{}",
           color::Fg(color::White),
           cursor::Goto(2, h),
           style::Bold, gen,
           style::Reset).unwrap();
    
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
                let pushval = match e.unwrap() {
                    Event::Key(Key::Left) => InputAction::KeyDownLeft,
                    Event::Key(Key::Right) => InputAction::KeyDownRight,
                    Event::Key(Key::Up) => InputAction::KeyDownUp,
                    Event::Key(Key::Down) => InputAction::KeyDownDown,
                    Event::Key(Key::Char(' ')) => InputAction::KeyDown(' '),
                    Event::Key(Key::Char('q')) => InputAction::KeyDown('q'),
                    Event::Mouse(MouseEvent::Press(MouseButton::Left, x, y)) => InputAction::MouseClick(x, y),
                    _ => InputAction::None
                };
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

    if let Some(x) = opt {
        let mut buf: HashSet<u32> = HashSet::new();
        for v in x.trim().split(',') {
            match v.parse::<u32>() {
                Ok(n) => buf.insert(n),
                Err(_) => false
            };
        }
        if !buf.is_empty() {
            *set = buf.drain().collect();
        }
    }
    
}

fn set_vars_from_opts(tick_time: &mut f32,
                      cells: &mut HashSet<Cell>,
                      new_cell: &mut HashSet<u32>,
                      cell_lives: &mut HashSet<u32>) -> Result<(),()> {

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

    let matches = try!(opts.parse(&args[1..]).map_err(|e| {
        println!("{}", e.to_string());
    }));
    
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
        return Err(());
    }

    if let Some(x) = matches.opt_str("t") {
        *tick_time = try!(x.trim().parse::<f32>().map_err(|e| {
            println!("{}", e.to_string());
        }));
    }

    parse_for_hashset(matches.opt_str("n"), new_cell);

    parse_for_hashset(matches.opt_str("s"), cell_lives);

    if let Some(x) = matches.opt_str("l") {
        let f = try!(File::open(x.clone()).map_err(|e| {
                println!("Couldn't open file \"{}\", error: {}", x, e); 
        }));
        let reader = BufReader::new(f);
        for line in reader.lines() {
            if let Ok(l) = line {
                let vals: Vec<&str> = l
                    .trim_right()
                    .split(' ')
                    .filter(|&v| v.parse::<i64>().is_ok())
                    .collect();
                if vals.len() >= 2 {
                    match (vals[0].parse::<i64>(), vals[1].parse::<i64>()) {
                        (Ok(x), Ok(y)) => {
                            cells.insert(Cell { x: x, y: y });
                        },
                        _ => continue
                    }
                }
            }
        }
    }
    
    Ok(())
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
    if let Err(()) = set_vars_from_opts(&mut tick_time, &mut cells, &mut new_cell, &mut cell_lives) {
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
