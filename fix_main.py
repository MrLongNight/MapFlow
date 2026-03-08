import os

with open("crates/mapmap/src/main.rs", "r") as f:
    content = f.read()

content = content.replace("pub mod orchestration;", "/// Orchestration module for managing application state updates\npub mod orchestration;")

with open("crates/mapmap/src/main.rs", "w") as f:
    f.write(content)
