use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

const IDIOM_COUNT: usize = 4096;

fn main() {
    let out_dir = env::var("OUT_DIR").expect("OUT_DIR not set");
    let dest_path = Path::new(&out_dir).join("idiom_data.rs");

    let data_dir = Path::new("data");
    let txt_path = data_dir.join("four_chinese_idiom.txt");
    let csv_path = data_dir.join("four_chinese_idiom.csv");

    // Verify that the required file exists
    if !txt_path.exists() {
        panic!("Missing required file: data/four_chinese_idiom.txt");
    }

    // Read TXT file
    let txt_file = File::open(&txt_path).expect("Failed to open four_chinese_idiom.txt");
    let txt_reader = BufReader::new(txt_file);

    let mut idioms: Vec<String> = Vec::with_capacity(IDIOM_COUNT);
    let mut seen_idioms = std::collections::HashSet::new();

    for (line_num, line) in txt_reader.lines().enumerate() {
        let line = line.expect("Failed to read line");
        // Strip leading/trailing whitespace and possible UTF-8 BOM (U+FEFF)
        let idiom = line.trim().trim_start_matches('\u{feff}');

        // Validate non-empty
        if idiom.is_empty() {
            panic!("Empty line at line {}", line_num + 1);
        }

        // Validate four-character: non-four-character idioms would cause
        // idiom_to_index's fast path to always return None, silently occupying
        // a slot without being findable, breaking wordlist integrity.
        // Must abort the build.
        if idiom.chars().count() != 4 {
            panic!(
                "Non-4-character idiom at line {}: '{}' ({} chars)",
                line_num + 1,
                idiom,
                idiom.chars().count()
            );
        }

        // Validate no duplicates
        if seen_idioms.contains(idiom) {
            panic!("Duplicate idiom at line {}: {}", line_num + 1, idiom);
        }
        seen_idioms.insert(idiom.to_string());

        idioms.push(idiom.to_string());
    }

    // Validate total count
    if idioms.len() != IDIOM_COUNT {
        panic!("Expected {} idioms but got {}", IDIOM_COUNT, idioms.len());
    }

    // Read CSV file (if present)
    let mut idioms_with_pinyin: Vec<(String, String)> = Vec::with_capacity(IDIOM_COUNT);

    if csv_path.exists() {
        let csv_file = File::open(&csv_path).expect("Failed to open four_chinese_idiom.csv");
        let csv_reader = BufReader::new(csv_file);

        let mut line_count = 0;
        for (line_num, line) in csv_reader.lines().enumerate() {
            let line = line.expect("Failed to read line");

            // Skip header line (supports both full-width and half-width commas)
            if line_num == 0 {
                let normalized_header = line.replace('\u{ff0c}', ",");
            if normalized_header != "index,chinese_idiom,pinyin" {
                panic!(
                    "Invalid CSV header. Expected 'index,chinese_idiom,pinyin' but got '{}'",
                    line
                );
            }
                continue;
            }

            // Support full-width comma separators
            let normalized_line = line.replace('\u{ff0c}', ",");
            let parts: Vec<&str> = normalized_line.split(',').collect();
            if parts.len() != 3 {
                panic!(
                    "Invalid CSV format at line {}: expected 3 fields, got {}",
                    line_num + 1,
                    parts.len()
                );
            }

            let idx: usize = parts[0]
                .trim()
                .parse()
                .unwrap_or_else(|_| panic!("Invalid index at line {}: {}", line_num + 1, parts[0]));
            let idiom = parts[1].trim().trim_start_matches('\u{feff}');
            let pinyin = parts[2].trim().trim_start_matches('\u{feff}');

            // Validate index
            if idx != line_num - 1 {
                panic!(
                    "Index mismatch at line {}: expected {} but got {}",
                    line_num + 1,
                    line_num - 1,
                    idx
                );
            }

            // Validate idiom consistency
            if idiom != idioms[idx] {
                panic!(
                    "Idiom mismatch at index {}: TXT has '{}' but CSV has '{}'",
                    idx, idioms[idx], idiom
                );
            }

            // Validate pinyin format (should contain tone numbers 1-4)
            if !pinyin.chars().any(|c| c.is_ascii_digit()) {
                println!(
                    "cargo:warning=Pinyin may be missing tone marks at index {}: {}",
                    idx, pinyin
                );
            }

            idioms_with_pinyin.push((idiom.to_string(), pinyin.to_string()));
            line_count += 1;
        }

        // Validate CSV line count
        if line_count != IDIOM_COUNT {
            panic!(
                "Expected {} data lines in CSV but got {}",
                IDIOM_COUNT, line_count
            );
        }
    }

    // ---- Generate Unicode codepoint-sorted search array for binary search ----
    // The wordlist is sorted by pinyin, but Rust string comparison uses Unicode
    // codepoint order, which differs. We therefore generate an extra array sorted
    // by Unicode codepoint for use with binary_search.
    let mut search_entries: Vec<(&str, usize)> = idioms
        .iter()
        .enumerate()
        .map(|(idx, idiom)| (idiom.as_str(), idx))
        .collect();
    search_entries.sort_by_key(|&(word, _)| word);

    // Verify search_entries is correctly sorted (prerequisite for binary search)
    for i in 1..search_entries.len() {
        if search_entries[i].0 < search_entries[i - 1].0 {
            panic!(
                "search_entries not sorted at index {}: '{}' < '{}'",
                i,
                search_entries[i].0,
                search_entries[i - 1].0
            );
        }
    }

    // ---- Generate Rust code ----
    let mut output = File::create(&dest_path).expect("Failed to create output file");

    // Generate codepoint-sorted search array for O(log n) binary search
    writeln!(
        output,
        "/// (Idiom, original index) array sorted by Unicode codepoint for binary search lookup.\n/// Contains {} entries, sorted in ascending codepoint order.",
        IDIOM_COUNT
    )
    .unwrap();
    writeln!(
        output,
        "pub static IDIOM_SEARCH: [(&str, usize); {}] = [",
        IDIOM_COUNT
    )
    .unwrap();
    for (idiom, idx) in &search_entries {
        writeln!(output, "    (\"{}\", {}),", idiom, idx).unwrap();
    }
    writeln!(output, "];\n").unwrap();

    // Generate the plain idiom array (pinyin order, for index → idiom lookup)
    writeln!(
        output,
        "/// Idiom wordlist array (pinyin order), containing {} idioms.",
        IDIOM_COUNT
    )
    .unwrap();
    writeln!(output, "pub static IDIOM_LIST: [&str; {}] = [", IDIOM_COUNT).unwrap();
    for idiom in &idioms {
        writeln!(output, "    \"{}\",", idiom).unwrap();
    }
    writeln!(output, "];\n").unwrap();

    // Generate pinyin array (populated if CSV exists, otherwise empty)
    if !idioms_with_pinyin.is_empty() {
        writeln!(
            output,
            "/// Idiom and pinyin array, containing {} idioms with their pinyin.",
            IDIOM_COUNT
        )
        .unwrap();
        writeln!(
            output,
            "pub static IDIOM_WITH_PINYIN: [(&str, &str); {}] = [",
            IDIOM_COUNT
        )
        .unwrap();
        for (idiom, pinyin) in &idioms_with_pinyin {
            writeln!(output, "    (\"{}\", \"{}\"),", idiom, pinyin).unwrap();
        }
        writeln!(output, "];\n").unwrap();
    } else {
        writeln!(
            output,
            "/// Idiom and pinyin array (CSV file not present, empty array)."
        )
        .unwrap();
        writeln!(
            output,
            "pub static IDIOM_WITH_PINYIN: [(&str, &str); 0] = [];\n"
        )
        .unwrap();
    }

    // Generate count constant
    writeln!(output, "/// Total idiom count.").unwrap();
    writeln!(output, "pub const IDIOM_COUNT: usize = {};\n", IDIOM_COUNT).unwrap();

    println!("cargo:rerun-if-changed=data/four_chinese_idiom.txt");
    println!("cargo:rerun-if-changed=data/four_chinese_idiom.csv");
}
