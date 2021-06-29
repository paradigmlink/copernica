use {
    beard::beard,
    anyhow::{Result},
    //copernica_common::{PrivateIdentityInterface},
    //rand::Rng,
    std::{
        fs::{OpenOptions, File},
        io::{self, Write, BufReader, BufRead, Error},
        //path::Path,
        str::FromStr,
        io::prelude::*,
        process::{Command, Stdio},
    },
    console_engine::{ pixel, Color, KeyCode },
    copernica_monitor::{LogEntry, DotEntry, GraphVizPlainExt},
    term_size,
};
fn main() -> Result<()> {
    let file = File::open("copernica.log")?;
    let lines = io::BufReader::new(file).lines();
    let mut queue: Vec<LogEntry> = vec![];
    for line in lines {
        if let Ok(ip) = line {
            match LogEntry::from_str(&ip) {
                Ok(log_entry) => queue.push(log_entry),
                Err(_e) => continue,
            }
        }
    }
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open("monitor.dot")?;
    beard! {
        file,
        "
digraph G {
  rankdir=TB;
  size=\"8.3,11.7!\";
  margin=0;
  node [shape=record];\n"
  for entry in ( queue ) {
      { DotEntry(entry) }
  }
"
}
"
    };
    let output = Command::new("dot").arg("-Tplain-ext").arg("monitor.dot").output()?;
    if !output.status.success() {
        println!("Command executed with failing error code");
    }
    let out = String::from_utf8(output.stdout)?;
    for l in out.lines() {
        println!("{:?}", l);
        println!("{:?}", GraphVizPlainExt::from_str(l));
    }

    if let Some((width, height)) = term_size::dimensions() {
        //let width = engine.get_width();
        //let height = engine.get_width();
    }
    let (width, height) = match term_size::dimensions() {
        Some((width, height)) => (width as u32, height as u32),
        None => (20 as u32, 10 as u32),
    };

    // initializes a screen of 20x10 characters with a target of 3 frame per second
    // coordinates will range from [0,0] to [19,9]
    let mut engine = console_engine::ConsoleEngine::init(width, height, 3)?;
    let value = 14;
    loop {
        engine.wait_frame(); // wait for next frame + capture inputs
        engine.clear_screen(); // reset the screen
        engine.check_resize();

        engine.line(10, 10, 30, 30, pixel::pxl('●')); // draw a line of '#' from [0,0] to [19,9]
        engine.print(0, 4, format!("width: {}, height: {}", width, height).as_str()); // prints some value at [0,4]

        engine.set_pxl(4, 0, pixel::pxl_fg('O', Color::Cyan)); // write a majestic cyan 'O' at [4,0]

        if engine.is_key_pressed(KeyCode::Char('q')) { // if the user presses 'q' :
            break; // exits app
        }

        engine.draw(); // draw the screen
    }

    Ok(())
}
