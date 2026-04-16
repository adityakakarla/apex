use std::fs::{self, read_dir};

use directories::{self, ProjectDirs};

pub fn is_initialized() -> bool {
    if let Some(proj_dirs) = ProjectDirs::from("com", "Apex", "Apex") {
        let data_dir = proj_dirs.data_dir();
        data_dir.exists()
    } else {
        false
    }
}

pub fn get_courses() -> Vec<String> {
    if let Some(proj_dirs) = ProjectDirs::from("com", "Apex", "Apex") {
        let data_dir = proj_dirs.data_dir();
        read_dir(data_dir)
            .map(|entries| {
                entries
                    .filter_map(|e| e.ok())
                    .filter(|e| e.path().is_dir())
                    .filter_map(|e| e.file_name().into_string().ok())
                    .collect()
            })
            .unwrap_or_default()
    } else {
        Vec::new()
    }
}

pub fn get_course_sections(course: String) -> Vec<String> {
    if let Some(proj_dirs) = ProjectDirs::from("com", "Apex", "Apex") {
        let data_dir = proj_dirs.data_dir();
        let index_path = data_dir.join(course).join("index.json");
        let contents = std::fs::read_to_string(index_path).unwrap_or_default();
        let json: serde_json::Value = serde_json::from_str(&contents).unwrap_or_default();
        json["order"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default()
    } else {
        Vec::new()
    }
}

pub fn get_claude_command() -> String {
    if let Some(proj_dirs) = ProjectDirs::from("com", "Apex", "Apex") {
        let data_dir = proj_dirs.data_dir();
        format!("cd \"{}\" && claude", data_dir.display().to_string())
    } else {
        String::new()
    }
}

pub fn initialize_directory() -> bool {
    if let Some(proj_dirs) = ProjectDirs::from("com", "Apex", "Apex") {
        let data_dir = proj_dirs.data_dir();

        if fs::create_dir_all(data_dir).is_err() {
            return false;
        }

        let file_path = data_dir.join("CLAUDE.md");
        fs::write(
            file_path,
            "
You are an AI agent in charge of building out courses. Each course is a
subdirectory in the directory you are currently in. Within each course there
are multiple other subdirectories, corresponding to course sections. Within
each section, there are markdown files (corresponding to course content) and
json files (corresponding to questions and answers for quizzes).

Markdown must follow commonmark guidelines. Be concise, but powerful. Humans
must enjoy reading it, and it should not feel like a waste of time. Use
cutting-edge teaching principles.

The JSON files should look like:
{'what is 2 + 2': '4', 'what is 3 + 3': '6'}, but using double quotes instead
of single quotes

You should also create index.json files for any new course or course section
you create. It should also be updated for each new markdown/json file created,
otherwise the apps won't function properly.

index.json (for both course and course section directories) should look like:
{
 'order': ['x.md', 'y.json', 'z.md']
}
",
        )
        .is_ok()
    } else {
        false
    }
}
