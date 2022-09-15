mod usrp;
fn main() {
    let c = usrp::Config::load_from_file("./usrprecorder.toml").unwrap();
    let ok = usrp::rx_loop(&c).unwrap();
    println!("{}", ok) // just a dummy to make compiler happy
}