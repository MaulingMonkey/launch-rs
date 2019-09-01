//! Definition of Launchpad devices.
//!
//! For now, only Launchpad Mark 2 devices are supported.

use midir;
use color::nearest_palette;
use std::sync::mpsc;

pub type Color = u8;

pub struct MidiEvent {
    pub timestamp: u32,
    pub message: MidiMessage,
}

pub struct MidiMessage {
    pub status: u8,
    pub data1: u8,
    pub data2: u8,
}

/// A launchpad device.
struct LaunchpadInternal {
    #[allow(dead_code)] input_port: midir::MidiInputConnection<()>, // Must be kept alive to receive messages
    output_port: midir::MidiOutputConnection,
    recv: mpsc::Receiver<MidiEvent>,
}

impl LaunchpadInternal {
    pub fn guess(expected_name: &str) -> Self {
        let input  = midir::MidiInput::new("launchpad").expect("Failed to open midir MidiInput Instance!");
        let output = midir::MidiOutput::new("launchpad").expect("Failed to open midir MidiOutput Instance!");
        Self::guess_from(input, output, expected_name)
    }

    /// Attempt to find the first Launchpad Mark 2 by scanning
    /// available MIDI ports with matching names. Bring your own
    /// PortMidi.
    pub fn guess_from(input: midir::MidiInput, output: midir::MidiOutput, expected_name: &str) -> Self {
        let mut input_port: Option<midir::MidiInputPort> = None;
        let mut output_port: Option<midir::MidiOutputPort> = None;

        for port in input.ports() {
            let name = input.port_name(&port).unwrap();
            if name.contains(expected_name) {
                input_port = Some(port);
                break;
            }
        }

        for port in output.ports() {
            let name = output.port_name(&port).unwrap();
            if name.contains(expected_name) {
                output_port = Some(port);
                break;
            }
        }

        let input_port  = input_port.expect("No Launchpad Input Found!");
        let output_port = output_port.expect("No Launchpad Output Found!");

        let (send, recv) = mpsc::channel();

        let input_port = input.connect(&input_port, "", move |time, msg, _user| {
            match msg {
                &[status, data1, data2] => {
                    let event = MidiEvent {
                        timestamp: time as u32,
                        message: MidiMessage {
                            status,
                            data1,
                            data2,
                        }
                    };
                    send.send(event).unwrap();
                },
                _ => {}, // Ignore
            }
        }, ()).expect("No Launchpad Mk2/Mini Input Found!");
        let output_port = output.connect(&output_port, "").expect("No Launchpad Mk2/Mini Output Found!");

        LaunchpadInternal {
            input_port,
            output_port,
            recv,
        }
    }

    pub fn send(&mut self, message: &[u8]) -> Result<(), midir::SendError> {
        self.output_port.send(message)
    }

    pub fn poll(&self) -> Option<Vec<MidiEvent>> {
        let events = self.recv.try_iter().collect::<Vec<MidiEvent>>();
        if events.is_empty() { None } else { Some(events) }
    }
}



/// A Launchpad Mk 1 / Mini / S device.
pub struct Launchpad(LaunchpadInternal);

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum GridMappingMode {
    /// X-Y Layout.  0xXY = X th column, Y th row, from top left (00 = top left, 77 = bottom right)
    XYLayout = 1,

    /// Drum rack layout.
    DrumRackLayout = 2,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Brightness {
    Low     = 125,
    Medium  = 126,
    High    = 127,
}

// https://d2xhy469pqj8rc.cloudfront.net/sites/default/files/novation/downloads/4080/launchpad-programmers-reference.pdf
impl Launchpad {
    pub fn guess() -> Self { Self(LaunchpadInternal::guess("Launchpad Mini")) }
    pub fn guess_from(input: midir::MidiInput, output: midir::MidiOutput) -> Self { Self(LaunchpadInternal::guess_from(input, output, "Launchpad Mini")) }
    pub fn reset(&mut self) { self.0.send(&[0xB0, 0x00, 0x00]).unwrap() }
    pub fn set_grid_mapping_mode(&mut self, mode: GridMappingMode) { self.0.send(&[0xB0, 0x00, unsafe { std::mem::transmute(mode) }]).unwrap() }
    pub fn ctrl_double_buffer_display_update_flash_copy(&mut self, display: bool, update: bool, flash: bool, copy: bool) {
        let display = (display   as u8) << 0;
        let update  = (update    as u8) << 2;
        let flash   = (flash     as u8) << 3;
        let copy    = (copy      as u8) << 4;
        let flags = 0b0100000 | display | update | flash | copy;
        self.0.send(&[0xB0, 0x00, flags]).unwrap()
    }

    pub fn light_all(&mut self, brightness: Brightness) { self.0.send(&[0xB0, 0x00, unsafe { std::mem::transmute(brightness) }]).unwrap() }
    // skipping "Set the duty cycle" for now.

    pub fn light_top(&mut self, column: u8, data: u8) {
        assert!(column < 8);
        assert!(data < 128);
        self.0.send(&[0xB0, 0x68 + column, data]).unwrap()
    }

    pub fn light_grid(&mut self, grid: &[u8], top: &[u8], right: &[u8]) {
        self.0.send(&[0xB0, 0x68 + 8, 0]).unwrap(); // Reset cursor position

        assert!(grid .len() == 8*8);
        assert!(top  .len() == 8);
        assert!(right.len() == 8);

        for ab in grid .chunks_exact(2) { self.0.send(&[0x92, ab[0], ab[1]]).unwrap(); }
        for ab in top  .chunks_exact(2) { self.0.send(&[0x92, ab[0], ab[1]]).unwrap(); }
        for ab in right.chunks_exact(2) { self.0.send(&[0x92, ab[0], ab[1]]).unwrap(); }
    }
}



/// A Launchpad Mark 2 Device.
pub struct LaunchpadMk2(LaunchpadInternal);

/// A single button/led
#[derive(Debug)]
pub struct ColorLed {
    pub color: Color,
    pub position: u8,
}

#[derive(Debug)]
/// A single column (0...8)
pub struct ColorColumn {
    pub color: Color,
    pub column: u8,
}

/// A single row (0...8)
#[derive(Debug)]
pub struct ColorRow {
    pub color: Color,
    pub row: u8,
}

pub const SCROLL_SLOWEST: &'static str = "\u{01}";
pub const SCROLL_SLOWER: &'static str = "\u{02}";
pub const SCROLL_SLOW: &'static str = "\u{03}";
pub const SCROLL_NORMAL: &'static str = "\u{04}";
pub const SCROLL_FAST: &'static str = "\u{05}";
pub const SCROLL_FASTER: &'static str = "\u{06}";
pub const SCROLL_FASTEST: &'static str = "\u{07}";

// https://d2xhy469pqj8rc.cloudfront.net/sites/default/files/novation/downloads/10529/launchpad-mk2-programmers-reference-guide-v1-02.pdf
impl LaunchpadMk2 {
    /// Attempt to find the first Launchpad Mark 2 by scanning
    /// available MIDI ports with matching names
    pub fn guess() -> Self { Self(LaunchpadInternal::guess("Launchpad MK2")) }

    /// Attempt to find the first Launchpad Mark 2 by scanning
    /// available MIDI ports with matching names. Bring your own
    /// PortMidi.
    pub fn guess_from(input: midir::MidiInput, output: midir::MidiOutput) -> Self {
        Self(LaunchpadInternal::guess_from(input, output, "Launchpad MK2"))
    }

    /// Set all LEDs to the same color
    pub fn light_all(&mut self, color: Color) {
        assert_color(color);
        // Message cannot be repeated.
        self.0.send(&[0xF0, 0x00, 0x20, 0x29, 0x02, 0x18, 0x0E, color, 0xF7]).unwrap();
    }

    /// Set a single LED to flash. Uses a smaller header than `flash_led` or
    /// `flash_leds` with a single item
    pub fn flash_single(&mut self, led: &ColorLed) {
        assert_position(led.position);
        assert_color(led.color);
        self.0.send(&[0x91, led.position, led.color]).unwrap();
    }

    /// Set a single LED to pulse. Uses a smaller header than `pulse_led` or
    /// `pulse_leds` with a single item
    pub fn pulse_single(&mut self, led: &ColorLed) {
        self.0.send(&[0x92, led.position, led.color]).unwrap();
    }

    /// Set a single LED to a palette color. Use `light_single` instead, its faster.
    pub fn light_led(&mut self, led: &ColorLed) {
        // F0h 00h 20h 29h 02h 18h 0Ah <LED> <Colour> F7h
        // Message can be repeated up to 80 times.
        self.light_leds(&[led])
    }

    /// Set LEDs to a certain color. Up to 80 LEDs can be set uniquely at once.
    pub fn light_leds(&mut self, leds: &[&ColorLed]) {
        assert!(leds.len() <= 80);
        for led in leds {
            assert_position(led.position);
            assert_color(led.color);
            self.0.send(&[0xF0, 0x00, 0x20, 0x29, 0x02, 0x18, 0x0A, led.position, led.color, 0xF7]).unwrap();
        }
    }

    /// Light a column of LEDs to the same color.
    pub fn light_column(&mut self, col: &ColorColumn) {
        // F0h 00h 20h 29h 02h 18h 0Ch <Column> <Colour> F7h
        // Message can be repeated up to 9 times.
        self.light_columns(&[col])
    }

    /// Light columns of LEDs to the same color. Each column may be set to a
    /// unique color. Up to 9 columns may be set at once.
    pub fn light_columns(&mut self, cols: &[&ColorColumn]) {
        assert!(cols.len() <= 9);
        for col in cols {
            assert_column(col.column);
            assert_color(col.color);
            self.0.send(&[0xF0, 0x00, 0x20, 0x29, 0x02, 0x18, 0x0C, col.column, col.color, 0xF7]).unwrap();
        }
    }

    /// Light a row of LEDs to the same color.
    pub fn light_row(&mut self, row: &ColorRow) {
        // F0h 00h 20h 29h 02h 18h 0Dh <Row> <Colour> F7h
        // Message can be repeated up to 9 times.
        self.light_rows(&[row])
    }

    /// Light rows of LEDs to the same color. Each row may be set to a
    /// unique color. Up to 9 rows may be set at once.
    pub fn light_rows(&mut self, rows: &[&ColorRow]) {
        assert!(rows.len() <= 9);
        for row in rows {
            assert_row(row.row);
            assert_color(row.color);
            self.0.send(&[0xF0, 0x00, 0x20, 0x29, 0x02, 0x18, 0x0D, row.row, row.color, 0xF7]).unwrap();
        }
    }

    /// Begin scrolling a message. The screen will be blanked, and the letters
    /// will be the same color. If the message is set to loop, it can be cancelled
    /// by sending an empty `scroll_text` command. String should only contain ASCII
    /// characters, or the byte value of 1-7 to set the speed (`\u{01}` to `\u{07}`)
    pub fn scroll_text(&mut self, color: Color, doloop: bool, text: &str) {
        // 14H <Color> <loop> <text...> F7h
        // Message cannot be repeated.
        assert_color(color);
        let mut msg: Vec<u8> = vec![0xF0, 0x00, 0x20, 0x29, 0x02, 0x18, 0x14, color, if doloop { 0x01 } else { 0x00 }];
        msg.extend_from_slice(text.as_bytes());
        msg.push(0xF7);

        self.0.send(&msg).unwrap();
    }

    /// Experimental. Try to set an LED by the color value in a "fast" way by
    /// by choosing the nearest neighbor palette color. This is faster because
    /// setting an LED using palette colors is a 3 byte message, whereas setting
    /// a specific RGB color takes at least 12 bytes.
    pub fn light_fuzzy_rgb(&mut self, position: u8, red: u8, green: u8, blue: u8) {
        self.light_led(&ColorLed {
            position: position,
            color: nearest_palette(red, green, blue),
        })
    }

    /// Retrieve pending MidiEvents
    pub fn poll(&self) -> Option<Vec<MidiEvent>> { self.0.poll() }
}

/// Make sure the position is valid
fn assert_position(pos: u8) {
    // Probably just make a Result
    if !match pos {
        11..=19 => true,
        21..=29 => true,
        31..=39 => true,
        41..=49 => true,
        51..=59 => true,
        61..=69 => true,
        71..=79 => true,
        81..=89 => true,
        104..=111 => true,
        _ => false,
    } {
        panic!("Bad Positon!")
    }
}

/// Make sure the palette color is valid
fn assert_color(clr: u8) {
    if clr > 127 {
        panic!("Bad Color!");
    }
}

/// Make sure the column is valid
fn assert_column(col: u8) {
    if col > 8 {
        panic!("Bad Column");
    }
}

/// Make sure the row is valid
fn assert_row(row: u8) {
    if row > 8 {
        panic!("Bad Row");
    }
}

//////////////////////////////////////////////////////////////////
// TODO ITEMS
//////////////////////////////////////////////////////////////////


// pub fn device_inquiry() {
//     // (240,126,127, 6, 1, 247)
// }

// #[derive(Debug)]
// enum Layout {
//     Session,
//     User_1,
//     User_2,
//     Ableton_Reserved,
//     Volume,
//     Pan
// }

// // pg6
// pub fn set_layout(layout: Layout) -> Result<()> {
//     use Layout::*;
//     let i = match layout {
//         Session => 0u8,
//         User_1 => 1u8,
//         User_2 => 2u8,
//         Ableton_Reserved => 3u8,
//         Volume => 4u8,
//         Pan => 5u8,
//     };
//     unimplemented!()
// }


// pub fn flash_led(led: &ColorLed) {
//     // F0h 00h 20h 29h 02h 18h 23h <LED> <Colour> F7h
//     // Message can be repeated up to 80 times.
//     flash_leds(&[led])
// }

// pub fn flash_leds(leds: &[&ColorLed]) {

// }

// pub fn pulse_led(led: &ColorLed) {
//     // F0h 00h 20h 29h 02h 18h 28h <LED> <Colour> F7h
//     // Message can be repeated up to 80 times.
//     pulse_leds(&[led])
// }

// pub fn pulse_leds(leds: &[&ColorLed]) {

// }

// pub fn light_rgb(light: u8, red: u8, green: u8, blue: u8) {
//     // F0h 00h 20h 29h 02h 18h 0Bh <LED>, <Red> <Green> <Blue> F7h
//     // Message can be repeated up to 80 times.
// }

// pub fn start_vol_fader() {

// }

// pub fn start_pan_fader() {

// }

// pub fn start_fader(layout: u8, number: u8, color: Color, value: u8)
// {

// }

// pub fn scroll_text(text: &[u8], loop: bool, color: Color) {

// }
