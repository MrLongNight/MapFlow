import os

with open("crates/mapmap/src/orchestration/mod.rs", "r") as f:
    content = f.read()

content = content.replace("pub mod evaluation;", "/// Evaluates node logic\npub mod evaluation;")
content = content.replace("pub mod media;", "/// Syncs media handles\npub mod media;")
content = content.replace("pub mod node_logic;", "/// Core application node logic updates\npub mod node_logic;")

with open("crates/mapmap/src/orchestration/mod.rs", "w") as f:
    f.write(content)
