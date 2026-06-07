<script setup lang="ts">
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";

// Reactive states for UI feedback
const filePath = ref<string>("/home/user/music/track.mp3");
const isPlaying = ref<boolean>(false);
const errorMessage = ref<string | null>(null);

// Invoke the play_audio command
async function handlePlay() {
    if (!filePath.value) {
        errorMessage.value = "Please provide a valid file path.";
        return;
    }

    try {
        errorMessage.value = null;
        isPlaying.value = true;

        // Calls your Rust #[tauri::command]
        await invoke("fetch_suggestions");
    } catch (err) {
        errorMessage.value = String(err);
        isPlaying.value = false;
    }
}

// Invoke the stop_audio command
async function handleStop() {
    try {
        errorMessage.value = null;

        // Calls your Rust stop command
        await invoke("stop_audio");

        isPlaying.value = false;
    } catch (err) {
        errorMessage.value = String(err);
    }
}
</script>

<template>
    <div class="audio-player">
        <h2>Tauri Audio Controller</h2>

        <div class="input-group">
            <label for="file-path">Absolute Audio Path:</label>
            <input
                id="file-path"
                v-model="filePath"
                type="text"
                placeholder="e.g., /absolute/path/to/audio.mp3"
            />
        </div>

        <div class="controls">
            <button
                @click="handlePlay"
                :disabled="isPlaying"
                class="btn btn-play"
            >
                {{ isPlaying ? "Playing..." : "Play Audio" }}
            </button>

            <button
                @click="handleStop"
                :disabled="!isPlaying"
                class="btn btn-stop"
            >
                Stop
            </button>
        </div>

        <div v-if="isPlaying" class="status listening">
            <span class="pulse-dot"></span> Audio stream active
        </div>

        <div v-if="errorMessage" class="error-msg">
            <strong>Error:</strong> {{ errorMessage }}
        </div>
    </div>
</template>

<style scoped>
.audio-player {
    max-width: 450px;
    margin: 2rem auto;
    padding: 1.5rem;
    background: #1e1e2e;
    color: #cdd6f4;
    border-radius: 12px;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
    font-family: sans-serif;
}

h2 {
    margin-top: 0;
    color: #cba6f7;
    font-size: 1.5rem;
}

.input-group {
    margin-bottom: 1.5rem;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
}

label {
    font-size: 0.85rem;
    color: #a6adc8;
}

input {
    padding: 0.75rem;
    background: #313244;
    border: 1px solid #45475a;
    border-radius: 6px;
    color: #cdd6f4;
    font-size: 0.9rem;
}

input:focus {
    outline: none;
    border-color: #cba6f7;
}

.controls {
    display: flex;
    gap: 1rem;
}

.btn {
    flex: 1;
    padding: 0.75rem;
    border: none;
    border-radius: 6px;
    font-weight: bold;
    cursor: pointer;
    transition: opacity 0.2s;
}

.btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
}

.btn-play {
    background: #a6e3a1;
    color: #11111b;
}

.btn-stop {
    background: #f38ba8;
    color: #11111b;
}

.status {
    margin-top: 1rem;
    display: flex;
    align-items: center;
    gap: 0.5rem;
    color: #a6e3a1;
    font-size: 0.9rem;
}

.pulse-dot {
    width: 8px;
    height: 8px;
    background: #a6e3a1;
    border-radius: 50%;
    animation: pulse 1.5s infinite;
}

.error-msg {
    margin-top: 1rem;
    padding: 0.75rem;
    background: rgba(243, 139, 168, 0.1);
    border-left: 4px solid #f38ba8;
    color: #f38ba8;
    font-size: 0.85rem;
}

@keyframes pulse {
    0% {
        transform: scale(0.8);
        opacity: 0.5;
    }
    50% {
        transform: scale(1.2);
        opacity: 1;
    }
    100% {
        transform: scale(0.8);
        opacity: 0.5;
    }
}
</style>
