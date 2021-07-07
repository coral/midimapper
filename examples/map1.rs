use midimapper::{MIDIMapper, Mapping};

#[tokio::main]
pub async fn main() {
    let m = Mapping::load_from_file("mappings/launchcontrol.json").unwrap();
    //dbg!(m);

    let mut p = MIDIMapper::new(m).unwrap();
    let mut ch = p.get_channel();

    tokio::spawn(async move { p.run(0).await });

    loop {
        let m = ch.recv().await;

        dbg!(m);
    }
}
