#! /usr/bin/env python3
import os

def fix_file(file_path):
    exts = ('.rs', '.md', '.toml', '.yaml', '.yml', '.wgsl', '.json', '.ps1', '.sh', '.bat')
    if file_path.endswith(exts):
        try:
            with open(file_path, 'rb') as f:
                content = f.read()

            # Convert CRLF (\r\n) to LF (\n)
            content = content.replace(b'\r\n', b'\n')

            # Split into lines
            lines = content.split(b'\n')
            # Strip trailing whitespace
            fixed_lines = [line.rstrip() for line in lines]

            # Rejoin with LF and ensure single newline at EOF
            new_content = b'\n'.join(fixed_lines).rstrip() + b'\n'

            if content != new_content:
                with open(file_path, 'wb') as f:
                    f.write(new_content)
                return True
        except Exception as e:
            print(f"Error fixing {file_path}: {e}")
    return False

def main():
    fixed_count = 0
    for root, dirs, files in os.walk('.'):
        if any(skip in root for skip in ['.git', 'target', 'vcpkg', 'vcpkg_installed']):
            continue
        for file in files:
            if fix_file(os.path.join(root, file)):
                fixed_count += 1
    print(f"Fixed {fixed_count} files.")

if __name__ == "__main__":
    main()
