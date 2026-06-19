#!/usr/bin/env python3
import subprocess
import tiktoken
import sys
import os

def count_tokens(text: str, model: str = "cl100k_base") -> int:
    """Counts tokens using tiktoken (cl100k_base is a good proxy for most modern LLMs)."""
    enc = tiktoken.get_encoding(model)
    return len(enc.encode(text))

def run_command(cmd: str, cwd: str = ".") -> str:
    """Runs a shell command and returns its merged stdout/stderr."""
    try:
        # We use check=False to capture output even if the command fails (e.g. failing tests)
        result = subprocess.run(cmd, shell=True, cwd=cwd, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True)
        return result.stdout
    except Exception as e:
        return str(e)

def benchmark_command(name: str, standard_cmd: str, rtk_cmd: str, cwd: str = "."):
    print(f"\n--- Benchmarking: {name} ---")
    
    # 1. Run Standard
    print(f"Running Standard: `{standard_cmd}`...")
    std_out = run_command(standard_cmd, cwd)
    std_tokens = count_tokens(std_out)
    
    # 2. Run RTK
    print(f"Running RTK: `{rtk_cmd}`...")
    rtk_out = run_command(rtk_cmd, cwd)
    rtk_tokens = count_tokens(rtk_out)
    
    if rtk_tokens < 100:
        print(f"DEBUG RTK OUTPUT: {rtk_out}")

    # Calculate savings
    if std_tokens > 0:
        savings = ((std_tokens - rtk_tokens) / std_tokens) * 100
    else:
        savings = 0.0
        
    print(f"Standard Tokens: {std_tokens}")
    print(f"RTK Tokens:      {rtk_tokens}")
    print(f"Savings:         {savings:.1f}%")
    return std_tokens, rtk_tokens, savings

def main():
    print("🚀 RTK Native Benchmark Suite")
    print("====================================")
    
    # Ensure RTK is in PATH or alias is loaded. We will use the absolute path to the binary if needed, 
    # but let's assume it's installed or we use `cargo run`.
    # Actually, we can use `rtk` if it's in PATH, or `./target/debug/rtk`. Let's use `cargo run --manifest-path ../rtk/Cargo.toml --`
    rtk_bin = "~/.cargo/bin/cargo run -q --manifest-path rtk/Cargo.toml --"

    benchmark_command(
        name="cargo test (RTK internal test suite)",
        standard_cmd="~/.cargo/bin/cargo test --manifest-path rtk/Cargo.toml",
        rtk_cmd=f"{rtk_bin} cargo test --manifest-path rtk/Cargo.toml",
        cwd="."
    )

    benchmark_command(
        name="git log (Last 10 commits)",
        standard_cmd="git log -n 10",
        rtk_cmd=f"{rtk_bin} git log -n 10",
        cwd="."
    )
    
    benchmark_command(
        name="ls -la (Recursive)",
        standard_cmd="ls -laR rtk/src",
        rtk_cmd=f"{rtk_bin} ls -laR rtk/src",
        cwd="."
    )

    print(f"\n--- Benchmarking: rtk pack ---")
    print("Running Standard: `find rtk/src -type f -name '*.rs' -exec cat {} +`")
    std_out = run_command("find rtk/src -type f -name '*.rs' -exec cat {} +", cwd=".")
    std_tokens = count_tokens(std_out)
    
    print("Running RTK: `rtk pack rtk/src --strip --skeleton`")
    rtk_out = run_command(f"{rtk_bin} pack rtk/src --strip --skeleton", cwd=".")
    rtk_tokens = count_tokens(rtk_out)
    
    if std_tokens > 0:
        savings = ((std_tokens - rtk_tokens) / std_tokens) * 100
    else:
        savings = 0.0
        
    print(f"Standard Tokens (cat all): {std_tokens}")
    print(f"RTK Tokens (rtk pack):     {rtk_tokens}")
    print(f"Savings:                   {savings:.1f}%")

if __name__ == "__main__":
    main()
