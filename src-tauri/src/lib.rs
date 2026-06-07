use rodio::{Decoder, DeviceSinkBuilder, MixerDeviceSink};
use std::fs::File;
use std::io::BufReader;
use std::sync::{Arc, Mutex};
use tauri::State;
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
async fn fetch_suggestions() -> Result<Vec<String>, String> {
    // 1. Initialize the unauthenticated engine
    let yt = YtMusic::new_unauthenticated()
        .await
        .map_err(|e| format!("Failed to initialize YTM Client: {}", e))?;

    println!("Here");

    // 2. Query YouTube Music for search suggestions
    let results = yt
        .search_songs("The strokes")
        .await
        .map_err(|e| format!("YTM API Error: {}", e))?;

    println!("There");

    // 3. Map out and return the raw suggestions
    // (Depending on your exact ytmapi_rs version, results may be a vector of structs or strings)
    let suggestions = results.into_iter().map(|s| s.title).collect();

    println!("{:?}", suggestions);

    Ok(suggestions)
}

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
