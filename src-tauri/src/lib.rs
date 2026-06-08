use rodio::{Decoder, DeviceSinkBuilder, MixerDeviceSink};
use rusty_ytdl::{Video, VideoOptions, VideoQuality, VideoSearchOptions};
use std::fs::File;
use std::io::BufReader;
use std::sync::{Arc, Mutex};
use tauri::ipc::IpcResponse;
use tauri::State;
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
    let yt = YtMusic::new_unauthenticated()
        .await
        .map_err(|e| format!("Failed to initialize YTM Client: {}", e))?;

    println!("Here");

    // 2. Query YouTube Music for search suggestions
    let mut results = yt
        .search_songs("The strokes")
        .await
        .map_err(|e| format!("YTM API Error: {}", e))?;

    // 3. Map out and return the raw suggestions
    // (Depending on your exact ytmapi_rs version, results may be a vector of structs or strings)
    // 3. Extract the first result (Make sure 'results' is declared with 'let mut')

    // FIX 1: Safely convert VideoID to a clean String by using its Debug representation
    let video_id_cow = &results[0]
        .video_id
        .get_video_id()
        .expect("Failed to get video ID");

    println!("{}", &*video_id_cow);
    println!("Fetched Video ID: {}", video_id_cow);

    // 4. Request the Audio Stream URL from YouTube using rusty_ytdl
    let video_options = VideoOptions {
        quality: VideoQuality::HighestAudio,
        filter: VideoSearchOptions::Audio,
        ..Default::default()
    };

    let video = Video::new_with_options(&**video_id_cow, video_options)
        .map_err(|e| format!("Failed to initialize video fetcher: {}", e))?;

    print!("There for now 0");

    let video_info = video
        .get_info()
        .await
        .map_err(|e| format!("Failed to fetch video info: {}", e))?;

    // FIX 2: Check for the format and cleanly return the String stream_url
    if let Some(format) = video_info.formats.first() {
        print!("There for now");
        let stream_url = format.url.clone();
        println!("Direct Audio Stream URL: {}", stream_url);

        Ok(stream_url)
    } else {
        Err("No suitable audio formats found for this video".to_string())
    }
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
