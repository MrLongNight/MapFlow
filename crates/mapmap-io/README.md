# mapmap-io

**mapmap-io** handles Input/Output operations related to project files and video streaming protocols.

## Key Features

*   **Project I/O**: Serialization and deserialization of MapFlow project files.
*   **NDI (Network Device Interface)**:
    *   `NdiReceiver`: Receiving video streams from the network.
    *   `NdiSender`: Sending video output to the network.
*   **Spout**: (Windows only) Inter-process video sharing.
*   **File Formats**: Definitions for `.mflow` and legacy formats.

## Features Flags

*   `ndi`: Enables NDI support.

## Modules

*   `project`: Project file management.
*   `ndi`: NDI integration.
*   `spout`: Spout integration.
