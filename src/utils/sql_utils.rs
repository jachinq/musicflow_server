pub fn detail_sql() -> String {
    "SELECT s.id, s.title, 
            ar.name as artist, 
            s.artist_id, 
            al.name as album, 
            s.album_id, 
            s.track_number, s.disc_number, s.duration, s.bit_rate, s.genre,
            s.year, s.content_type, s.file_path as path,
            al.cover_art_path as cover_art
         FROM songs s
         JOIN albums al ON s.album_id = al.id
         JOIN artists ar ON s.artist_id = ar.id".to_string()
}
