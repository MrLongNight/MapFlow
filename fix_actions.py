import os

with open("crates/mapmap/src/app/actions.rs", "r") as f:
    content = f.read()

content = content.replace("load_project_file(app, &path);", "let _ = load_project_file(app, &path);")

with open("crates/mapmap/src/app/actions.rs", "w") as f:
    f.write(content)
