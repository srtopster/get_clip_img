#![windows_subsystem = "windows"]
use arboard::{Clipboard};
use image::{RgbaImage,ImageBuffer,DynamicImage,ImageFormat};
use std::io::Cursor;
use std::{thread,process,str,time};
use base64;
use curl::easy::{Easy,Form,List};
use tinyjson;
use webbrowser;
use soloud::*;

fn play_sound() {
    let sl = Soloud::default().unwrap();
    let mut wav = audio::Wav::default();
    wav.load_mem(include_bytes!("upload.mp3")).unwrap();
    sl.play(&wav);
    while sl.voice_count() > 0 {
        thread::sleep(time::Duration::from_millis(100))
    }
}

fn main() {
    let cb_image = match Clipboard::new().unwrap().get_image(){
        Ok(img) => img,
        Err(_) => return
    };
    let raw_image: RgbaImage = ImageBuffer::from_raw(
        cb_image.width.try_into().unwrap(),
        cb_image.height.try_into().unwrap(),
        cb_image.bytes.into_owned()).unwrap();
    let mut png: Vec<u8> = Vec::new();
    let image = DynamicImage::ImageRgba8(raw_image);
    image.write_to(&mut Cursor::new(&mut png), ImageFormat::Png).unwrap();
    
    let client_id = "yourclientid";
    let mut form = Form::new();
    form.part("image").contents(base64::encode(&png).as_bytes()).add().unwrap();
    form.part("type").contents("base64".as_bytes()).add().unwrap();

    let mut list = List::new();
    list.append(&format!("Authorization: Client-ID {}",client_id)).unwrap();

    let mut response = Vec::new();
    let mut easy = Easy::new();
    easy.url("https://api.imgur.com/3/upload.json").unwrap();
    easy.post(true).unwrap();
    easy.http_headers(list).unwrap();
    easy.httppost(form).unwrap();
    {
    let mut transfer = easy.transfer();
    transfer.write_function(|data|{
        response.extend_from_slice(data);
        Ok(data.len())
    }).unwrap();
    transfer.perform().unwrap();
    }

    let json: tinyjson::JsonValue = str::from_utf8(&response).unwrap().parse().unwrap();
    let link = json["data"]["link"].stringify().unwrap().replace('"',"");

    Clipboard::new().unwrap().set_text(link.to_owned()).unwrap();
    thread::spawn(||play_sound());
    let notify = process::Command::new("notifu64.exe")
        .args(["/m",&format!("Link copiado: {}",&link),"/p","Upload completo","/i","icon.ico","/d","30000"])
        .spawn()
        .expect("falha a executar notifu64.exe")
        .wait();
    if notify.unwrap().code().unwrap() == 3 {
        webbrowser::open(&link).unwrap();
    }
}