//! File formats by extension, and keyboard layouts.
//!
//! ".pdf file format", "png file", "what is a .docx file", "mime type of mp3",
//! "english keyboard layout", "french keyboard, german keyboard", "Dvorak keyboard".

use crate::engine::*;
use regex::Regex;
use std::sync::OnceLock;

macro_rules! re {
    ($s:expr) => {{
        static R: OnceLock<Regex> = OnceLock::new();
        R.get_or_init(|| Regex::new($s).unwrap())
    }};
}

/// extensions | name | developer | year introduced (empty when unclear) | category | MIME type | note
const FORMATS: &[(&str, &str, &str, &str, &str, &str, &str)] = &[
    ("pdf", "Portable Document Format", "Adobe", "1993", "document", "application/pdf", "fixed-layout documents; an open ISO standard (ISO 32000) since 2008"),
    ("docx", "Office Open XML word-processing document", "Microsoft", "2007", "document", "application/vnd.openxmlformats-officedocument.wordprocessingml.document", "zipped XML; the default Microsoft Word format since Word 2007 (ISO/IEC 29500)"),
    ("doc", "Microsoft Word binary document", "Microsoft", "", "document", "application/msword", "the Word format before 2007"),
    ("xlsx", "Office Open XML spreadsheet", "Microsoft", "2007", "spreadsheet", "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet", "zipped XML; the default Microsoft Excel format since Excel 2007"),
    ("xls", "Microsoft Excel binary spreadsheet", "Microsoft", "", "spreadsheet", "application/vnd.ms-excel", "the Excel format before 2007"),
    ("pptx", "Office Open XML presentation", "Microsoft", "2007", "presentation", "application/vnd.openxmlformats-officedocument.presentationml.presentation", "zipped XML; the default PowerPoint format since PowerPoint 2007"),
    ("ppt", "Microsoft PowerPoint binary presentation", "Microsoft", "", "presentation", "application/vnd.ms-powerpoint", "the PowerPoint format before 2007"),
    ("odt", "OpenDocument Text", "OASIS", "2005", "document", "application/vnd.oasis.opendocument.text", "open standard (ISO/IEC 26300) used by LibreOffice"),
    ("ods", "OpenDocument Spreadsheet", "OASIS", "2005", "spreadsheet", "application/vnd.oasis.opendocument.spreadsheet", "open standard (ISO/IEC 26300)"),
    ("rtf", "Rich Text Format", "Microsoft", "1987", "document", "application/rtf", "plain-text markup for formatted documents"),
    ("txt", "plain text", "", "", "text", "text/plain", "unformatted text in some character encoding (today usually UTF-8)"),
    ("csv", "comma-separated values", "", "", "data", "text/csv", "tabular data as text lines, fields separated by commas (RFC 4180)"),
    ("tsv", "tab-separated values", "", "", "data", "text/tab-separated-values", "tabular data as text lines, fields separated by tabs"),
    ("json", "JavaScript Object Notation", "Douglas Crockford", "2001", "data", "application/json", "text format for objects, arrays, strings, numbers, booleans and null (RFC 8259)"),
    ("xml", "Extensible Markup Language", "W3C", "1998", "markup", "application/xml", "tagged, nested text markup"),
    ("yaml;yml", "YAML Ain't Markup Language", "Clark Evans, Ingy döt Net, Oren Ben-Kiki", "2001", "data", "application/yaml", "indentation-based data serialization, a superset of JSON"),
    ("toml", "Tom's Obvious, Minimal Language", "Tom Preston-Werner", "2013", "data", "application/toml", "configuration file format"),
    ("html;htm", "HyperText Markup Language", "Tim Berners-Lee; now WHATWG", "1993", "markup", "text/html", "the markup language of web pages"),
    ("css", "Cascading Style Sheets", "W3C", "1996", "style sheet", "text/css", "styling rules for HTML and XML documents"),
    ("js;mjs", "JavaScript source", "Netscape (Brendan Eich)", "1995", "source code", "text/javascript", "the scripting language of web browsers"),
    ("md;markdown", "Markdown", "John Gruber", "2004", "markup", "text/markdown", "lightweight plain-text formatting"),
    ("tex", "TeX / LaTeX source", "Donald Knuth (TeX); Leslie Lamport (LaTeX)", "1978", "markup", "application/x-tex", "typesetting source, compiled to PDF or DVI"),
    ("png", "Portable Network Graphics", "PNG Development Group", "1996", "raster image", "image/png", "lossless compression; supports transparency; created as a patent-free replacement for GIF"),
    ("jpg;jpeg;jpe", "JPEG image", "Joint Photographic Experts Group", "1992", "raster image", "image/jpeg", "lossy compression based on the discrete cosine transform; the usual format for photographs"),
    ("gif", "Graphics Interchange Format", "CompuServe", "1987", "raster image", "image/gif", "up to 256 colours per frame; lossless LZW compression; supports animation"),
    ("bmp", "bitmap image", "Microsoft", "", "raster image", "image/bmp", "usually uncompressed device-independent bitmap"),
    ("tif;tiff", "Tagged Image File Format", "Aldus", "1986", "raster image", "image/tiff", "flexible container used in printing, scanning and archiving"),
    ("svg", "Scalable Vector Graphics", "W3C", "2001", "vector image", "image/svg+xml", "XML-based vector graphics"),
    ("webp", "WebP", "Google", "2010", "raster image", "image/webp", "lossy and lossless compression for the web"),
    ("heic;heif", "High Efficiency Image File Format", "MPEG", "2015", "raster image", "image/heic", "HEVC-compressed images; the default photo format on iPhones since iOS 11"),
    ("avif", "AV1 Image File Format", "Alliance for Open Media", "2019", "raster image", "image/avif", "AV1-compressed images"),
    ("ico", "ICO icon", "Microsoft", "", "raster image", "image/vnd.microsoft.icon", "Windows icons and website favicons"),
    ("psd", "Photoshop Document", "Adobe", "1990", "raster image", "image/vnd.adobe.photoshop", "layered images from Adobe Photoshop"),
    ("eps", "Encapsulated PostScript", "Adobe", "", "vector image", "application/postscript", "a PostScript file meant to be placed in another document"),
    ("ps", "PostScript", "Adobe", "1984", "document", "application/postscript", "page-description programming language"),
    ("mp3", "MPEG-1 Audio Layer III", "Fraunhofer IIS / MPEG", "1993", "audio", "audio/mpeg", "lossy audio compression; typical bit rates 128–320 kbit/s"),
    ("wav", "Waveform Audio File Format", "Microsoft and IBM", "1991", "audio", "audio/wav", "usually uncompressed PCM audio in a RIFF container"),
    ("flac", "Free Lossless Audio Codec", "Xiph.Org Foundation", "2001", "audio", "audio/flac", "lossless audio compression, typically to about 50–70% of the original size"),
    ("aac", "Advanced Audio Coding", "MPEG", "1997", "audio", "audio/aac", "lossy audio, the successor to MP3"),
    ("m4a", "MPEG-4 audio", "MPEG / Apple", "", "audio", "audio/mp4", "AAC or ALAC audio in an MPEG-4 container"),
    ("ogg;oga", "Ogg (Vorbis audio)", "Xiph.Org Foundation", "", "audio", "audio/ogg", "open container, usually with Vorbis or Opus audio"),
    ("opus", "Opus audio", "IETF (Xiph.Org, Skype and others)", "2012", "audio", "audio/opus", "low-latency lossy audio for speech and music (RFC 6716)"),
    ("mid;midi", "MIDI file", "MIDI Manufacturers Association", "1983", "audio", "audio/midi", "note events rather than recorded sound"),
    ("mp4;m4v", "MPEG-4 Part 14", "MPEG", "2001", "video", "video/mp4", "container for H.264/H.265 video and AAC audio"),
    ("avi", "Audio Video Interleave", "Microsoft", "1992", "video", "video/x-msvideo", "older RIFF-based video container"),
    ("mov;qt", "QuickTime File Format", "Apple", "1991", "video", "video/quicktime", "the basis of the MPEG-4 container"),
    ("mkv", "Matroska video", "Matroska project", "2002", "video", "video/x-matroska", "open container holding any number of video, audio and subtitle tracks"),
    ("webm", "WebM", "Google", "2010", "video", "video/webm", "VP8/VP9/AV1 video with Vorbis or Opus audio for the web"),
    ("zip", "ZIP archive", "Phil Katz (PKWARE)", "1989", "archive", "application/zip", "compressed archive, usually with DEFLATE"),
    ("gz;gzip", "gzip", "Jean-loup Gailly and Mark Adler", "1992", "archive", "application/gzip", "DEFLATE compression of a single file"),
    ("tar", "tape archive", "Unix (AT&T)", "1979", "archive", "application/x-tar", "uncompressed bundle of files, often compressed afterwards (.tar.gz)"),
    ("7z", "7z archive", "Igor Pavlov", "1999", "archive", "application/x-7z-compressed", "LZMA/LZMA2 compressed archive"),
    ("rar", "RAR archive", "Eugene Roshal", "1993", "archive", "application/vnd.rar", "proprietary compressed archive"),
    ("exe", "Windows executable (Portable Executable)", "Microsoft", "1993", "executable", "application/vnd.microsoft.portable-executable", "programs for Windows"),
    ("dll", "dynamic-link library", "Microsoft", "", "executable", "application/vnd.microsoft.portable-executable", "shared code for Windows programs"),
    ("apk", "Android Package", "Google", "2008", "executable", "application/vnd.android.package-archive", "installable Android apps (a ZIP archive)"),
    ("dmg", "Apple Disk Image", "Apple", "", "disk image", "application/x-apple-diskimage", "macOS installers and disk images"),
    ("iso", "ISO 9660 disc image", "ISO", "1988", "disk image", "application/x-iso9660-image", "an exact image of an optical disc"),
    ("epub", "EPUB", "IDPF (now W3C)", "2007", "e-book", "application/epub+zip", "reflowable e-books: zipped XHTML"),
    ("mobi", "Mobipocket e-book", "Mobipocket", "2000", "e-book", "application/x-mobipocket-ebook", "older Kindle e-book format"),
    ("ttf", "TrueType font", "Apple", "1991", "font", "font/ttf", "outline fonts with quadratic Bézier curves"),
    ("otf", "OpenType font", "Microsoft and Adobe", "1996", "font", "font/otf", "outline fonts with TrueType or CFF outlines"),
    ("woff;woff2", "Web Open Font Format", "W3C", "2010", "font", "font/woff", "compressed fonts for web pages"),
    ("stl", "STL (stereolithography)", "3D Systems", "1987", "3D model", "model/stl", "triangle meshes, the usual format for 3D printing"),
    ("obj", "Wavefront OBJ", "Wavefront Technologies", "", "3D model", "model/obj", "plain-text 3D geometry"),
    ("dwg", "AutoCAD drawing", "Autodesk", "1982", "CAD drawing", "image/vnd.dwg", "native AutoCAD format"),
    ("ics", "iCalendar", "IETF", "1998", "calendar", "text/calendar", "calendar events (RFC 5545)"),
    ("vcf", "vCard", "Versit Consortium", "1995", "contact card", "text/vcard", "electronic business cards"),
    ("eml", "email message", "IETF", "", "email", "message/rfc822", "a single email in Internet Message Format"),
    ("sqlite;db", "SQLite database", "D. Richard Hipp", "2000", "database", "application/vnd.sqlite3", "a whole SQL database in one file"),
    ("py", "Python source", "Guido van Rossum", "1991", "source code", "text/x-python", "Python program"),
    ("c;h", "C source", "Dennis Ritchie (Bell Labs)", "1972", "source code", "text/x-c", "C program or header"),
    ("cpp;cc;cxx;hpp", "C++ source", "Bjarne Stroustrup", "1985", "source code", "text/x-c++src", "C++ program"),
    ("java", "Java source", "Sun Microsystems (James Gosling)", "1995", "source code", "text/x-java-source", "Java program"),
    ("rs", "Rust source", "Mozilla (Graydon Hoare)", "2015", "source code", "text/rust", "Rust program"),
    ("go", "Go source", "Google", "2009", "source code", "text/x-go", "Go program"),
    ("sh", "shell script", "", "", "source code", "application/x-sh", "Unix shell commands"),
    ("sql", "SQL script", "", "", "source code", "application/sql", "database queries and statements"),
];

struct Layout {
    names: &'static [&'static str],
    name: &'static str,
    rows: [&'static str; 4],
    note: &'static str,
}

const LAYOUTS: &[Layout] = &[
    Layout { names: &["qwerty", "us", "us english", "english", "american", "us qwerty", "english us", "united states"], name: "US QWERTY", rows: ["` 1 2 3 4 5 6 7 8 9 0 - =", "Q W E R T Y U I O P [ ] \\", "A S D F G H J K L ; '", "Z X C V B N M , . /"], note: "named after the first six letters of the top letter row; descends from the Sholes and Glidden typewriter (1873)" },
    Layout { names: &["uk", "british", "english uk", "uk english", "uk qwerty"], name: "UK QWERTY", rows: ["` 1 2 3 4 5 6 7 8 9 0 - =", "Q W E R T Y U I O P [ ]", "A S D F G H J K L ; ' #", "\\ Z X C V B N M , . /"], note: "like US QWERTY with £ on Shift+3, \" on Shift+2, @ on Shift+', a # key and an extra \\ key" },
    Layout { names: &["azerty", "french", "france"], name: "French AZERTY", rows: ["² & é \" ' ( - è _ ç à ) =", "A Z E R T Y U I O P ^ $", "Q S D F G H J K L M ù *", "< W X C V B N , ; : !"], note: "digits need Shift; A/Q and Z/W are swapped relative to QWERTY and M sits on the home row" },
    Layout { names: &["qwertz", "german", "germany", "deutsch"], name: "German QWERTZ", rows: ["^ 1 2 3 4 5 6 7 8 9 0 ß ´", "Q W E R T Z U I O P Ü +", "A S D F G H J K L Ö Ä #", "< Y X C V B N M , . -"], note: "Y and Z are swapped relative to QWERTY; umlauts and ß have their own keys" },
    Layout { names: &["dvorak", "us dvorak", "dvorak simplified"], name: "Dvorak Simplified Keyboard", rows: ["` 1 2 3 4 5 6 7 8 9 0 [ ]", "' , . P Y F G C R L / = \\", "A O E U I D H T N S -", "; Q J K X B M W V Z"], note: "August Dvorak and William Dealey, patented 1936; vowels on the left home row and common consonants on the right, so that hands alternate" },
    Layout { names: &["colemak"], name: "Colemak", rows: ["` 1 2 3 4 5 6 7 8 9 0 - =", "Q W F P G J L U Y ; [ ] \\", "A R S T D H N E I O '", "Z X C V B K M , . /"], note: "Shai Coleman, 2006; changes 17 keys from QWERTY and keeps the shortcuts Z X C V" },
    Layout { names: &["spanish", "spain"], name: "Spanish QWERTY", rows: ["º 1 2 3 4 5 6 7 8 9 0 ' ¡", "Q W E R T Y U I O P ` +", "A S D F G H J K L Ñ ´ ç", "< Z X C V B N M , . -"], note: "Ñ has its own key; accents are typed with dead keys" },
    Layout { names: &["italian", "italy"], name: "Italian QWERTY", rows: ["\\ 1 2 3 4 5 6 7 8 9 0 ' ì", "Q W E R T Y U I O P è +", "A S D F G H J K L ò à ù", "< Z X C V B N M , . -"], note: "accented vowels have their own keys" },
];

struct Format {
    exts: Vec<&'static str>,
    row: &'static (&'static str, &'static str, &'static str, &'static str, &'static str, &'static str, &'static str),
}

fn formats() -> &'static Vec<Format> {
    static F: OnceLock<Vec<Format>> = OnceLock::new();
    F.get_or_init(|| FORMATS.iter().map(|r| Format { exts: r.0.split(';').collect(), row: r }).collect())
}

fn find_format(e: &str) -> Option<&'static Format> {
    let e = e.trim().trim_start_matches('.').to_lowercase();
    formats().iter().find(|f| f.exts.contains(&e.as_str()) || f.row.1.to_lowercase() == e)
}

fn format_answer(f: &'static Format, focus: Option<&str>) -> Answer {
    let (_, name, dev, year, cat, mime, note) = *f.row;
    let exts = f.exts.iter().map(|e| format!(".{e}")).collect::<Vec<_>>().join(", ");
    let mut rows = vec![vec![text("name"), text(name)], vec![text("extensions"), text(exts.clone())], vec![text("type"), text(cat)], vec![text("MIME type"), text(mime)]];
    if !dev.is_empty() {
        rows.push(vec![text("developed by"), text(dev)]);
    }
    if !year.is_empty() {
        rows.push(vec![text("introduced"), text(year)]);
    }
    rows.push(vec![text("notes"), text(note)]);
    let (title, value) = match focus {
        Some("mime") => ("MIME type", mime.to_string()),
        _ => ("Result", format!("{name} ({cat} file)")),
    };
    Answer::new(format!("{exts} file format"), vec![
        pod("Input", "Input interpretation", text(format!("{exts} (file format)"))),
        primary(pod("Result", title, text(value))),
        pod("Properties", "Properties", table(None, rows)),
    ])
}

fn find_layout(s: &str) -> Option<&'static Layout> {
    let s = s.trim().trim_end_matches(" layout").trim_end_matches(" keyboard").trim_end_matches(" keyboard layout").trim();
    let s = s.trim_start_matches("the ").trim_start_matches("a ");
    LAYOUTS.iter().find(|k| k.names.contains(&s))
}

fn layout_pods(k: &Layout) -> Vec<Pod> {
    vec![
        pod(&k.name.replace(' ', ""), k.name, table(None, k.rows.iter().map(|r| vec![text(r.to_string())]).collect())),
        pod(&format!("{}Note", k.name.replace(' ', "")), "Notes", text(k.note)),
    ]
}

pub fn handle(q: &Query) -> Option<Answer> {
    let l = q.lower.trim().trim_end_matches(['?', '.', '!']).trim();
    let l = re!(r"\s+").replace_all(l, " ").to_string();
    let l = l.trim_start_matches("a ").trim_start_matches("an ").trim_start_matches("the ").to_string();
    // "mime type of mp3", "mime type for .png"
    if let Some(m) = re!(r"^(?:the )?mime type (?:of|for) (?:an? )?\.?([a-z0-9]+)(?: files?)?$").captures(&l) {
        return find_format(&m[1]).map(|f| format_answer(f, Some("mime")));
    }
    // ".pdf file format", "png file", "mp3 format", "what is a .docx file", "file extension .rs", ".heic"
    if let Some(m) = re!(r"^(?:what is (?:an? |the )?|what are |what's (?:an? )?)?(?:(?:file )?extension )?(\.)?([a-z0-9]{1,8}) (?:file format|file type|file extension|files?|format|extension)$|^(?:what is (?:an? )?)?(?:file )?extension (\.)?([a-z0-9]{1,8})$|^(\.)([a-z0-9]{1,8})$").captures(&l) {
        let ext = m.get(2).or(m.get(4)).or(m.get(6))?.as_str();
        let dotted = m.get(1).or(m.get(3)).or(m.get(5)).is_some();
        let f = find_format(ext)?;
        // "c files", "go format" are too ambiguous without a dot
        if !dotted && ext.len() < 3 && !l.contains("file") {
            return None;
        }
        return Some(format_answer(f, None));
    }
    // keyboard layouts: "english keyboard layout", "french keyboard, arabic keyboard", "QWERTY", "Dvorak keyboard"
    if l.contains("keyboard") || ["qwerty", "azerty", "qwertz", "dvorak", "colemak"].contains(&l.as_str()) {
        let parts: Vec<&str> = re!(r"\s*(?:,|\bvs\.?|\band\b)\s*").split(&l).filter(|s| !s.is_empty()).collect();
        let ks: Vec<&'static Layout> = parts.iter().filter_map(|p| find_layout(p)).collect();
        if ks.is_empty() || ks.len() != parts.len() && ks.len() < 2 {
            return None;
        }
        let mut pods = vec![pod("Input", "Input interpretation", text(format!("{} (keyboard layout)", ks.iter().map(|k| k.name).collect::<Vec<_>>().join(" | ")))), primary(pod("Result", "Letter rows", text(ks.iter().map(|k| format!("{}: {}", k.name, k.rows[1..].iter().map(|r| r.split(' ').filter(|c| c.chars().all(char::is_alphabetic)).collect::<String>()).collect::<Vec<_>>().join(" / "))).collect::<Vec<_>>().join(" | "))))];
        for k in &ks {
            pods.extend(layout_pods(k));
        }
        return Some(Answer::new("keyboard layout", pods));
    }
    None
}
