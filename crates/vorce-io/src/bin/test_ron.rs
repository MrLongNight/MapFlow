use vorce_core::AppState;
use vorce_io::project_format::ProjectFile;

fn main() {
    let app_state = AppState::default();
    let project_file = ProjectFile::new(app_state);
    let s = ron::ser::to_string_pretty(&project_file, ron::ser::PrettyConfig::default()).unwrap();
    let lines: Vec<&str> = s.lines().collect();
    for (i, line) in lines.iter().enumerate() {
        if i > 60 && i < 85 {
            println!("{:02}: {}", i + 1, line);
        }
    }
}
