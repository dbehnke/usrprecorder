mod usrp;
fn main() {
    let c = usrp::Config::load_from_file("./usrprecorder.toml").unwrap();
    usrp::rx_loop(&c)
}
