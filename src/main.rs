mod usrp;
fn main() {
    log4rs::init_file("log4rs.yml", Default::default()).unwrap();
    let c = usrp::Config::load_from_file("./usrprecorder.toml").unwrap();
    let ok = usrp::rx_loop(&c).unwrap();
    println!("{}", ok) // just a dummy to make compiler happy
}