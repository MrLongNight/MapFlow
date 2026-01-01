# mapmap-io

Input/Output, Serialization, and Project Management.

## Overview
This crate handles file I/O, project serialization, and external stream protocols like NDI and Spout.

## Features

- **Project Files**:
  - Save/Load functionality for MapFlow projects (`.mflow`).
  - Serialization of the entire application state.
  - Version handling and validation.

- **Video I/O Presets**:
  - **NDI**: Network Device Interface integration for video over IP (Send/Receive).
  - **Spout**: Low-latency GPU texture sharing on Windows (Send/Receive).

- **Format Management**:
  - Definitions for supported video modes and resolutions.
