use chrono::{ DateTime, Utc };
use byteorder::{ BigEndian, LittleEndian, ReadBytesExt };
use log::{ info, warn };
use std::path::Path;
use serde_derive::Deserialize;
use std::fs::File;
use std::io::prelude::*;
use std::io::Cursor;
use std::net::UdpSocket;
use std::str;
use toml;

#[derive(Clone, Deserialize)]
pub struct Config {
    group: String,
    rx_host_port: String,
    audio_write_path: String,
}

impl Config {
    pub fn _new(group: &str, rx_host_port: &str, audio_write_path: &str) -> Config {
        Config {
            group: String::from(group),
            rx_host_port: String::from(rx_host_port),
            audio_write_path: String::from(audio_write_path),
        }
    }

    pub fn load_from_file(filepath: &str) -> Result<Config, Box<dyn std::error::Error>> {
        let mut file = File::open(filepath)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        let c: Config = toml::from_str(&contents)?;
        return Ok(c);
    }
}

struct Transmission {
    group: String, // name of a group this transmission belongs to
    start_time: DateTime<Utc>, //StartTime time.Time    // start of Transmission
    end_time: DateTime<Utc>, //EndTime   time.Time    // end of Transmission
    callsign: String, // who's transmitting
    audio: Vec<u8>, // raw audio payload
    config: Config,
}

impl Transmission {
    pub fn new(group: String, callsign: String, config: &Config) -> Transmission {
        Transmission {
            group: group,
            start_time: Utc::now(),
            end_time: Utc::now(),
            callsign: callsign,
            audio: vec![],
            config: config.to_owned(),
        }
    }

    fn write_transmission(&mut self) -> Result<bool, Box<dyn std::error::Error>> {
        let timestamp = Utc::now().timestamp();
        let filename = String::from(
            format!("{}-{}-{}.pcm", self.config.group, timestamp, self.callsign)
        );
        let p = Path::new(&self.config.audio_write_path).join(filename);
        let mut output = File::create(&p)?;
        output.write_all(&self.audio)?;
        info!("wrote {:?} ({} bytes)", &p, self.audio.len());
        Ok(true)
    }

    pub fn end_transmission(&mut self) {
        self.end_time = Utc::now();
        let elapsed = self.end_time.signed_duration_since(self.start_time);
        info!(
            "END TX: {} {} lasted {} seconds and used {} bytes",
            self.group,
            self.callsign,
            elapsed.num_seconds(),
            self.audio.len()
        );
        match self.write_transmission() {
            Ok(_ok) => (),
            Err(e) => { warn!("unable to write tranmission to disk: {}", e) }
        }
    }
}

// USRP Packet Types
const USRP_TYPE_VOICE: u32 = 0;
const _USRP_TYPE_DTMF: u32 = 1;
const USRP_TYPE_TEXT: u32 = 2;
const USRP_TYPE_PING: u32 = 3;
const _USRP_TYPE_TLV: u32 = 4;
const _USRP_TYPE_VOICE_ADPCM: u32 = 5;
const _USRP_TYPE_VOICE_ULAW: u32 = 6;

// TLV
const TLV_TAG_SET_INFO: u8 = 8;

fn find_call(b: &[u8]) -> String {
    match str::from_utf8(&b) {
        Ok(s) => s.trim().to_string(),
        Err(_) => {
            return String::from("UNKNOWN");
        }
    }
}

pub fn rx_loop(c: &Config) -> Result<bool, Box<dyn std::error::Error>> {
    info!("Loaded Config for {}", c.group);
    info!("Opening receive UDP on {}", c.rx_host_port);
    let socket = UdpSocket::bind(&c.rx_host_port)?;
    let mut buf = [0; 1024];
    let mut t = Transmission::new(String::from(&c.group), String::from("UNKNOWN"), c);
    loop {
        let (number_of_bytes, _src_addr) = socket.recv_from(&mut buf)?;

        if number_of_bytes < 4 {
            warn!("bytes read is too small < 4: {} received", number_of_bytes);
            continue;
        }
        let filled_buf = &mut buf[..number_of_bytes];
        let first_four = str::from_utf8(&filled_buf[0..4])?;
        if first_four != "USRP" {
            warn!("Not a USRP packet");
        }
        let mut rdr = Cursor::new(&filled_buf[4..]);
        let _seq = rdr.read_u32::<BigEndian>()?; //4..8
        let _memory = rdr.read_u32::<LittleEndian>()?; //8..12
        let keyup: u32 = rdr.read_u32::<BigEndian>()?; //12..16
        let _talkgroup: u32 = rdr.read_u32::<BigEndian>()?; //16..20
        let usrp_type: u32 = rdr.read_u32::<BigEndian>()?; //20..24
        let audio = &mut filled_buf[32..];

        match usrp_type {
            USRP_TYPE_VOICE => {
                if keyup == 0 {
                    t.end_transmission();
                } else {
                    t.audio.extend(audio.iter()); //capture the audio
                }
            }
            USRP_TYPE_TEXT => {
                if audio[0] == TLV_TAG_SET_INFO {
                    info!("Info TAG, BEGIN TX");
                    let callsign = find_call(&audio[14..50]);
                    t = Transmission::new(String::from(&c.group), String::from(callsign), c);
                }
            }
            USRP_TYPE_PING => {}
            _ => {}
        }
    }
}