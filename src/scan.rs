use crate::dto::{AlbumMetadata, ImageFile, MusicDir, MusicFile};
use crate::mp3_tags::get_mp3_metadata;
use anyhow::{anyhow, Result};
use std::num::NonZero;
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicUsize;
use std::sync::Arc;
use std::time::{Duration, Instant};

pub fn scan_dirs(path: PathBuf) -> Result<Vec<MusicDir>> {
    let (result_sender, result_receiver) = crossbeam_channel::unbounded::<Result<MusicDir>>();
    let (subdir_sender, subdir_receiver) = crossbeam_channel::unbounded::<PathBuf>();

    let cpus = std::thread::available_parallelism()
        .ok()
        .map(NonZero::get)
        .unwrap_or(1_usize);
    let idle = Arc::new(AtomicUsize::new(0));

    let mut thread_handles = vec![];
    for idx in 0..cpus {
        let result_sender = result_sender.clone();
        let subdir_sender = subdir_sender.clone();
        let subdir_receiver = subdir_receiver.clone();
        let idle = Arc::clone(&idle);

        thread_handles.push(std::thread::spawn(move || {
            eprintln!("Started thread [{idx}]");

            loop {
                let idle_threads = idle.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1;
                let subdir =
                    match subdir_receiver.recv_deadline(Instant::now() + Duration::from_millis(500)) {
                        Ok(subdir) => {
                            idle.fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
                            subdir
                        },
                        Err(_) => {
                            if idle_threads == cpus {
                                eprintln!("Thread {idx} timed out getting new dir, it will quit");
                                break;
                            } else {
                                eprintln!(
                                    "Thread {idx} timed out getting new dir but there are {} active threads so it will not quit",
                                    cpus - idle_threads
                                );
                                idle.fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
                                continue;
                            }
                        }
                    };

                eprintln!("Thread {idx}: Scanning directory [{subdir:?}]");
                let result: Result<(Option<MusicDir>, Vec<PathBuf>)> = scan_dir(&subdir);

                match result {
                    Ok((option_music_dir, subdirs)) => {
                        if let Some(music_dir) = option_music_dir {
                            result_sender.send(Ok(music_dir)).expect("Failed to send to the result sender");
                        }
                        for subdir in subdirs {
                            subdir_sender.send(subdir).expect("Failed to send to the subdir sender");
                        }
                    }
                    Err(err) => {
                        let _ = result_sender.send(Err(anyhow!(
                            "Error on thread {idx} scanning directory {subdir:?}: {err:?}"
                        )));
                    }
                }
            }

            eprintln!("Thread {idx} exit");
        }));
    }

    subdir_sender.send(path)?;
    drop(result_sender);
    drop(subdir_sender);

    for handle in thread_handles {
        let thread_id = handle.thread().id();
        if handle.join().is_err() {
            return Err(anyhow!("failed to join thread handle {:?}", thread_id));
        }
    }

    let mut results = vec![];
    for result in result_receiver {
        results.push(result?);
    }

    Ok(results)
}

fn scan_dir(path: &PathBuf) -> Result<(Option<MusicDir>, Vec<PathBuf>)> {
    let mut music_files: Vec<(u32, MusicFile)> = Vec::new();
    let mut subdirs: Vec<PathBuf> = Vec::new();
    let mut image_files: Vec<ImageFile> = Vec::new();
    let mut album_metadata: Option<AlbumMetadata> = None;

    for (index, file) in path.read_dir()?.enumerate() {
        let file = file?;

        if file.file_name().to_string_lossy().ends_with(".mp3") {
            let (track, track_metadata, new_album_metadata) =
                get_mp3_metadata(file.path(), index as u32)?.into_track_and_metadatas();
            album_metadata.get_or_insert(new_album_metadata);

            music_files.push((
                track,
                MusicFile {
                    file_path: assert_unicode_path(file.path().file_name().unwrap()),
                    metadata: track_metadata,
                },
            ));
        }

        if let Some("jpeg" | "jpg" | "png") = file.file_name().to_string_lossy().split('.').last() {
            image_files.push(ImageFile {
                file_path: assert_unicode_path(file.path().file_name().unwrap()),
                use_as_cover: false,
            });
        }

        if file.metadata()?.is_dir() {
            subdirs.push(file.path());
        }
    }

    let music_files = sorted_by_track_number(music_files);

    Ok((
        album_metadata.map(|metadata| MusicDir {
            path: assert_unicode_path(path),
            metadata,
            music_files,
            image_files,
        }),
        subdirs,
    ))
}

fn sorted_by_track_number(mut music_files_with_tracks: Vec<(u32, MusicFile)>) -> Vec<MusicFile> {
    music_files_with_tracks.sort_by_key(|it| it.0);
    music_files_with_tracks.into_iter().map(|it| it.1).collect()
}

fn assert_unicode_path(path: impl AsRef<Path>) -> String {
    path.as_ref()
        .to_str()
        .unwrap_or_else(|| panic!("Path must be UTF-8! [{}]", path.as_ref().to_string_lossy()))
        .to_string()
}
