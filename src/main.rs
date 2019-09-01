extern crate launchpad;
extern crate clap;
extern crate midir;

use launchpad::*;

use std::thread;
use std::process;
use std::time::Duration;
use std::num::Wrapping;

mod cli;

fn main() {
    // initialize the PortMidi context.

    let args = cli::build_args();
    let inpt = args.get_matches();
    if inpt.occurrences_of("list") > 0 {
        list();
    }

    run_mk1();
}

fn list() -> ! {
    let input  = midir::MidiInput::new("launch-rs").unwrap();
    let output = midir::MidiOutput::new("launch-rs").unwrap();

    println!("inputs:");
    for port in input.ports() {
        println!("  {}", input.port_name(&port).unwrap());
    }

    println!("outputs:");
    for port in output.ports() {
        println!("  {}", output.port_name(&port).unwrap());
    }

    process::exit(0);
}

fn run_mk1() {
    println!("Please enjoy!");
    let mut lpad = Launchpad::guess();

    // Buffers
    let mut top   = [0,0,0,0, 0,0,0,0];
    let mut right = [0,0,0,0, 0,0,0,0];
    let mut grid = Vec::new();
    grid.resize(8*8, 0);

    loop {
        println!("Clear screen...");
        lpad.reset();
        thread::sleep(Duration::from_millis(500));

        println!("Tweak brightness!");
        for _ in 0..2 {
            for &level in &[Brightness::Low, Brightness::Medium, Brightness::High] {
                lpad.light_all(level);
                thread::sleep(Duration::from_millis(500));
            }
        }

        println!("Grid colors!");
        for color in 0..=64 {
            // XXX: bits 2/3 are copy/clear flags, 7 should be 0.
            let color = Wrapping(color);
            let mut local_color = color;
            for c in &mut top   { *c = local_color.0 & 0x7F; local_color += Wrapping(1); }
            for c in &mut grid  { *c = local_color.0 & 0x7F; local_color += Wrapping(1); }
            for c in &mut right { *c = local_color.0 & 0x7F; local_color += Wrapping(1); }
            lpad.light_grid(&grid[..], &top[..], &right[..]);
            thread::sleep(Duration::from_millis(16));
        }

        println!("Cycle colors!");
        for _ in 0..3 {
            for &color in &[
            //    0ggccrr
                0b0000000,
                0b0000001,
                0b0000010,
                0b0000011,
                0b0010011,
                0b0100011,
                0b0110011,
                0b0110010,
                0b0110001,
                0b0110000,
                0b0100000,
                0b0010000,
            ] {
                for c in &mut top   { *c = color; }
                for c in &mut grid  { *c = color; }
                for c in &mut right { *c = color; }
                lpad.light_grid(&grid[..], &top[..], &right[..]);
                //thread::sleep(Duration::from_millis(33));
                thread::sleep(Duration::from_millis(300));
            }
        }
    }
}

fn run_mk2() {
    println!("Please enjoy!");
    let timeout = Duration::from_millis(1);
    let mut lpad = LaunchpadMk2::guess();

    println!("Clear screen...");
    lpad.light_all(0);

    // println!("Fuzzy!");
    // for r in 0..255 {
    //     if r % 10 != 0 {
    //         continue;
    //     }
    //     for g in 0..255 {
    //         if g % 10 != 0 {
    //             continue;
    //         }
    //         for b in 0..255 {
    //             if b % 10 != 0 {
    //                 continue;
    //             }
    //             lpad.light_fuzzy_rgb(11, r, g, b);
    //         }
    //     }
    // }
    // thread::sleep(Duration::from_millis(500));
    // println!("Fuzzy!");

    println!("Columns on!");
    for i in 0..9 {
        lpad.light_column(&ColorColumn {
            column: i,
            color: 5,
        });
        thread::sleep(Duration::from_millis(25));
    }

    thread::sleep(Duration::from_millis(500));

    println!("Columns off!");
    for i in 0..9 {
        lpad.light_row(&ColorRow { row: i, color: 0 });
        thread::sleep(Duration::from_millis(25));
    }

    thread::sleep(Duration::from_millis(500));

    println!("Whole panel colors...");
    for color in vec![18, 54, 13, 104, 0] {
        lpad.light_all(color);
        thread::sleep(Duration::from_millis(1000));
    }

    // Playground

    println!("Light rows in a silly way");
    for row in 1..9 {
        for column in 1..9 {
            let x = 10 * row + column;
            lpad.light_led(&ColorLed {
                position: x,
                color: 88,
            });
            thread::sleep(Duration::from_millis(1));
        }
        thread::sleep(Duration::from_millis(16));
    }

    thread::sleep(Duration::from_millis(500));

    println!("Bottom Right to Top Left");
    lpad.light_leds(&vec![&ColorLed {position: 11, color: 41,},
                          &ColorLed {position: 22, color: 41,},
                          &ColorLed {position: 33, color: 41,},
                          &ColorLed {position: 44, color: 41,},
                          &ColorLed {position: 55, color: 41,},
                          &ColorLed {position: 66, color: 41,},
                          &ColorLed {position: 77, color: 41,},
                          &ColorLed {position: 88, color: 41,},]);

    thread::sleep(Duration::from_millis(500));

    println!("Bottom Left to Top Right");
    lpad.light_leds(&vec![&ColorLed {position: 81, color: 5,},
                          &ColorLed {position: 72, color: 5,},
                          &ColorLed {position: 63, color: 5,},
                          &ColorLed {position: 54, color: 5,},
                          &ColorLed {position: 45, color: 5,},
                          &ColorLed {position: 36, color: 5,},
                          &ColorLed {position: 27, color: 5,},
                          &ColorLed {position: 18, color: 5,},]);

    thread::sleep(Duration::from_millis(500));

    println!("Right controls on");
    lpad.light_leds(&vec![&ColorLed {position: 19, color: 3,},
                          &ColorLed {position: 29, color: 3,},
                          &ColorLed {position: 39, color: 3,},
                          &ColorLed {position: 49, color: 3,},
                          &ColorLed {position: 59, color: 3,},
                          &ColorLed {position: 69, color: 3,},
                          &ColorLed {position: 79, color: 3,},
                          &ColorLed {position: 89, color: 3,},]);

    thread::sleep(Duration::from_millis(500));

    println!("Top controls on");
    lpad.light_leds(&vec![&ColorLed {position: 104, color: 4,},
                          &ColorLed {position: 105, color: 4,},
                          &ColorLed {position: 106, color: 4,},
                          &ColorLed {position: 107, color: 4,},
                          &ColorLed {position: 108, color: 4,},
                          &ColorLed {position: 109, color: 4,},
                          &ColorLed {position: 110, color: 4,},
                          &ColorLed {position: 111, color: 4,},]);


    thread::sleep(Duration::from_millis(500));
    println!("Blank screen");
    lpad.light_all(0);

    println!("Scroll Text");
    lpad.scroll_text(27, false, &format!("{}Your {}Turn!", SCROLL_SLOWER, SCROLL_FASTER));

    let mut foo = 0;

    println!("Blinky/Pulsy playground!");
    loop {
        if let Some(events) = lpad.poll() {
            // println!("{:?}", event);
            for press in events {
                if press.message.data2 == 127 {
                    foo += 1;
                    foo %= 128;
                    let led = ColorLed {
                        color: foo,
                        position: press.message.data1,
                    };
                    if 0x1 == (foo & 0x1) {
                        lpad.pulse_single(&led);
                    } else {
                        lpad.flash_single(&led);
                    }
                }
            }
        }

        // there is no blocking receive method in PortMidi, therefore
        // we have to sleep some time to prevent a busy-wait loop
        thread::sleep(timeout);
    }

}
