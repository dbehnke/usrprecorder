use byteorder::{ BigEndian, LittleEndian, ReadBytesExt };
use log::{ info, warn };
use serde_derive::Deserialize;
use std::fs::File;
use std::io::Cursor;
use std::io::{ prelude::* };
use std::net::UdpSocket;
use std::str;
use toml;

#[derive(Deserialize)]
pub struct Config {
    group: String,
    rx_host_port: String,
}

impl Config {
    pub fn _new(group: &str, rx_host_port: &str) -> Config {
        Config { group: String::from(group), rx_host_port: String::from(rx_host_port) }
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
    //StartTime time.Time    // start of Transmission
    //EndTime   time.Time    // end of Transmission
    Callsign: String, // who's transmitting
    Audio: Vec<u8>, // raw audio payload
}

/*
func NewTransmission(group, callsign string) Transmission {
	log.Printf("BEGIN TX - %s - %s", group, callsign)
	return Transmission{
		Group:     group,
		Callsign:  callsign,
		StartTime: time.Now(),
	}
}

func (t *Transmission) WriteAudioToFile(OutputPath string) (int64, error) {
	timestampID := time.Now().UTC().Unix()
	filename := fmt.Sprintf("%s-%d-%s.pcm", t.Group, timestampID, t.Callsign)
	err := os.WriteFile(OutputPath+string(os.PathSeparator)+filename, t.Audio.Bytes(), 0600)
	if err != nil {
		return timestampID, err
	}
	return timestampID, nil
}

func (t *Transmission) EndTransmission() {
	t.EndTime = time.Now()
	timeElapsed := t.EndTime.Sub(t.StartTime)
	log.Printf("END TX - %s - %s - %v - %d bytes", t.Group, t.Callsign, timeElapsed, t.Audio.Len())
	if timeElapsed.Seconds() < 5 || timeElapsed.Seconds() > 200 {
		log.Println("Not writing to disk .. elapsed time is too short or too long")
		return
	}
	id, err := t.WriteAudioToFile("./audio")
	if err != nil {
		log.Printf("didn't write audio: %v", err)
		return
	}
	log.Printf("Wrote %d to disk", id)
}
*/

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

pub fn rx_loop(c: &Config) -> Result<bool, Box<dyn std::error::Error>> {
    info!("Loaded Config for {}", c.group);
    info!("Opening receive UDP on {}", c.rx_host_port);
    let socket = UdpSocket::bind(&c.rx_host_port)?;
    let mut buf = [0; 1024];
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
                    info!("END TX");
                } else {
                    //capture the audio
                }
            }
            USRP_TYPE_TEXT => {
                if audio[0] == TLV_TAG_SET_INFO {
                    info!("Info TAG, BEGIN TX");
                    //callsign = findCall(audio[14:50])
                }
            }
            USRP_TYPE_PING => {}
            _ => {}
        }
    }
}