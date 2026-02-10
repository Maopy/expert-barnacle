use gpui_component::IconName;

/// Get the appropriate icon for a file based on its extension.
pub fn icon_for_file(extension: Option<&str>, is_directory: bool) -> IconName {
    if is_directory {
        return IconName::Folder;
    }

    match extension {
        Some(ext) => match ext {
            // Documents
            "pdf" => IconName::FileText,
            "doc" | "docx" | "odt" | "rtf" => IconName::FileText,
            "xls" | "xlsx" | "ods" | "csv" => IconName::FileSpreadsheet,
            "ppt" | "pptx" | "odp" => IconName::Presentation,
            "txt" | "md" | "markdown" => IconName::FileText,

            // Code
            "rs" | "py" | "js" | "ts" | "jsx" | "tsx" | "go" | "java" | "c" | "cpp" | "h"
            | "hpp" | "rb" | "php" | "swift" | "kt" | "scala" | "sh" | "bash" | "zsh"
            | "fish" | "ps1" | "bat" | "cmd" => IconName::FileCode,
            "html" | "htm" | "css" | "scss" | "sass" | "less" => IconName::FileCode,
            "json" | "yaml" | "yml" | "toml" | "xml" | "ini" | "cfg" | "conf" => {
                IconName::FileCode
            }

            // Images
            "jpg" | "jpeg" | "png" | "gif" | "bmp" | "svg" | "webp" | "ico" | "tiff"
            | "tif" => IconName::Image,

            // Audio
            "mp3" | "wav" | "flac" | "aac" | "ogg" | "wma" | "m4a" => IconName::Music,

            // Video
            "mp4" | "avi" | "mkv" | "mov" | "wmv" | "flv" | "webm" | "m4v" => IconName::Video,

            // Archives
            "zip" | "tar" | "gz" | "bz2" | "xz" | "7z" | "rar" | "zst" => IconName::Archive,

            // Default
            _ => IconName::File,
        },
        None => IconName::File,
    }
}

/// Get the icon for an open directory.
pub fn icon_for_open_directory() -> IconName {
    IconName::FolderOpen
}
