use crate::dto::{AlbumMetadata, ImageFile, Metadata, MusicDir};
use anyhow::{Ok, Result};
use chrono::Datelike;
use id3::frame::Picture;
use id3::{no_tag_ok, Tag, TagLike};
use image::io::Reader as ImageReader;
use std::io::Cursor;
use std::path::{Path, PathBuf};

pub fn get_mp3_metadata(path: impl AsRef<Path>, track: u32) -> Result<Metadata> {
    let tag = no_tag_ok(Tag::read_from_path(path))?.unwrap_or(Tag::new());

    fn fix_toml_string(it: String) -> String {
        it.replace(char::from(0), "").replace('"', "\\\"")
    }

    Ok(Metadata {
        artist: tag.artist().map(str::to_owned).map(fix_toml_string),
        title: tag
            .title()
            .map(str::to_owned)
            .map(fix_toml_string)
            .unwrap_or(String::new()),
        track,
        album: tag
            .album()
            .map(str::to_owned)
            .map(fix_toml_string)
            .unwrap_or(String::new()),
        album_artist: tag
            .artist()
            .map(str::to_owned)
            .map(fix_toml_string)
            .unwrap_or(String::new()),
        year: tag.year().unwrap_or(
            tag.date_released().map(|it| it.year).unwrap_or(
                tag.original_date_released().map(|it| it.year).unwrap_or(
                    tag.date_recorded()
                        .map(|it| it.year)
                        .unwrap_or(chrono::Utc::now().year()),
                ),
            ),
        ) as u32,
    })
}

pub fn save_music_dirs(dirs: Vec<MusicDir>) -> Result<()> {
    for dir in dirs {
        let AlbumMetadata {
            ref album,
            ref album_artist,
            year,
        } = dir.metadata;

        let image = dir.image_files.into_iter().find(|it| it.use_as_cover).map(
            |ImageFile { file_path, .. }| {
                let mut path: PathBuf = dir.path.as_str().into();
                path.push(file_path);
                path
            },
        );

        for (index, file) in dir.music_files.into_iter().enumerate() {
            let metadata = Metadata {
                track: (index as u32) + 1,
                title: file.metadata.title,
                artist: file.metadata.artist,
                album: album.clone(),
                album_artist: album_artist.clone(),
                year,
            };

            let mut path: PathBuf = dir.path.as_str().into();
            path.push(file.file_path);

            save_metadata_for_file(path.to_str().unwrap(), metadata, image.as_deref())?;
        }
    }

    Ok(())
}

fn save_metadata_for_file(path: &str, metadata: Metadata, image: Option<&Path>) -> Result<()> {
    let mut tag = id3::no_tag_ok(Tag::read_from_path(path))?.unwrap_or(Tag::new());

    let artist = match metadata.artist.as_ref().map(String::as_str) {
        Some("") | None => metadata.album_artist.as_str(),
        Some(a) => a,
    };

    tag.set_artist(artist);
    tag.set_title(&metadata.title);
    tag.set_track(metadata.track);
    tag.set_album(&metadata.album);
    tag.set_album_artist(&metadata.album_artist);

    tag.set_year(metadata.year as i32);
    tag.set_date_released(id3::Timestamp {
        year: metadata.year as i32,
        ..Default::default()
    });
    tag.set_original_date_released(id3::Timestamp {
        year: metadata.year as i32,
        ..Default::default()
    });
    tag.set_date_recorded(id3::Timestamp {
        year: metadata.year as i32,
        ..Default::default()
    });

    if let Some(image_path) = image {
        let picture = prepare_tag_picture(image_path)?;
        tag.add_frame(picture);
    }

    tag.write_to_path(path, tag.version())?;
    Ok(())
}

fn prepare_tag_picture(image_path: &Path) -> Result<Picture> {
    let img = ImageReader::open(image_path)?.decode()?;
    let img = img.resize_to_fill(512, 512, image::imageops::FilterType::Triangle);
    let mut buff = Vec::<u8>::with_capacity(4096);
    img.write_to(&mut Cursor::new(&mut buff), image::ImageOutputFormat::Png)?;

    Ok(Picture {
        mime_type: "image/jpg".into(),
        picture_type: id3::frame::PictureType::CoverFront,
        description: "Cover".into(),
        data: buff,
    })
}
