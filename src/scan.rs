use crate::dto::{AlbumMetadata, ImageFile, MusicDir, MusicFile};
use crate::mp3_tags::get_mp3_metadata;
use anyhow::Result;
use std::path::{Path, PathBuf};

pub fn scan_dirs(path: PathBuf) -> Result<Vec<MusicDir>> {
    let mut music_files: Vec<(u32, MusicFile)> = Vec::new();
    let mut subdirs: Vec<MusicDir> = Vec::new();
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
            subdirs.append(&mut scan_dirs(file.path())?);
        }
    }

    let music_files = sorted_by_track_number(music_files);

    Ok(match album_metadata {
        Some(metadata) => [MusicDir {
            path: assert_unicode_path(path),
            metadata,
            music_files,
            image_files,
        }]
        .into_iter()
        .chain(subdirs)
        .collect(),

        None => subdirs,
    })
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
