#![cfg_attr(not(debug_assertions),windows_subsystem = "windows")]
use std::collections::HashMap;
use std::{time,thread,process};
use std::io::{Cursor,Error};
use clipboard_win::{formats,get_clipboard,set_clipboard_string};
use image::io::Reader as ImageReader;
use image::ImageOutputFormat;
use reqwest;
use reqwest::header::{HeaderName,HeaderMap};
use serde_json;
use base64;
use webbrowser;
use soloud::*;

const CLIENT_ID: &str = env!("IMGUR_CLIENT_ID");

fn show_notification(title: &str,message: &str) -> Result<process::ExitStatus, Error> {
    let notify = process::Command::new("notifu64.exe")
        .args(["/m",message,"/p",title,"/i","icon.ico","/d","30000"])
        .spawn()
        .expect("falha a executar notifu64.exe")
        .wait();
    notify
}

fn play_sound() {
    let sl = Soloud::default().unwrap();
    let mut wav = audio::Wav::default();
    wav.load_mem(include_bytes!("upload.mp3")).unwrap();
    sl.play(&wav);
    while sl.voice_count() > 0 {
        thread::sleep(time::Duration::from_millis(100))
    }
}

fn upload_image(image: Vec<u8>) -> String {
    let base64img = base64::encode(image);
    let mut postdata = HashMap::new();
    postdata.insert("image",base64img);
    postdata.insert("type","base64".to_string());

    let mut headers = HeaderMap::new();
    let name: HeaderName = "Authorization".parse().unwrap();
    headers.insert(name,format!("Client-ID {}",CLIENT_ID).parse().unwrap());

    let client = reqwest::blocking::Client::new();
    let response = client.post("https://api.imgur.com/3/upload.json")
        .headers(headers)
        .json(&postdata)
        .send()
        .expect("Falha ao fazer request.");
    let json: serde_json::Value = response.json().expect("Falha ao deserializar json.");
    json["data"]["link"].as_str().unwrap().to_owned()
}

fn main() {
    #[cfg(debug_assertions)]
    println!("Pegando image do clipboard...");
    let clipbmp = match get_clipboard(formats::Bitmap) {
        Ok(img) => img,
        Err(_) => {
            #[cfg(debug_assertions)]
            println!("Erro ao pegar imagem do clipboard.");
            show_notification("Erro ao pegar imagem", "O que está no área de trasferência provavelmente não é uma imagem.").expect("Erro ao mostrar notificação.");
            return;
        }
    };

    #[cfg(debug_assertions)]
    println!("Covertendo BMP para JPG");
    let dinamicimg = ImageReader::new(Cursor::new(clipbmp)).with_guessed_format().unwrap().decode().unwrap();
    let mut jpgbuff: Cursor<Vec<u8>> = Cursor::new(Vec::new());
    dinamicimg.write_to(&mut jpgbuff,ImageOutputFormat::Jpeg(100)).unwrap();

    #[cfg(debug_assertions)]
    println!("Enviando imagem...");
    let link = upload_image(jpgbuff.get_ref().to_vec());

    #[cfg(debug_assertions)]
    println!("Copiando link para o clipboard...");
    set_clipboard_string(&link).expect("Erro ao copiar link para o clipboard");

    #[cfg(debug_assertions)]
    println!("Tocando som...");
    thread::spawn(||play_sound());

    #[cfg(debug_assertions)]
    println!("Mostrando notificação...");
    let notify = show_notification("Upload completo", &format!("Link copiado: {}",&link)).expect("Falha ao mostrar notificação.");
    if notify.code().unwrap() == 3 {
        webbrowser::open(&link).unwrap();
    }
}
