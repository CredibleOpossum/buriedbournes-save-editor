use aes::Aes256;
use base64::{engine::general_purpose, Engine as _};
use block_modes::block_padding::Pkcs7;
use block_modes::{BlockMode, Cbc};
use serde_json::Value;
use std::error;
use std::path::Path;
use winreg::enums::HKEY_CURRENT_USER;
use winreg::enums::REG_BINARY;
use winreg::RegKey;
use winreg::RegValue;

const SAVE_LOCATION: &str = "Software\\Nussygame\\Buriedbornes";
const SAVE_NAME: &str = "UqgwrrCj_h2961706936";
const SAVE_KEY: &[u8] = b"a5VckiKFIP76hvPnoae9PRZ5x1PzUiVA";
const IV_SIZE: usize = 16;

type Aes256Cbc = Cbc<Aes256, Pkcs7>;

pub fn backup() -> Vec<u8> {
    let hklm = RegKey::predef(HKEY_CURRENT_USER);
    let data_location = hklm.open_subkey(SAVE_LOCATION).unwrap();
    let bytes = data_location.get_raw_value(SAVE_NAME).unwrap().bytes;
    println!("{:?}", bytes);
    bytes
}

pub fn restore(bytes: Vec<u8>) {
    println!("{:?}", bytes);
    let hklm = RegKey::predef(HKEY_CURRENT_USER);
    let reg_data = RegValue {
        vtype: REG_BINARY,
        bytes: bytes,
    };
    let path = Path::new(SAVE_LOCATION);
    let (key, _disp) = hklm.create_subkey(path).unwrap();
    key.set_raw_value(SAVE_NAME, &reg_data).unwrap();
}

fn decrypt(cipher_bytes: &[u8], iv: &[u8]) -> Result<String, Box<dyn error::Error>> {
    let mut encrypted_data = cipher_bytes.to_owned();
    let cipher = Aes256Cbc::new_from_slices(SAVE_KEY, iv)?;
    cipher.decrypt(&mut encrypted_data).unwrap().to_vec();
    Ok(String::from_utf8(encrypted_data)?)
}

fn encrypt(plain_text: &[u8], iv: &[u8]) -> Result<Vec<u8>, Box<dyn error::Error>> {
    let bytes = plain_text.to_owned();
    let cipher = Aes256Cbc::new_from_slices(SAVE_KEY, iv)?;
    Ok(cipher.encrypt_vec(&bytes))
}

pub fn savefile_read() -> (Value, Vec<u8>, u8) {
    let hklm = RegKey::predef(HKEY_CURRENT_USER);
    let data_location = hklm.open_subkey(SAVE_LOCATION).unwrap();
    let mut bytes = data_location.get_raw_value(SAVE_NAME).unwrap().bytes;
    let remainder = bytes.pop().unwrap(); // remove remainder
    let base64_string = String::from_utf8(bytes).unwrap(); // decode
    let decoded = general_purpose::STANDARD.decode(base64_string).unwrap();
    let (iv, cipher_bytes) = decoded.split_at(IV_SIZE); // decrypt
    let decrypted = decrypt(cipher_bytes, iv).unwrap();
    let (json_data, _unknown_data) = decrypted.split_at(decrypted.rfind('}').unwrap() + 1);
    let json: Value = serde_json::from_str(json_data).unwrap();
    (json, iv.to_vec(), remainder)
}

pub fn savefile_writejson(json: Value, iv: Vec<u8>, remainder: u8) {
    let hklm = RegKey::predef(HKEY_CURRENT_USER);
    let bytes: Vec<u8> = json.to_string().try_into().unwrap();
    let mut re_encrypted = iv.to_vec();
    re_encrypted.extend(encrypt(&bytes, &iv).unwrap()); // encrypt
    let binding = general_purpose::STANDARD.encode(re_encrypted); // encode
    let mut stored_data = binding.as_bytes().to_vec();
    stored_data.push(remainder);
    let raw_data = stored_data;
    let reg_data = RegValue {
        vtype: REG_BINARY,
        bytes: raw_data,
    };
    let path = Path::new(SAVE_LOCATION);
    let (key, _disp) = hklm.create_subkey(path).unwrap();
    key.set_raw_value(SAVE_NAME, &reg_data).unwrap();
}
