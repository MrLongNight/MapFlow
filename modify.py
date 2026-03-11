import re

with open('crates/mapmap-io/src/error.rs', 'r') as f:
    error_rs = f.read()

# Fix error.rs: Move impl From<zip::result::ZipError> before #[cfg(test)]
error_rs = re.sub(r'impl From<zip::result::ZipError> for IoError \{\n    fn from\(err: zip::result::ZipError\) -> Self \{\n        IoError::ZipError\(err\.to_string\(\)\)\n    \}\n\}', '', error_rs)

error_rs = re.sub(
    r'(\#\[cfg\(test\)\])',
    r'impl From<zip::result::ZipError> for IoError {\n    fn from(err: zip::result::ZipError) -> Self {\n        IoError::ZipError(err.to_string())\n    }\n}\n\n\1',
    error_rs
)

with open('crates/mapmap-io/src/error.rs', 'w') as f:
    f.write(error_rs)


with open('crates/mapmap-io/src/project.rs', 'r') as f:
    project_rs = f.read()

# Extract the export_project function
match = re.search(r'(/// Exports the application state and media assets to a ZIP archive\..*?Ok\(\(\)\)\n\})', project_rs, flags=re.DOTALL)
if match:
    export_fn = match.group(1)

    # Remove it from its current position
    project_rs = project_rs.replace(export_fn, '')

    # Insert it before #[cfg(test)]
    project_rs = re.sub(
        r'(\#\[cfg\(test\)\])',
        export_fn + r'\n\n\1',
        project_rs
    )

with open('crates/mapmap-io/src/project.rs', 'w') as f:
    f.write(project_rs)
