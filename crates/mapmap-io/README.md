# MapFlow IO (`mapmap-io`)

Input/Output systems for MapFlow, including file formats and network streams.

## Features

- **Project Format**: Serialization and deserialization of `.mflow` project files.
- **NDI**: Network Device Interface integration for video streaming.
- **Spout**: Texture sharing on Windows (via `mapmap-io/spout`).

## Usage

```rust
use mapmap_io::project::Project;

// Load a project
let project = Project::load("show.mflow")?;
```
