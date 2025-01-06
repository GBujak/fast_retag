# fast_retag

Set or update mp3 metadata with your `$EDITOR`.

This program:

1. Recursively scans the current directory for directories with `.mp3` files.
2. Generates a `.toml` config containing all the current tags formatted to be
   easy to change in vim (but inferior editors are also supported).
3. You change the `.toml` config in your `$EDITOR`.
4. The tags are updated on the `.mp3` files.

The program also finds `.jpg` and `.png` files in the directories with `.mp3`
files and lets you choose one of them as the cover image. The image will be
rescaled to a `512x512px` `JPEG` and embedded into the `.mp3` files.

You can change the tags and cover images for multiple directories at the same
time.

Example:

```toml
["./Gza - Liquid Swords"]
path = "./Gza - Liquid Swords"
metadata = { album = "Liquid Swords", album_artist = "Gza", year = 1995 }

# Order in this list determines track number
music_files = [
     { metadata = { title = "Liquid Swords", artist = "Gza" }, file_path = "01 Gza, Liquid Swords.mp3" }, 
     { metadata = { title = "Duel of the Iron Mic", artist = "Gza" }, file_path = "02 Gza, Duel Of The Iron Mic (Feat. Dreddy Kruger).mp3" }, 
     { metadata = { title = "Living in the World Today", artist = "Gza" }, file_path = "03 Gza, Living In The World Today.mp3" }, 
     { metadata = { title = "Gold", artist = "Gza" }, file_path = "04 Gza, Gold.mp3" }, 
     { metadata = { title = "Cold World", artist = "Gza" }, file_path = "05 Gza, Cold World (Feat. Inspectah Deck & Life).mp3" }, 
     { metadata = { title = "Labels", artist = "Gza" }, file_path = "06 Gza, Labels.mp3" }, 
     { metadata = { title = "4th Chamber", artist = "Gza" }, file_path = "07 Gza, 4th Chamber (Feat. Ghostface Killah, Killah Priest & Rza).mp3" }, 
     { metadata = { title = "Shadowboxin'", artist = "Gza" }, file_path = "08 Gza, Shadowboxin' (Feat. Method Man).mp3" }, 
     { metadata = { title = "Hell's Wind Staff (skit) Killah Hills 10304", artist = "Gza" }, file_path = "09 Gza, Hell's Wind Staff (skit) (Feat. Killah Priest & Dreddy Kruger) , Killah Hills 10304.mp3" }, 
     { metadata = { title = "Investigative Reports", artist = "Gza" }, file_path = "10 Gza, Investigative Reports (Feat. Raekwon, Ghostface Killah '& U-God).mp3" }, 
     { metadata = { title = "Swordsman", artist = "Gza" }, file_path = "11 Gza, Swordsman (Feat. Killah Priest).mp3" }, 
     { metadata = { title = "I Gotcha Back", artist = "Gza" }, file_path = "12 Gza, I Gotcha Back.mp3" }, 
     { metadata = { title = "B.I.B.L.E.", artist = "Gza" }, file_path = "13 Gza, B.I.B.L.E. (Basic Instructions Before Leaving Earth) (Feat. Killah Priest).mp3" }, 
    
]

# If tracks have pictures already, leaving all as false will not change them
image_files = [
     { use_as_cover = false, file_path = "cover.jpg" }, 
    
]
```
