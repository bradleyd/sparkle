use crate::file_metadata::{self, AgeCategory, FileType, SizeCategory};
use file_format::{FileFormat, Kind};
use mime_guess2::mime;
use std::time::SystemTime;
use std::{fs::Metadata, os::unix::fs::MetadataExt, path::Path};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Category {
    Document,
    Spreadsheet,
    Presentation,
    Pdf,
    Image,
    Screenshot,
    RawPhoto,
    Audio,
    Video,
    Archive,
    Installer,
    DiskImage,
    Code,
    Config,
    Log,
    Text,
    Executable,
    Script,
    SystemFile,
    Unknown,
}

impl Category {
    pub fn as_str(&self) -> &'static str {
        match self {
            Category::Document => "Document",
            Category::Spreadsheet => "Spreadsheet",
            Category::Presentation => "Presentation",
            Category::Pdf => "PDF",
            Category::Image => "Image",
            Category::Screenshot => "Screenshot",
            Category::RawPhoto => "RawPhoto",
            Category::Audio => "Audio",
            Category::Video => "Video",
            Category::Archive => "Archive",
            Category::Installer => "Installer",
            Category::DiskImage => "DiskImage",
            Category::Code => "Code",
            Category::Config => "Config",
            Category::Log => "Log",
            Category::Text => "Text",
            Category::Executable => "Executable",
            Category::Script => "Script",
            Category::SystemFile => "SystemFile",
            Category::Unknown => "Unknown",
        }
    }
}

#[derive(Debug, Clone)]
pub struct DetectResult {
    pub category: Category,
    pub mime_hint: Option<&'static str>,
    pub confidence: u8,          // 0-100
    pub details: Option<String>, // why we think so
}

#[allow(clippy::collapsible_if)]
pub fn detect2(path: &std::path::Path) -> std::io::Result<DetectResult> {
    use std::fs::File;
    use std::io::Read;
    let mut f = File::open(path)?;
    let mut buf = vec![0u8; 32768];
    let n = f.read(&mut buf)?;
    buf.truncate(n);

    // 1) magic: infer
    if let Some(kind) = infer::get(&buf) {
        let mime = kind.mime_type();
        if mime == "application/pdf" {
            return ok(Category::Pdf, Some(mime), 95, "infer:pdf");
        }
        if mime == "image/png" || mime.starts_with("image/") {
            // verify decodability and maybe mark Screenshot later
            return ok(Category::Image, Some(mime), 85, "infer:image");
        }
        if mime == "application/zip" { /* refine below */ }
        // … handle other strong signals
    }

    // 2) container refinement (zip-based)
    if file_format::FileFormat::from_bytes(&buf) == file_format::FileFormat::Zip {
        if let Ok(mut zf) = zip::ZipArchive::new(File::open(path)?) {
            let mut has = |p: &str| -> bool {
                (0..zf.len()).any(|i| {
                    zf.by_index(i)
                        .map(|f| f.name().starts_with(p))
                        .unwrap_or(false)
                })
            };
            if has("word/") {
                return ok(
                    Category::Document,
                    Some("application/vnd.openxmlformats-officedocument.wordprocessingml.document"),
                    96,
                    "zip:docx",
                );
            }
            if has("xl/") {
                return ok(
                    Category::Spreadsheet,
                    Some("application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"),
                    96,
                    "zip:xlsx",
                );
            }
            if has("ppt/") {
                return ok(
                    Category::Presentation,
                    Some(
                        "application/vnd.openxmlformats-officedocument.presentationml.presentation",
                    ),
                    96,
                    "zip:pptx",
                );
            }
            if has("AndroidManifest.xml") && has("classes.dex") {
                return ok(
                    Category::Executable,
                    Some("application/vnd.android.package-archive"),
                    92,
                    "zip:apk",
                );
            }
            if has("META-INF/MANIFEST.MF") {
                return ok(
                    Category::Executable,
                    Some("application/java-archive"),
                    85,
                    "zip:jar",
                );
            }
            return ok(
                Category::Archive,
                Some("application/zip"),
                80,
                "zip:generic",
            );
        }
    }

    // 3) PDF quick check
    if buf.starts_with(b"%PDF-") {
        return ok(Category::Pdf, Some("application/pdf"), 99, "magic:%PDF");
    }

    // 4) OLE/CFB (old Office / MSI)
    if buf.starts_with(&[0xD0, 0xCF, 0x11, 0xE0]) {
        // You can refine to MSI vs DOC via deeper parse; here just bucket:
        return ok(
            Category::Document,
            Some("application/x-ole-storage"),
            70,
            "ole/cfb",
        );
    }

    // 5) image verify (optional, only if extension/magic hints)
    // use `image` + `kamadak-exif` here if needed…

    // 6) text vs binary
    if content_inspector::inspect(&buf).is_text() {
        // shebang detection
        if buf.starts_with(b"#!") {
            return ok(Category::Script, Some("text/plain"), 85, "shebang");
        }
        // cheap code/config heuristics
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or_default()
            .to_ascii_lowercase();
        match ext.as_str() {
            "rs" | "go" | "py" | "js" | "ts" | "java" | "c" | "cpp" | "sh" => {
                return ok(Category::Code, Some("text/plain"), 80, "ext:code");
            }
            "toml" | "yaml" | "yml" | "json" | "ini" => {
                return ok(Category::Config, Some("text/plain"), 80, "ext:config");
            }
            "log" => return ok(Category::Log, Some("text/plain"), 70, "ext:log"),
            _ => return ok(Category::Text, Some("text/plain"), 60, "text"),
        }
    }

    // 7) last-resort: tree_magic_mini as tie-breaker
    let m = tree_magic_mini::from_u8(&buf);
    if m.starts_with("image/") {
        return ok(Category::Image, Some(m), 70, "tree_magic");
    }
    if m == "application/x-iso9660-image" {
        return ok(Category::DiskImage, Some(m), 75, "tree_magic");
    }
    if m == "application/x-xar" {
        return ok(Category::Installer, Some(m), 85, "xar:pkg");
    }

    ok(Category::Unknown, None, 0, "fallback")
}

fn ok(
    cat: Category,
    mime: impl Into<Option<&'static str>>,
    c: u8,
    why: &str,
) -> std::io::Result<DetectResult> {
    Ok(DetectResult {
        category: cat,
        mime_hint: mime.into(),
        confidence: c,
        details: Some(why.to_string()),
    })
}

pub fn get_file_type(f: &Path) -> crate::file_metadata::FileType {
    let extension_result = guess_mime(f);
    if extension_result != FileType::Unknown {
        return extension_result;
    }

    let mime = mime_guess2::from_path(f);
    if mime.is_empty() {
        let fmt = FileFormat::from_file(f);
        match fmt {
            Ok(ff) => match ff.kind() {
                Kind::Document => FileType::Document,
                Kind::Image => FileType::Image,
                Kind::Other => FileType::Unknown,
                Kind::Archive => FileType::Archive,
                _ => FileType::Unknown,
            },
            Err(_) => FileType::Unknown,
        }
    } else {
        let mime = mime.first_or_octet_stream();
        match mime {
            m if m == mime::IMAGE_GIF => FileType::Image,
            m if m == mime::IMAGE_BMP => FileType::Image,
            m if m == mime::IMAGE_JPEG => FileType::Image,
            m if m == mime::IMAGE_SVG => FileType::Image,
            m if m == mime::APPLICATION_PDF => FileType::Document,
            m if m == mime::APPLICATION_JAVASCRIPT => FileType::Code,
            m if m == mime::TEXT_PLAIN => FileType::Text,
            _ => FileType::Unknown,
        }
    }
}

fn guess_mime(path: &Path) -> FileType {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    match ext.as_str() {
        "java" | "rs" | "rb" | "ex" | "go" | "js" => FileType::Code,
        "md" => FileType::Document,
        "yml" | "yaml" | "toml" => FileType::Configuration,
        "txt" => FileType::Text,
        _ => FileType::Unknown,
    }
}

pub fn get_file_size_category(metadata: &Metadata) -> crate::file_metadata::SizeCategory {
    let fsize = metadata.size();
    match fsize {
        0..1024 => SizeCategory::Tiny,
        1024..=1_048_576 => SizeCategory::Small,
        1_048_577..=104_857_600 => SizeCategory::Medium,
        104_857_601..=1_073_741_824 => SizeCategory::Medium,
        1_073_741_825..=u64::MAX => SizeCategory::Medium,
    }
}

fn system_time_to_days(time: SystemTime) -> u64 {
    let now = SystemTime::now();
    let age = now.duration_since(time).expect("Time went backwards");
    age.as_secs() / 86_400 // 86,400 seconds in a day
}

pub fn get_age_category(metadata: &Metadata) -> crate::file_metadata::AgeCategory {
    if let Ok(fdate) = metadata.modified() {
        let days = system_time_to_days(fdate);
        match days {
            0 => AgeCategory::Recent,
            1..=7 => AgeCategory::Week,
            8..=30 => AgeCategory::Month,
            31..=365 => AgeCategory::Year,
            366..=u64::MAX => AgeCategory::Old,
        }
    } else {
        file_metadata::AgeCategory::Old
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_get_file_type_by_extension() {
        assert_eq!(get_file_type(Path::new("test.java")), FileType::Code);
        assert_eq!(get_file_type(Path::new("test.rs")), FileType::Code);
        assert_eq!(get_file_type(Path::new("test.js")), FileType::Code);
        assert_eq!(get_file_type(Path::new("test.go")), FileType::Code);
        assert_eq!(get_file_type(Path::new("test.rb")), FileType::Code);
        assert_eq!(get_file_type(Path::new("test.ex")), FileType::Code);
    }

    #[test]
    fn test_get_file_type_documents() {
        assert_eq!(get_file_type(Path::new("readme.md")), FileType::Document);
    }

    #[test]
    fn test_get_file_type_configuration() {
        assert_eq!(
            get_file_type(Path::new("config.yml")),
            FileType::Configuration
        );
        assert_eq!(
            get_file_type(Path::new("config.yaml")),
            FileType::Configuration
        );
        assert_eq!(
            get_file_type(Path::new("config.toml")),
            FileType::Configuration
        );
    }

    #[test]
    fn test_get_file_type_text() {
        assert_eq!(get_file_type(Path::new("notes.txt")), FileType::Text);
    }

    #[test]
    fn test_get_file_type_unknown() {
        assert_eq!(get_file_type(Path::new("file.xyz")), FileType::Unknown);
        assert_eq!(get_file_type(Path::new("no_extension")), FileType::Unknown);
    }

    #[test]
    fn test_get_file_type_case_insensitive() {
        assert_eq!(get_file_type(Path::new("TEST.JAVA")), FileType::Code);
        assert_eq!(
            get_file_type(Path::new("Config.YML")),
            FileType::Configuration
        );
    }

    #[test]
    fn test_get_file_size_category_with_test_files() {
        use std::fs;
        use tempfile::NamedTempFile;

        let tiny_file = NamedTempFile::new().unwrap();
        fs::write(&tiny_file, vec![0u8; 512]).unwrap();
        let tiny_metadata = fs::metadata(tiny_file.path()).unwrap();
        assert!(matches!(
            get_file_size_category(&tiny_metadata),
            SizeCategory::Tiny
        ));

        let small_file = NamedTempFile::new().unwrap();
        fs::write(&small_file, vec![0u8; 5000]).unwrap();
        let small_metadata = fs::metadata(small_file.path()).unwrap();
        assert!(matches!(
            get_file_size_category(&small_metadata),
            SizeCategory::Small
        ));

        let medium_file = NamedTempFile::new().unwrap();
        fs::write(&medium_file, vec![0u8; 5_000_000]).unwrap();
        let medium_metadata = fs::metadata(medium_file.path()).unwrap();
        assert!(matches!(
            get_file_size_category(&medium_metadata),
            SizeCategory::Medium
        ));
    }
}
