//! Клавиатура ZET Blade Pro (0416:A0F8) через службу wkb-service из Blade Pro Software.
//! HID держит сама служба (от SYSTEM); говорим с ней так же, как wkbapp.exe 1.2.4.2 (снято перехватом):
//! TCP 127.0.0.1:57887, заголовок 32 байта + JSON, всё AES-256-ECB.

use aes::Aes256;
use aes::cipher::{BlockDecrypt, BlockEncrypt, KeyInit, generic_array::GenericArray};
use base64::{Engine, engine::general_purpose::STANDARD as B64};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    io::{self, Read, Write},
    net::TcpStream,
    time::Duration,
};

const ADDR: &str = "127.0.0.1:57887";
const MAGIC: [u8; 4] = [0xaf, 0x3e, 0x5d, 0xa2];
/// Шифрует сообщения до рукопожатия
const K_LINK: &str = "d00aa0c9ddec921e260dd6c2432c40ca5f53a7e7bae37d25221cf9dc9dc67bb4";
/// Шифрует вызов службы и выданный ею сеансовый ключ
const K_AUTH: &str = "931379953e9c8e25c030faab5d11dfdd83b4cd214f825f46338c9a5b7999a2ff";

const HELLO: u16 = 0x0b;
const GET_EFFECT: u16 = 0x16;
const SET_EFFECT: u16 = 0x17;
const LIST_DEVICES: u16 = 0x1f;

pub const MODE_STATIC: u8 = 1;

/// Эффект клавиатуры как у службы: режим 1 статичный … 13 своя раскраска;
/// цвет 0xRRGGBB, в старшем байте флаги (бит 24 — радуга, бит 26 — случайный)
#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub struct Effect {
    pub mode: u8,
    pub brightness: u8,
    pub speed: u8,
    pub direction: u8,
    pub palette: Vec<u32>,
}

/// Сколько ступеней у ползунков режима и цветов в его палитре
#[derive(Serialize, PartialEq, Debug)]
pub struct ModeInfo {
    pub index: u8,
    pub speeds: u8,
    pub brights: u8,
    pub dirs: u8,
    pub colors: u8,
}

pub struct Zet {
    sock: TcpStream,
    aes: Aes256,
}

impl Zet {
    pub fn connect() -> io::Result<Self> {
        let sock = TcpStream::connect_timeout(&ADDR.parse().unwrap(), Duration::from_millis(300))?;
        sock.set_read_timeout(Some(Duration::from_secs(2)))?;
        let mut z = Self { sock, aes: cipher(K_LINK.as_bytes()) };
        let r = z.call(HELLO, &json!({}))?;
        let auth = cipher(K_AUTH.as_bytes());
        let answer = ecb(&auth, b64(&r["data"])?, true);
        let r = z.call(HELLO, &json!({ "data": B64.encode(answer) }))?;
        z.aes = Aes256::new(GenericArray::from_slice(&ecb(&auth, b64(&r["key"])?, false)));
        Ok(z)
    }

    fn call(&mut self, cmd: u16, body: &Value) -> io::Result<Value> {
        self.sock.write_all(&ecb(&self.aes, frame(cmd, body), true))?;
        let head = self.read(32)?;
        let head = ecb(&self.aes, head, false);
        if head[..4] != MAGIC {
            return Err(io::Error::other("служба ZET ответила не по протоколу"));
        }
        let n = u32::from_le_bytes(head[12..16].try_into().unwrap()) as usize;
        if n == 0 {
            return Ok(Value::Null);
        }
        let body = self.read(n.next_multiple_of(16))?;
        let body = ecb(&self.aes, body, false);
        serde_json::from_slice(&body[..n]).map_err(io::Error::other)
    }

    fn read(&mut self, n: usize) -> io::Result<Vec<u8>> {
        let mut buf = vec![0; n];
        self.sock.read_exact(&mut buf)?;
        Ok(buf)
    }

    pub fn modes(&mut self) -> io::Result<Vec<ModeInfo>> {
        let r = self.call(LIST_DEVICES, &json!({ "device_id": 255 }))?;
        let kb = r["devices"].as_array().and_then(|d| d.iter().find(|d| d["device_id"] == 1)).ok_or_else(|| io::Error::other("служба ZET не видит клавиатуру"))?;
        Ok(kb["modes"].as_array().into_iter().flatten().map(|m| mode_info(m["mode_index"].as_u64().unwrap_or(0) as u8, m["mode_id"].as_u64().unwrap_or(0) as u32)).collect())
    }

    /// Настройки, сохранённые в клавиатуре для режима (не обязательно включённого)
    pub fn effect(&mut self, mode: u8) -> io::Result<Effect> {
        serde_json::from_value(self.call(GET_EFFECT, &json!({ "mode": mode, "device_id": 1 }))?).map_err(io::Error::other)
    }

    /// Включить режим с этими настройками
    pub fn set_effect(&mut self, e: &Effect) -> io::Result<()> {
        let mut palette = e.palette.clone();
        palette.resize(8, 0);
        let body = json!({ "profile_id": 1, "device_id": 1, "mode": e.mode, "brightness": e.brightness, "speed": e.speed, "direction": e.direction, "palette": palette });
        self.call(SET_EFFECT, &body).map(|_| ())
    }
}

/// Заголовок: магия, 0 = запрос, команда, число нулей выравнивания тела (служба его проверяет), длина JSON
fn frame(cmd: u16, body: &Value) -> Vec<u8> {
    let data = serde_json::to_vec(body).unwrap();
    let pad = data.len().next_multiple_of(16) - data.len();
    let mut msg = MAGIC.to_vec();
    msg.extend([0, 0]);
    msg.extend(cmd.to_le_bytes());
    msg.extend([0, 0]);
    msg.extend((pad as u16).to_le_bytes());
    msg.extend((data.len() as u32).to_le_bytes());
    msg.extend([0; 16]);
    msg.extend(&data);
    msg.resize(msg.len() + pad, 0);
    msg
}

/// Младшие 16 бит mode_id — по полубайту: ступени скорости, яркости, направлений и число цветов (как KbApi.modeInfoFromId)
fn mode_info(index: u8, id: u32) -> ModeInfo {
    ModeInfo { index, speeds: (id >> 12 & 15) as u8, brights: (id >> 8 & 15) as u8, dirs: (id >> 4 & 15) as u8, colors: (id & 15) as u8 }
}

fn cipher(hex: &[u8]) -> Aes256 {
    let key: Vec<u8> = hex.chunks(2).map(|h| u8::from_str_radix(std::str::from_utf8(h).unwrap(), 16).unwrap()).collect();
    Aes256::new(GenericArray::from_slice(&key))
}

fn ecb(c: &Aes256, mut data: Vec<u8>, encrypt: bool) -> Vec<u8> {
    for block in data.chunks_exact_mut(16) {
        let block = GenericArray::from_mut_slice(block);
        if encrypt { c.encrypt_block(block) } else { c.decrypt_block(block) }
    }
    data
}

fn b64(v: &Value) -> io::Result<Vec<u8>> {
    B64.decode(v.as_str().unwrap_or_default()).map_err(io::Error::other)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frame_matches_wkbapp() {
        // так wkbapp.exe запрашивает эффект режима 13 (снято перехватом)
        let f = frame(GET_EFFECT, &json!({ "mode": 13, "device_id": 1 }));
        assert_eq!(&f[..16], &[0xaf, 0x3e, 0x5d, 0xa2, 0, 0, 0x16, 0, 0, 0, 7, 0, 0x19, 0, 0, 0]);
        assert_eq!(&f[32..57], br#"{"device_id":1,"mode":13}"#);
        assert_eq!(f.len(), 64);
    }

    #[test]
    fn mode_ids_from_service() {
        assert_eq!(mode_info(1, 0x17701), ModeInfo { index: 1, speeds: 7, brights: 7, dirs: 0, colors: 1 }); // статичный
        assert_eq!(mode_info(3, 0x37742), ModeInfo { index: 3, speeds: 7, brights: 7, dirs: 4, colors: 2 }); // поток
        assert_eq!(mode_info(13, 0xe0801), ModeInfo { index: 13, speeds: 0, brights: 8, dirs: 0, colors: 1 }); // своя раскраска
    }

    #[test]
    fn ecb_roundtrip() {
        let c = cipher(K_AUTH.as_bytes());
        let data: Vec<u8> = (0..32).collect();
        assert_eq!(ecb(&c, ecb(&c, data.clone(), true), false), data);
    }

    #[test]
    #[ignore] // нужна запущенная служба ZET: cargo test -- --ignored --nocapture zet_read
    fn zet_read() {
        let mut z = Zet::connect().unwrap();
        println!("{:?}", z.modes().unwrap());
        println!("{:?}", z.effect(MODE_STATIC).unwrap());
    }
}
