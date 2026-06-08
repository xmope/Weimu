use rodio::{Decoder, DeviceSinkBuilder, MixerDeviceSink};
use rusty_ytdl::{Video, VideoOptions, VideoQuality, VideoSearchOptions};
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tauri::ipc::IpcResponse;
use tauri::State;
use tokio::fs::create_dir_all;
use yt_dlp::client::deps::Libraries;
use yt_dlp::{Downloader, VideoSelection};
use ytmapi_rs::common::YoutubeID;
use ytmapi_rs::query::playlist::GetWatchPlaylistQueryID;
use ytmapi_rs::{Error as YtError, YtMusic};
// We store the handle inside our Tauri state to keep the audio thread alive.
struct AudioState {
    // MixerDeviceSink keeps the playback connection open
    audio_handle: Arc<Mutex<Option<MixerDeviceSink>>>,
}

#[tauri::command]
fn play_audio(state: State<'_, AudioState>, file_path: String) -> Result<(), String> {
    // 1. Get an OS-Sink handle to the default physical sound device.
    let handle = DeviceSinkBuilder::open_default_sink()
        .map_err(|e| format!("Failed to open default audio stream: {}", e))?;

    // 2. Load the sound file
    let file = File::open(&file_path).map_err(|e| format!("Failed to open file: {}", e))?;

    // Wrap in BufReader for performance/compatibility with rodio's Decoder
    let buf_reader = BufReader::new(file);

    // 3. Decode that sound file into a source
    let source =
        Decoder::try_from(buf_reader).map_err(|e| format!("Failed to decode audio file: {}", e))?;

    // 4. Play the sound directly on the device mixer
    handle.mixer().add(source);

    // 5. Save the handle into our global Tauri state so it isn't dropped immediately!
    let mut guard = state.audio_handle.lock().unwrap();
    *guard = Some(handle);

    Ok(())
}

#[tauri::command]
async fn fetch_suggestions() -> Result<String, String> {
    // 1. Initialize the unauthenticated engine
    // let yt = YtMusic::new_unauthenticated()
    //     .await
    //     .map_err(|e| format!("Failed to initialize YTM Client: {}", e))?;

    // println!("Here");

    // // 2. Query YouTube Music for search suggestions
    // let mut results = yt
    //     .search_songs("The strokes")
    //     .await
    //     .map_err(|e| format!("YTM API Error: {}", e))?;

    // println!("asdasd");

    let executables_dir = PathBuf::from("libs");
    let output_dir = PathBuf::from("output");

    // Create fetcher and install binaries
    let downloader = Downloader::with_new_binaries(executables_dir, output_dir).await;

    let _ = downloader
        .map_err(|e| format!("Failed to create downloader: {}", e))?
        .build()
        .await
        .map_err(|e| format!("Failed to build downloader: {}", e))?;

    let libraries_dir = PathBuf::from("libs");
    let output_dir = PathBuf::from("output");

    let youtube = libraries_dir.join("yt-dlp");
    let ffmpeg = libraries_dir.join("ffmpeg");

    println!("Here 1");
    let libraries = Libraries::new(youtube, ffmpeg);
    let downloader = Downloader::builder(libraries, output_dir)
        .build()
        .await
        .map_err(|e| format!("Failed to build downloader: {}", e))?;

    println!("Here 2");

    let url = String::from("https://www.youtube.com/watch?v=-_6dHIPVoTM");

    let video = downloader
        .fetch_video_infos_fresh(url)
        .await
        .map_err(|e| format!("Failed to fetch video infos: {}", e))?;

    let best_audio_format = video
        .best_audio_format()
        .expect("No best audio format found");

    let asda = &&best_audio_format
        .download_info
        .url
        .as_ref()
        .expect("No audio URL found");

    println!("{}", asda);

    println!("Here 3");
    downloader
        .download_audio_stream(&video, "audio.mp3")
        .await
        .map_err(|e| format!("Failed to download audio stream: {}", e))?;

    println!("Here 4");

    Ok("Done".to_string())
} // <--- End of function closes here cleanly

#[tauri::command]
fn stop_audio(state: State<'_, AudioState>) -> Result<(), String> {
    let mut guard = state.audio_handle.lock().unwrap();

    // Taking the handle out of the Option and letting it drop
    // will instantly stop the background playback thread.
    if let Some(handle) = guard.take() {
        drop(handle);
    }

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(AudioState {
            audio_handle: Arc::new(Mutex::new(None)),
        })
        .invoke_handler(tauri::generate_handler![
            play_audio,
            stop_audio,
            fetch_suggestions
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
