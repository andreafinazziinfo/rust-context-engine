use lazy_static::lazy_static;
use regex::Regex;

pub fn filter(input: &str) -> String {
    let mut out = String::with_capacity(input.len());

    lazy_static! {
        // Match docker layer pull statuses:
        // b3c9a63d9178: Pulling fs layer
        // b3c9a63d9178: Downloading [=========>] 12.1MB/45MB
        // b3c9a63d9178: Waiting
        // b3c9a63d9178: Verifying Checksum
        // b3c9a63d9178: Download complete
        // b3c9a63d9178: Pull complete
        static ref DOCKER_PULL: Regex = Regex::new(
            r"^[a-f0-9]{12}:\s+(Pulling fs layer|Waiting|Downloading|Extracting|Verifying|Download complete|Pull complete|Already exists)\b"
        ).unwrap();

        // Match generic download bars like:
        // [1/2] [===>               ]
        static ref PROGRESS_BAR: Regex = Regex::new(r"\[[=\->\s]{5,}\]").unwrap();
    }

    for line in input.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if DOCKER_PULL.is_match(trimmed) || PROGRESS_BAR.is_match(trimmed) {
            continue;
        }
        out.push_str(line);
        out.push('\n');
    }

    if out.trim().is_empty() {
        input.to_string()
    } else {
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_docker_pull_filter() {
        let raw = "\
Sending build context to Docker daemon  2.048kB
Step 1/2 : FROM alpine:latest
latest: Pulling from library/alpine
c158987b05aa: Pulling fs layer
c158987b05aa: Downloading [===>                                               ]  1.1MB/3.1MB
c158987b05aa: Verifying Checksum
c158987b05aa: Download complete
c158987b05aa: Pull complete
Digest: sha256:8914ba42c
Status: Downloaded newer image for alpine:latest
 ---> 496a0902c676
";
        let filtered = filter(raw);
        assert!(filtered.contains("Step 1/2"));
        assert!(filtered.contains("Downloaded newer image"));
        assert!(!filtered.contains("Pulling fs layer"));
        assert!(!filtered.contains("Downloading"));
    }
}
