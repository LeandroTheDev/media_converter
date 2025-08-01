use std::ffi::OsStr;
use std::fs::DirEntry;
use std::io::{Error, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus};
use std::time::Duration;
use std::{env, fs, io, thread};

fn copy_recursively(src: &Path, dst: &Path) -> io::Result<()> {
    if !dst.exists() {
        fs::create_dir_all(dst)?;
    }

    for entry_result in fs::read_dir(src)? {
        let entry = entry_result?;
        let path = entry.path();
        let dest_path = dst.join(entry.file_name());

        if path.is_dir() {
            copy_recursively(&path, &dest_path)?;
        } else {
            fs::copy(&path, &dest_path)?;
        }
    }

    Ok(())
}

fn collect_files_except_lock(dir: &Path) -> io::Result<Vec<PathBuf>> {
    let mut files: Vec<PathBuf> = Vec::new();

    if dir.is_dir() {
        for entry_result in fs::read_dir(dir)? {
            let entry: DirEntry = entry_result?;
            let path: PathBuf = entry.path();

            if path.is_dir() {
                // Recurse into subdirectories
                files.extend(collect_files_except_lock(&path)?);
            } else if path.is_file() {
                // Skip folder.lock
                if let Some(name) = path.file_name() {
                    if name != "folder.lock" {
                        files.push(path);
                    }
                }
            }
        }
    }

    Ok(files)
}

fn clear_temp_folder(temp_folder: &Path) -> std::io::Result<()> {
    if temp_folder.is_dir() {
        for entry in fs::read_dir(temp_folder)? {
            let entry: DirEntry = entry?;
            let path: PathBuf = entry.path();

            if path.is_file() {
                fs::remove_file(path)?;
            } else if path.is_dir() {
                fs::remove_dir_all(path)?;
            }
        }
    }
    Ok(())
}

fn main() {
    let args: Vec<String> = env::args().collect();

    // Default values
    let mut fps: u32 = 30;
    let mut resolution: String = String::from("720p");
    let mut bitrate: u32 = 3000;
    let mut result_folder: String = String::new();
    let mut temp_folder: String = String::new();
    let mut extension: String = String::new();
    let mut ffmpeg_path: String = String::new();
    let mut input_path: String = String::new();

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--fps" => {
                if i + 1 < args.len() {
                    fps = args[i + 1].parse().unwrap_or_else(|_| {
                        eprintln!("Invalid FPS value, using default: {}", fps);
                        fps
                    });
                    i += 1;
                }
            }
            "--resolution" => {
                if i + 1 < args.len() {
                    resolution = args[i + 1].clone();
                    i += 1;
                }
            }
            "--bitrate" => {
                if i + 1 < args.len() {
                    bitrate = args[i + 1].parse().unwrap_or_else(|_| {
                        eprintln!("Invalid bitrate value, using default: {}", bitrate);
                        bitrate
                    });
                    i += 1;
                }
            }
            "--resultFolder" => {
                if i + 1 < args.len() {
                    result_folder = String::from(&args[i + 1]);
                    i += 1;
                }
            }
            "--tempFolder" => {
                if i + 1 < args.len() {
                    temp_folder = String::from(&args[i + 1]);
                    i += 1;
                }
            }
            "--extension" => {
                if i + 1 < args.len() {
                    extension = String::from(args[i + 1].clone());
                    i += 1;
                }
            }
            "--ffmpegPath" => {
                if i + 1 < args.len() {
                    ffmpeg_path = String::from(args[i + 1].clone());
                    i += 1;
                }
            }
            _ => {
                // If this is the last argument and does not start with "--", treat as input_path
                if i == args.len() - 1 && !args[i].starts_with("--") {
                    input_path = String::from(args[i].clone());
                } else {
                    eprintln!("Unknown or malformed parameter: {}", args[i]);
                }
            }
        }
        i += 1;
    }

    // Input path is mandatory
    if input_path.is_empty() {
        eprintln!(
            "Usage: [--resultFolder <folder>] [--resolution <resolution>] [--bitrate <number>] [--ffmpegPath <path>] [--tempFolder <path>] [--extension <ext>] <directory/file>"
        );
        std::process::exit(1);
    }

    // Set default ffmpeg_path if not provided
    if ffmpeg_path.is_empty() {
        #[cfg(windows)]
        {
            let exe_path: PathBuf = env::current_exe().unwrap_or_else(|_| PathBuf::from("."));
            let mut ffmpeg_default: PathBuf = exe_path
                .parent()
                .unwrap_or_else(|| Path::new("."))
                .to_path_buf();
            ffmpeg_default.push("libraries");
            ffmpeg_default.push("ffmpeg.exe");
            ffmpeg_path = ffmpeg_default.to_string_lossy().to_string();
        }
        #[cfg(not(windows))]
        {
            ffmpeg_path = "/usr/bin/ffmpeg".to_string();
        }
    }

    // Set default temp_folder if not provided
    if temp_folder.is_empty() {
        let exe_path: PathBuf = env::current_exe().unwrap_or_else(|_| PathBuf::from("."));
        let mut temp_default: PathBuf = exe_path
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .to_path_buf();
        temp_default.push("temp");
        temp_folder = temp_default.to_string_lossy().to_string();
    }

    // Set default result_folder if not provided
    if result_folder.is_empty() {
        let exe_path: PathBuf = env::current_exe().unwrap_or_else(|_| PathBuf::from("."));
        let mut results_default: PathBuf = exe_path
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .to_path_buf();
        results_default.push("results");
        result_folder = results_default.to_string_lossy().to_string();
    }

    let input_path_obj: &Path = Path::new(&input_path);
    let temp_path_obj: &Path = Path::new(&temp_folder);

    // Wait if folder.lock exists
    let lock_file: PathBuf = temp_path_obj.join("folder.lock");
    while lock_file.exists() {
        println!("folder.lock exists, waiting 1 second..., force delete the folder.lock");
        thread::sleep(Duration::from_secs(1));
    }

    // Create temp folder if missing
    if !temp_path_obj.exists() {
        if let Err(e) = fs::create_dir_all(temp_path_obj) {
            eprintln!("Failed to create temp folder '{}': {}", temp_folder, e);
            std::process::exit(1);
        }
    } else {
        match fs::File::create(&lock_file) {
            Ok(mut f) => {
                if let Err(e) = f.write_all(b"lock") {
                    eprintln!("Failed to write to lock file: {}", e);
                    std::process::exit(1);
                }
            }
            Err(e) => {
                eprintln!("Failed to create lock file: {}", e);
                std::process::exit(1);
            }
        }
    }

    // Copy input to temp_folder (file or directory)
    if input_path_obj.is_dir() {
        println!("Input path is directory. Copying recursively to temp folder...");
        if let Err(e) = copy_recursively(input_path_obj, temp_path_obj) {
            eprintln!("Failed to copy directory: {}", e);
            std::process::exit(1);
        }
    } else if input_path_obj.is_file() {
        println!("Input path is file. Copying to temp folder...");
        let file_name = input_path_obj.file_name().unwrap();
        let dest_file = temp_path_obj.join(file_name);
        if let Err(e) = fs::copy(input_path_obj, dest_file) {
            eprintln!("Failed to copy file: {}", e);
            std::process::exit(1);
        }
    } else {
        eprintln!("Input path does not exist or is not a file/directory.");
        std::process::exit(1);
    }

    println!("Files copied and folder.lock created. Proceeding with further processing...");

    let files: Vec<PathBuf> = collect_files_except_lock(temp_path_obj).unwrap_or_else(|e| {
        eprintln!("Failed to read files in temp folder: {}", e);
        std::process::exit(1);
    });

    if !Path::new(&result_folder).exists() {
        if let Err(e) = fs::create_dir_all(&result_folder) {
            eprintln!("Failed to create result folder '{}': {}", result_folder, e);
            std::process::exit(1);
        }
    }

    for input_file in files {
        // Build output file path inside result_folder with the same file name but new extension
        let file_name: &OsStr = input_file.file_name().expect("Failed to get file name");
        let mut output_file: PathBuf = PathBuf::from(&result_folder);
        output_file.push(file_name);

        if !extension.is_empty() {
            output_file.set_extension(&extension);
        }

        println!(
            "Converting file:\n  input: {}\n  output: {}",
            input_file.display(),
            output_file.display()
        );

        let scale_filter: &'static str = match resolution.as_str() {
            "1080p" => "scale=-2:1080",
            "720p" => "scale=-2:720",
            "480p" => "scale=-2:480",
            "360p" => "scale=-2:360",
            "240p" => "scale=-2:240",
            "144p" => "scale=-2:144",
            _ => {
                eprintln!("Unknown resolution '{}', using original size", resolution);
                "scale=iw:ih"
            }
        };

        let status: Result<ExitStatus, Error> = Command::new(&ffmpeg_path)
            .arg("-y") // overwrite without asking
            .arg("-i")
            .arg(&input_file)
            .arg("-r") // fps
            .arg(fps.to_string())
            .arg("-b:v") // video bitrate
            .arg(format!("{}k", bitrate))
            .arg("-vf") // video filter for resolution (scale)
            .arg(scale_filter)
            .arg(&output_file)
            .status();

        match status {
            Ok(s) if s.success() => {
                println!("Successfully converted {}", input_file.display());
            }
            Ok(s) => {
                eprintln!(
                    "ffmpeg exited with status {} for file {}",
                    s,
                    input_file.display()
                );
            }
            Err(e) => {
                eprintln!(
                    "Failed to execute ffmpeg for file {}: {}",
                    input_file.display(),
                    e
                );
            }
        }
    }

    if let Err(e) = clear_temp_folder(temp_path_obj) {
        eprintln!("Failed to clear temp folder: {}", e);
    } else {
        println!("Temp folder cleared");
    }
}
