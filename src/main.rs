use anyhow::Result;
use askama::Template;
use dto::MusicDir;
use std::io::BufRead;
use std::path::PathBuf;
use std::str::FromStr;

use crate::dto::{AlbumMetadata, MusicFile, TrackMetadata};

mod dto;
mod mp3_tags;
mod scan;

fn main() -> Result<()> {
    let dirs = scan::scan_dirs(PathBuf::from_str(".").unwrap())?;
    let mut buffer = String::with_capacity(4096);
    for dir in &dirs {
        dir.render_into(&mut buffer)?;
    }

    if found_info_and_should_abort(&dirs) {
        return Ok(());
    }

    loop {
        match edit::edit_bytes(&buffer) {
            Ok(bytes) => buffer = String::from_utf8(bytes)?,
            Err(err) => {
                eprintln!("Error when editing metadata: {err}");
                if should_abort() {
                    return Err(err.into());
                }
            }
        }

        let parsed_bytes = match dto::deserialize_music_dirs(&buffer) {
            Ok(dirs) => dirs,
            Err(err) => {
                eprintln!("Error when trying to parse the edited tags: {err}");
                match should_abort() {
                    true => return Err(err),
                    false => continue,
                }
            }
        };

        mp3_tags::save_music_dirs(parsed_bytes)?;
        break;
    }

    Ok(())
}

fn found_info_and_should_abort(found_dirs: &[MusicDir]) -> bool {
    println!("=> FOUND FOLLOWING TRACKS:");

    for (
        AlbumMetadata {
            album_artist,
            album,
            ..
        },
        file,
    ) in found_dirs
        .iter()
        .flat_map(|it| it.music_files.iter().map(|file| (&it.metadata, file)))
    {
        let MusicFile {
            file_path,
            metadata: TrackMetadata { title, .. },
        } = file;

        println!("[{album}] {title} ({album_artist})\t[{file_path}]")
    }

    should_abort()
}

fn should_abort() -> bool {
    let mut stdin = std::io::stdin().lock();
    let mut buff = String::new();
    println!("Do you want to continue? [Y/n]");

    loop {
        buff.clear();
        let _ = stdin.read_line(&mut buff);

        match &buff[..] {
            "\n" | "y\n" | "Y\n" => return false,
            "n\n" | "N\n" => return true,
            _ => println!("Type \"y\" or \"n\""),
        }
    }
}
