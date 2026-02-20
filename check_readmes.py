import os

crates_dir = "crates"
crates = [d for d in os.listdir(crates_dir) if os.path.isdir(os.path.join(crates_dir, d)) and d != "vendor"]
missing_readmes = []

for crate in crates:
    readme_path = os.path.join(crates_dir, crate, "README.md")
    if not os.path.exists(readme_path):
        missing_readmes.append(crate)

if missing_readmes:
    print("Missing READMEs in crates:")
    for crate in missing_readmes:
        print(f"- {crate}")
else:
    print("All crates have README.md")
