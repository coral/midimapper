use midimapper::Mapping;
fn main() {
    let m = Mapping::load_from_file("mappings/launchcontrol.json");
    dbg!(m);
}
