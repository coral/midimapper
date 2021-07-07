use anyhow::{anyhow, Result};
use log::error;
use midir::{Ignore, MidiInput, MidiInputConnection};
use std::collections::HashMap;
use std::u8;
use tokio::sync::mpsc;

//Exports for lib
mod mapping;
pub use mapping::Mapping;

pub struct MIDIMapper {
    ch: Vec<mpsc::Sender<FeatureResult>>,
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

#[derive(Debug, Clone)]
pub enum FeatureResult {
    Button(String),
    Value(String, u8),
}

impl MIDIMapper {
    pub fn new(mapping: mapping::Mapping) -> Result<MIDIMapper> {
        Ok(MIDIMapper {
            ch: Vec::new(),
            input: None,
            mapping: MIDIMapper::map_to_map(mapping),
        })
    }

    pub fn get_channel(&mut self) -> mpsc::Receiver<FeatureResult> {
        let (s, r) = mpsc::channel(32);
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

    pub async fn run(&mut self, midi_port: usize) -> Result<()> {
        let (s, mut r) = mpsc::unbounded_channel();

        let mut input = MidiInput::new("midimapper")?;
        input.ignore(Ignore::None);

        let in_ports = input.ports();
        let in_port = match in_ports.len() {
            0 => return Err(anyhow!("No input port found")),
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
        );

        let conn_in = match conn_in {
            Ok(v) => v,
            Err(e) => {
                return Err(anyhow!("ugh: {}", e));
            }
        };

        self.input = Some(conn_in);

        loop {
            let msg = match r.recv().await {
                Some(v) => v,
                None => continue,
            };

            match self
                .mapping
                .get(&MIDIMapper::keyhasher(msg.channel, msg.message))
            {
                Some(f) => {
                    let event = match f.kind {
                        FeatureType::Button => FeatureResult::Button(f.feature.name.clone()),
                        FeatureType::Encoder => {
                            FeatureResult::Value(f.feature.name.clone(), msg.value)
                        }
                        FeatureType::Fader => {
                            FeatureResult::Value(f.feature.name.clone(), msg.value)
                        }
                    };

                    for recv in &self.ch {
                        recv.try_send(event.clone());
                    }
                }
                None => {}
            }
        }
    }
}
