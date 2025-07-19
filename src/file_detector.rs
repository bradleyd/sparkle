use crate::file_metadata::{self, AgeCategory, FileType, SizeCategory};
use file_format::{FileFormat, Kind};
use mime_guess2::mime;
use std::time::SystemTime;
use std::{fs::Metadata, os::unix::fs::MetadataExt, path::Path};

pub fn get_file_type(f: &Path) -> crate::file_metadata::FileType {
    //println!("fmt {:?}", f);
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
        //println!("mime guess {}", mime.first_or_octet_stream());
        let mime = mime.first_or_octet_stream();
        let result = match mime {
            m if m == mime::IMAGE_GIF => FileType::Image,
            m if m == mime::IMAGE_BMP => FileType::Image,
            m if m == mime::IMAGE_JPEG => FileType::Image,
            m if m == mime::IMAGE_JPEG => FileType::Image,
            m if m == mime::IMAGE_SVG => FileType::Image,
            m if m == mime::APPLICATION_PDF => FileType::Document,
            m if m == mime::APPLICATION_JAVASCRIPT => FileType::Code,
            m if m == mime::TEXT_PLAIN => FileType::Text,
            _ => FileType::Unknown,
        };

        if result.eq(&FileType::Unknown) {
            guess_mime(f)
        } else {
            result
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
    //println!("file size category bytes {:?}", fsize);
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
        //println!("file age category days {:?}", days);
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
