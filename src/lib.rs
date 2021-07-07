use crossbeam_channel::{select, unbounded};
use log::error;
use midir::{Ignore, MidiInput, MidiInputConnection};
use std::collections::HashMap;
use std::{error::Error, u8};

//Exports for lib
mod mapping;
pub use mapping::Mapping;

pub struct MIDIMapper {
    ch: Vec<crossbeam_channel::Sender<mapping::Feature>>,
    input: Option<MidiInputConnection<()>>,
    mapping: HashMap<String, FlatFeature>,
}

struct Message {
    channel: u8,
    message: u8,
    value: u8,
}

struct FlatFeature {
    kind: FeatureType,
    feature: mapping::Feature,
}
enum FeatureType {
    Button,
    Fader,
    Encoder,
}
enum FeatureResult {
    Button(String, u8),
    Fader(String, u8, u8),
    Encoder,
}

impl MIDIMapper {
    pub fn new(mapping: mapping::Mapping) -> Result<MIDIMapper, Box<dyn Error>> {
        Ok(MIDIMapper {
            ch: Vec::new(),
            input: None,
            mapping: MIDIMapper::map_to_map(mapping),
        })
    }

    pub fn get_channel(&mut self) -> crossbeam_channel::Receiver<mapping::Feature> {
        let (s, r) = crossbeam_channel::unbounded();
        self.ch.push(s);

        r
    }

    //THIS FUNCTION NAME HAHAHAHA
    fn map_to_map(mapping: mapping::Mapping) -> HashMap<String, FlatFeature> {
        let mut mapmap = HashMap::new();

        for button in mapping.buttons {
            mapmap.insert(
                MIDIMapper::keyhasher(button.channel, button.message),
                FlatFeature {
                    kind: FeatureType::Button,
                    feature: button,
                },
            );
        }

        for fader in mapping.faders {
            mapmap.insert(
                MIDIMapper::keyhasher(fader.channel, fader.message),
                FlatFeature {
                    kind: FeatureType::Fader,
                    feature: fader,
                },
            );
        }

        for encoder in mapping.encoders {
            mapmap.insert(
                MIDIMapper::keyhasher(encoder.channel, encoder.message),
                FlatFeature {
                    kind: FeatureType::Encoder,
                    feature: encoder,
                },
            );
        }

        mapmap
    }

    fn keyhasher(channel: u8, message: u8) -> String {
        channel.to_string() + "_" + &message.to_string()
    }

    pub fn run(&mut self, midi_port: usize) -> Result<(), Box<dyn Error>> {
        let (s, r) = crossbeam_channel::unbounded();

        let mut input = MidiInput::new("midimapper")?;
        input.ignore(Ignore::None);

        let in_ports = input.ports();
        let in_port = match in_ports.len() {
            0 => return Err("no input port found".into()),
            _ => &in_ports[midi_port],
        };

        let conn_in = input.connect(
            in_port,
            "midir-read-input",
            move |_, message, _| match s.send(Message {
                channel: message[1],
                message: message[0],
                value: message[2],
            }) {
                Err(e) => {
                    error!("midi channel err: {}", e);
                }
                _ => {}
            },
            (),
        )?;

        self.input = Some(conn_in);

        loop {
            let msg = match r.recv() {
                Ok(msg) => msg,
                Err(e) => {
                    error!("lol?");
                    continue;
                }
            };

            match self
                .mapping
                .get(&MIDIMapper::keyhasher(msg.channel, msg.message))
            {
                Some(f) => {}
                None => {}
            }
        }
    }
}
