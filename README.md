# Media Converter

A command-line tool written in Rust that batch processes video files, converting them to a specified resolution, frame rate, bitrate, and file extension using `ffmpeg`.

It supports input as a single file or an entire directory, and it handles temporary file storage and cleanup automatically.

---

## Features

- ✅ Convert a single video or all videos in a folder.
- ✅ Customize output resolution (e.g., 1080p, 720p, etc.).
- ✅ Set output frame rate (FPS).
- ✅ Define custom video bitrate.
- ✅ Change the file extension (e.g., `.mp4`, `.mkv`, etc.).
- ✅ Automatically copies files to a temporary folder before processing.
- ✅ Automatically deletes the temporary folder contents after conversion.
- ✅ Prevents race conditions with a lock file (`folder.lock`).

---

## 🔧 Requirements
- 🎬 ffmpeg (for audio/video conversion)

# 💻 Installation Windows
Windows users needs to download [ffmpeg](https://ffmpeg.org/download.html), put [ffmpeg](https://ffmpeg.org/download.html) inside media_downloader/libraries

# 🐧 Installation Windows
Install ffmpeg from your package manager

---

## 🚀 Usage
Opening executable is the easy and friendly way

⚙️ Additional Arguments:
| Option           | Description                                      | Example                  |
|------------------|--------------------------------------------------|--------------------------|
| `--fps`          | Set the output frame rate (default: 30)          | `--fps 24`               |
| `--resolution`   | Output resolution (`1080p`, `720p`, `480p`, etc.)| `--resolution 720p`      |
| `--bitrate`      | Video bitrate in kbps (default: 3000)            | `--bitrate 2500`         |
| `--extension`    | Output file extension                            | `--extension mp4`        |
| `--tempFolder`   | Temp folder for intermediate files               | `--tempFolder ./temp`    |
| `--resultFolder` | Folder where converted files will be stored      | `--resultFolder ./results` |
| `--ffmpegPath`   | Path to the `ffmpeg` executable                  | `--ffmpegPath ./ffmpeg.exe` |

---

### Full Example: 
``media_converter --fps 30 --bitrate 2500 --resolution 720p --extension mp4 ./videos``

---

## 📦 Building
```bash
git clone https://github.com/LeandroTheDev/media_vonerter
cd media_vonerter
cargo build --release
```