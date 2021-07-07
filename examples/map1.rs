use midimapper::{MIDIMapper, Mapping};
fn main() {
    let m = Mapping::load_from_file("mappings/launchcontrol.json").unwrap();
    //dbg!(m);

    let mut p = MIDIMapper::new(m).unwrap();
    let ch = p.get_channel();

    std::thread::spawn(move || p.run(0));

    loop {
        let m = ch.recv();

        dbg!(m);
    }
}
