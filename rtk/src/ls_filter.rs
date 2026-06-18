use lazy_static::lazy_static;
use regex::Regex;

const MAX_FILES_TO_SHOW: usize = 20;

/// Filter `ls` output.
///
/// Strategy:
///   - Compact `ls -l` long listing lines:
///     Drop owner, group, and link count.
///     Format sizes nicely (e.g. 1024 -> 1.0K).
///     Keep permissions, simplified size, and filename.
///   - If file listing exceeds MAX_FILES_TO_SHOW entries, collapse the middle and show a summary.
pub fn filter(input: &str) -> String {
    lazy_static! {
        // Regex to parse GNU ls -l lines:
        // Group 1: permissions (e.g. -rw-r--r--)
        // Group 2: link count (e.g. 1)
        // Group 3: owner (e.g. username)
        // Group 4: group (e.g. username)
        // Group 5: size in bytes (e.g. 1234)
        // Group 6: date/time (e.g. Jun 18 22:00 or Jun 18  2026)
        // Group 7: filename
        static ref LS_L_LINE: Regex = Regex::new(
            r"^([drwx-]{10})\s+(\d+)\s+(\S+)\s+(\S+)\s+(\d+)\s+([A-Z][a-z]{2}\s+\d+\s+[\d:]{4,5})\s+(.+)$"
        ).unwrap();
        
        static ref TOTAL_LINE: Regex = Regex::new(r"^total\s+\d+$").unwrap();
    }

    let mut lines: Vec<String> = Vec::new();
    let mut file_entries = 0;

    for line in input.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        // Drop "total N" line if it's there
        if TOTAL_LINE.is_match(trimmed) {
            continue;
        }

        if let Some(caps) = LS_L_LINE.captures(trimmed) {
            let perm = &caps[1];
            let size_bytes: u64 = caps[5].parse().unwrap_or(0);
            let date = &caps[6];
            let name = &caps[7];

            let size_str = format_size(size_bytes);
            
            // Format: perm size date name
            lines.push(format!("{perm} {size_str:>6} {date} {name}"));
            file_entries += 1;
        } else {
            // Non-long listing or header lines (just file names)
            lines.push(trimmed.to_string());
            file_entries += 1;
        }
    }

    if lines.is_empty() {
        return input.to_string();
    }

    // Collapse middle if there are too many files
    if file_entries > MAX_FILES_TO_SHOW {
        let head_count = 12;
        let tail_count = 5;
        let collapsed_count = file_entries - head_count - tail_count;

        let mut collapsed_out = String::new();
        for line in lines.iter().take(head_count) {
            collapsed_out.push_str(line);
            collapsed_out.push('\n');
        }
        collapsed_out.push_str(&format!("... and {collapsed_count} more entries ...\n"));
        for line in lines.iter().skip(file_entries - tail_count) {
            collapsed_out.push_str(line);
            collapsed_out.push('\n');
        }
        collapsed_out
    } else {
        lines.join("\n") + "\n"
    }
}

fn format_size(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{bytes}B")
    } else if bytes < 1024 * 1024 {
        format!("{:.1}K", bytes as f64 / 1024.0)
    } else if bytes < 1024 * 1024 * 1024 {
        format!("{:.1}M", bytes as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.1}G", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_size() {
        assert_eq!(format_size(500), "500B");
        assert_eq!(format_size(1024), "1.0K");
        assert_eq!(format_size(1536), "1.5K");
        assert_eq!(format_size(1024 * 1024 * 2), "2.0M");
    }

    #[test]
    fn test_filter_ls_l() {
        let input = concat!(
            "total 32\n",
            "drwxr-xr-x  5 username username 4096 Jun 18 22:00 .\n",
            "-rw-r--r--  1 username username 1234 Jun 18 22:00 file1.txt\n",
            "-rwxr-xr-x  1 username username 1048576 Jun 18 22:00 run.sh\n",
        );
        let out = filter(input);
        assert!(!out.contains("total 32"), "should drop total line");
        assert!(!out.contains("username"), "should drop owner and group");
        assert!(out.contains("drwxr-xr-x   4.0K Jun 18 22:00 ."), "incorrect directory formatting");
        assert!(out.contains("-rw-r--r--   1.2K Jun 18 22:00 file1.txt"), "incorrect file1 formatting");
        assert!(out.contains("-rwxr-xr-x   1.0M Jun 18 22:00 run.sh"), "incorrect run.sh formatting");
    }

    #[test]
    fn test_ls_collapse() {
        let mut input = String::new();
        for i in 0..25 {
            input.push_str(&format!("-rw-r--r--  1 username username 100 Jun 18 22:00 file{i}.txt\n"));
        }
        let out = filter(&input);
        let lines: Vec<&str> = out.lines().collect();
        assert_eq!(lines.len(), 18); // 12 head + 1 summary + 5 tail = 18 lines
        assert!(out.contains("... and 8 more entries ..."));
        assert!(out.contains("file0.txt"));
        assert!(out.contains("file24.txt"));
        assert!(!out.contains("file15.txt"));
    }
}
