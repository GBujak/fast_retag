use anyhow::{anyhow, Result};
use askama::Template;
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Template)]
#[template(
    ext = "txt",
    source = r#"
["{{ path }}"]
path = "{{ path }}"
metadata = { album = "{{ metadata.album }}", album_artist = "{{ metadata.album_artist }}", year = {{ metadata.year }} }

# Order in this list determines track number
music_files = [
    {% for track in music_files %} { metadata = { title = "{{track.metadata.title}}", artist = "{{crate::dto::get_option_str(track.metadata.artist)}}" }, file_path = "{{ track.file_path }}" },
    {% endfor %}
]

# If tracks have pictures already, leaving all as false will not change them
image_files = [
    {% for image in image_files %} { use_as_cover = {{ image.use_as_cover }}, file_path = "{{ image.file_path }}" },
    {% endfor %}
]
"#
)]
pub struct MusicDir {
    pub path: String,
    pub metadata: AlbumMetadata,
    pub music_files: Vec<MusicFile>,
    pub image_files: Vec<ImageFile>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct ImageFile {
    pub file_path: String,
    pub use_as_cover: bool,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct MusicFile {
    pub file_path: String,
    pub metadata: TrackMetadata,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct Metadata {
    pub title: String,
    pub track: u32,
    pub artist: Option<String>,
    pub album: String,
    pub album_artist: String,
    pub year: u32,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct TrackMetadata {
    pub title: String,
    pub artist: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct AlbumMetadata {
    pub album: String,
    pub album_artist: String,
    pub year: u32,
}

impl Metadata {
    pub fn into_track_and_metadatas(self) -> (u32, TrackMetadata, AlbumMetadata) {
        let Metadata {
            title,
            track,
            artist,
            album,
            album_artist,
            year,
        } = self;
        (
            track,
            TrackMetadata { title, artist },
            AlbumMetadata {
                album,
                album_artist,
                year,
            },
        )
    }
}

pub fn deserialize_music_dirs(
    str_value: &str,
    previous_dirs: &[MusicDir],
) -> Result<Vec<MusicDir>> {
    #[derive(Deserialize)]
    struct Toml(HashMap<PathBuf, MusicDir>);

    let res = toml::from_str::<Toml>(str_value)?.0;
    validate(previous_dirs, &res)?;

    Ok(res.into_values().collect())
}

fn validate(previous_dirs: &[MusicDir], res: &HashMap<PathBuf, MusicDir>) -> Result<()> {
    if res.len() != previous_dirs.len() {
        return Err(anyhow!("Can't add or delete elements in the list"));
    }

    if res.iter().map(|it| &it.1.path).collect::<HashSet<_>>()
        != previous_dirs.iter().map(|it| &it.path).collect()
    {
        return Err(anyhow!(
            "Can't change paths of music directories or add new ones in the file!"
        ));
    }

    for music_dir in previous_dirs {
        let from_res = res.iter().find(|it| it.1.path == music_dir.path);
        let from_res = if let Some(it) = from_res {
            it
        } else {
            return Err(anyhow!(
                "Could not find music dir with path [{path}]. If it was removed, that's not supported!",
                path = music_dir.path,
            ));
        };

        if music_dir
            .music_files
            .iter()
            .map(|it| &it.file_path)
            .collect::<HashSet<_>>()
            != from_res
                .1
                .music_files
                .iter()
                .map(|it| &it.file_path)
                .collect()
        {
            return Err(anyhow!(
                "Music file file paths in directory [{directory}] have changed. That is not supported! \
                You can only rearrange the elements and change the metadata but you can't add new elements or remove them",
                directory = music_dir.path,
            ));
        }
    }

    Ok(())
}

fn get_option_str(val: &Option<String>) -> &str {
    match val {
        Some(ref v) => v,
        None => "",
    }
}
