# Append-Only Key-Value Store 🦀

A robust, single-node key-value store built in Rust. It utilizes an **append-only log** for data persistence and an **in-memory hash map** for fast retrieval. This project serves as an educational implementation of storage engine basics, demonstrating crash safety and file handling.

---

## 🚀 Features

- **Persistent Storage**: All data is saved to disk in `data/kv.log`.
- **Fast Reads**: An in-memory `HashMap` index maps keys directly to their file offsets, ensuring O(1) lookup time (excluding disk I/O).
- **Crash Recovery**: On startup, the store replays the log to rebuild the index, ensuring data survives restarts.
- **CLI Interface**: Simple command-line tools to interact with the database.
- **Append-Only Design**: Writes are always appended to the end of the file, optimizing for write performance and simplifying data integrity.

---

## 🛠️ Installation & Usage

### Prerequisites
- [Rust & Cargo](https://www.rust-lang.org/tools/install) installed.

### Build
To compile the project:
```sh
cargo build --release
```

### Usage
Run the binary via `cargo run` (for development) or the built executable.

#### 1. Set a Key-Value Pair
Stores a value associated with a key. If the key exists, it updates the index to point to the new entry (old entries remain in the log but are ignored).
```sh
cargo run -- set <KEY> <VALUE>
# Example
cargo run -- set user:101 "Alice Wonderland"
```

#### 2. Get a Value
Retrieves the value for a given key.
```sh
cargo run -- get <KEY>
# Example
cargo run -- get user:101
# Output: Alice Wonderland
```

#### 3. Remove a Key
Removes a key from the store. This appends a "tombstone" record to the log, effectively hiding the key from future lookups.
```sh
cargo run -- rm <KEY>
# Example
cargo run -- rm user:101
```

---

## 🏗️ Architecture & Implementation

### Project Structure
- **`src/main.rs`**: The entry point. It uses `clap` to parse command-line arguments and dispatches commands to the `KvStore`.
- **`src/store.rs`**: The core storage engine.
    - Manages the `KvStore` struct.
    - Handles file I/O (reading/writing to `data/kv.log`).
    - Maintains the in-memory `index`.
- **`src/utils.rs`**: A module for utility functions and future extensions (currently a placeholder).

### Data Format (`data/kv.log`)
The storage file is a continuous sequence of binary records. Each record has the following format:

| Field | Type | Description |
|-------|------|-------------|
| **Key Length** | `u32` | Length of the key in bytes (Little Endian). |
| **Value Length** | `u32` | Length of the value in bytes (Little Endian). |
| **Key** | `[u8]` | The key data (UTF-8 string). |
| **Value** | `[u8]` | The value data (UTF-8 string). |

### How It Works
1.  **`set(key, value)`**:
    - Seeks to the end of the file.
    - Writes the header (`key_len`, `val_len`) and then the bodies (`key`, `value`).
    - Updates the in-memory `index` with the `key` and the **start offset** of this new record.
2.  **`get(key)`**:
    - Looks up the `key` in the in-memory `index` to find the file offset.
    - Seeks to that offset in the file.
    - Reads the lengths and then reads the value directly (skipping the key for performance).
3.  **`load_index()` (Startup)**:
    - Reads the entire log file from beginning to end.
    - Parses every record's key and length.
    - Populates the `index` map.
    - This ensures the in-memory state matches the persistent log.

---

## 📦 Dependencies

The project relies on a few key crates to ensure robustness and code clarity:

- **`anyhow`**: Provides flexible and easy-to-use error handling. It allows us to attach context to errors (e.g., "Failed to open log file") for better debugging.
- **`clap`**: A powerful command-line argument parser. We use it with the `derive` feature to define our CLI structure (`Set`, `Get`) directly from Rust structs and enums.
- **`byteorder`**: Helpers for reading/writing primitive numbers (like `u32`) in a specific endianness (Little Endian), ensuring portability of the data file across different systems.

---

## 📚 Why Use This?

- **Educational**: Perfect for understanding how databases like Bitcask or log-structured merge-trees (LSM) start.
- **Simple**: Minimal codebase makes it easy to read and modify.
- **Safe**: Rust's type system and explicit error handling prevent common crashes and data races.