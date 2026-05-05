use anyhow::{Context, Result};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::collections::HashMap;
use std::fs::{self, File, OpenOptions};
use std::io::{BufReader, BufWriter, Read, Seek, SeekFrom, Write};
use std::path::Path;

pub struct KvStore {
    file: File,
    // Maps key -> record_start_offset
    index: HashMap<String, u64>,
}

impl KvStore {
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        
        // Ensure the parent directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).context("Failed to create data directory")?;
        }

        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(path)
            .context("Failed to open log file")?;

        let mut store = KvStore {
            file,
            index: HashMap::new(),
        };

        store.load_index()?;
        Ok(store)
    }

    fn load_index(&mut self) -> Result<()> {
        let mut reader = BufReader::new(&self.file);
        let mut offset = 0;

        loop {
            // Try to read key length
            let key_len = match reader.read_u32::<LittleEndian>() {
                Ok(len) => len as u64,
                Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break, // End of file
                Err(e) => return Err(e.into()),
            };

            // Read value length
            let val_len = reader.read_u32::<LittleEndian>().context("Failed to read value length")?;

            // Read key
            let mut key_bytes = vec![0u8; key_len as usize];
            reader.read_exact(&mut key_bytes).context("Failed to read key")?;
            let key = String::from_utf8(key_bytes).context("Key contains invalid UTF-8")?;

            // Check for tombstone
            if val_len == u32::MAX {
                self.index.remove(&key);
                offset += 4 + 4 + key_len;
                continue;
            }

            // Store offset in index
            self.index.insert(key, offset);

            // Skip value
            reader.seek_relative(val_len as i64).context("Failed to skip value")?;

            // Update offset to next record
            offset += 4 + 4 + key_len + (val_len as u64);
        }

        Ok(())
    }

    pub fn remove(&mut self, key: String) -> Result<()> {
        let mut writer = BufWriter::new(&self.file);
        // Seek to end to append
        writer.seek(SeekFrom::End(0)).context("Failed to seek to end")?;

        let key_bytes = key.as_bytes();

        // Write header
        writer.write_u32::<LittleEndian>(key_bytes.len() as u32).context("Failed to write key len")?;
        writer.write_u32::<LittleEndian>(u32::MAX).context("Failed to write tombstone val len")?;
        
        // Write key only
        writer.write_all(key_bytes).context("Failed to write key")?;
        
        writer.flush().context("Failed to flush writes")?;
        drop(writer);
        self.file.sync_all().context("Failed to sync writes")?;

        // Remove from index
        self.index.remove(&key);

        Ok(())
    }

    pub fn set(&mut self, key: String, value: String) -> Result<()> {
        let mut writer = BufWriter::new(&self.file);
        // Seek to end to append
        let current_offset = writer.seek(SeekFrom::End(0)).context("Failed to seek to end")?;

        let key_bytes = key.as_bytes();
        let val_bytes = value.as_bytes();

        // Write header
        writer.write_u32::<LittleEndian>(key_bytes.len() as u32).context("Failed to write key len")?;
        writer.write_u32::<LittleEndian>(val_bytes.len() as u32).context("Failed to write val len")?;
        
        // Write data
        writer.write_all(key_bytes).context("Failed to write key")?;
        writer.write_all(val_bytes).context("Failed to write value")?;
        
        writer.flush().context("Failed to flush writes")?;
        drop(writer);
        self.file.sync_all().context("Failed to sync writes")?;

        // Update index
        self.index.insert(key, current_offset);

        Ok(())
    }

    pub fn get(&mut self, key: &str) -> Result<Option<String>> {
        let offset = match self.index.get(key) {
            Some(&o) => o,
            None => return Ok(None),
        };

        let mut reader = BufReader::new(&self.file);
        reader.seek(SeekFrom::Start(offset)).context("Failed to seek to record")?;

        // Read header
        let key_len = reader.read_u32::<LittleEndian>().context("Failed to read key len")? as u64;
        let val_len = reader.read_u32::<LittleEndian>().context("Failed to read val len")? as u64;

        // Skip key (we know it matches, but strict impl would check)
        reader.seek_relative(key_len as i64).context("Failed to skip key")?;

        // Read value
        let mut val_bytes = vec![0u8; val_len as usize];
        reader.read_exact(&mut val_bytes).context("Failed to read value")?;
        
        let value = String::from_utf8(val_bytes).context("Value contains invalid UTF-8")?;

        Ok(Some(value))
    }

    pub fn keys(&self) -> Vec<String> {
        let mut keys = self.index.keys().cloned().collect::<Vec<_>>();
        keys.sort();
        keys
    }
}
