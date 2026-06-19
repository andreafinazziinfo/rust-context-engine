use crate::tracking;
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

pub fn run_dashboard() -> Result<()> {
    let report_path = Path::new("dashboard.html");
    println!(
        "📊 Generating local savings dashboard at {}...",
        report_path.display()
    );

    let (count, original, _filtered, saved, usd_saved) =
        tracking::get_savings_data().context("failed to fetch tracking statistics")?;

    let command_breakdown =
        tracking::get_command_breakdown().context("failed to fetch command breakdown")?;

    // Generate Chart.js datasets from command breakdown
    let mut labels = Vec::new();
    let mut counts = Vec::new();
    let mut saved_tokens = Vec::new();

    for (cmd, cnt, svd) in command_breakdown {
        labels.push(format!("'{}'", cmd));
        counts.push(cnt.to_string());
        saved_tokens.push(svd.to_string());
    }

    let labels_str = labels.join(", ");
    let counts_str = counts.join(", ");
    let saved_tokens_str = saved_tokens.join(", ");

    let html_content = format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>AI Token Saver — Dashboard</title>
    <script src="https://cdn.tailwindcss.com"></script>
    <script src="https://cdn.jsdelivr.net/npm/chart.js"></script>
    <link href="https://fonts.googleapis.com/css2?family=Outfit:wght@300;400;600;800&display=swap" rel="stylesheet">
    <style>
        body {{
            font-family: 'Outfit', sans-serif;
            background-color: #0f172a;
            color: #f8fafc;
        }}
        .glass {{
            background: rgba(30, 41, 59, 0.7);
            backdrop-filter: blur(12px);
            border: 1px solid rgba(255, 255, 255, 0.05);
        }}
    </style>
</head>
<body class="min-h-screen flex flex-col justify-between py-8 px-4 sm:px-8">
    <div class="max-w-6xl mx-auto w-full">
        <!-- Header -->
        <header class="flex flex-col sm:flex-row justify-between items-start sm:items-center mb-8 gap-4">
            <div>
                <h1 class="text-4xl font-extrabold tracking-tight bg-gradient-to-r from-emerald-400 via-teal-400 to-cyan-500 bg-clip-text text-transparent">
                    AI Token Saver
                </h1>
                <p class="text-slate-400 text-sm mt-1">Local Savings Telemetry & Efficiency Report</p>
            </div>
            <div class="px-4 py-1.5 rounded-full text-xs font-semibold bg-emerald-500/10 text-emerald-400 border border-emerald-500/20 flex items-center gap-1.5">
                <span class="w-2 h-2 rounded-full bg-emerald-400 animate-pulse"></span>
                Local SQLite Database Connected
            </div>
        </header>

        <!-- Stats Grid -->
        <div class="grid grid-cols-1 md:grid-cols-4 gap-6 mb-8">
            <div class="glass p-6 rounded-2xl shadow-xl flex flex-col justify-between">
                <span class="text-slate-400 text-xs font-semibold uppercase tracking-wider">Commands Run</span>
                <span class="text-4xl font-extrabold text-white mt-2">{count}</span>
            </div>
            <div class="glass p-6 rounded-2xl shadow-xl flex flex-col justify-between">
                <span class="text-slate-400 text-xs font-semibold uppercase tracking-wider">Original Tokens</span>
                <span class="text-4xl font-extrabold text-slate-300 mt-2">{original}</span>
            </div>
            <div class="glass p-6 rounded-2xl shadow-xl flex flex-col justify-between">
                <span class="text-slate-400 text-xs font-semibold uppercase tracking-wider">Tokens Saved</span>
                <span class="text-4xl font-extrabold text-emerald-400 mt-2">{saved}</span>
            </div>
            <div class="glass p-6 rounded-2xl shadow-xl flex flex-col justify-between border-t border-emerald-500/20">
                <span class="text-slate-400 text-xs font-semibold uppercase tracking-wider">Estimated Savings</span>
                <span class="text-4xl font-extrabold text-teal-400 mt-2">${usd_saved:.2}</span>
            </div>
        </div>

        <!-- Charts Section -->
        <div class="grid grid-cols-1 lg:grid-cols-2 gap-8 mb-8">
            <div class="glass p-6 rounded-2xl shadow-xl">
                <h3 class="text-lg font-semibold text-white mb-4">Command Breakdown (Frequency)</h3>
                <div class="h-64 relative">
                    <canvas id="frequencyChart"></canvas>
                </div>
            </div>
            <div class="glass p-6 rounded-2xl shadow-xl">
                <h3 class="text-lg font-semibold text-white mb-4">Saved Tokens per Command</h3>
                <div class="h-64 relative">
                    <canvas id="savingsChart"></canvas>
                </div>
            </div>
        </div>

        <!-- Command breakdown table -->
        <div class="glass rounded-2xl shadow-xl p-6 overflow-hidden">
            <h3 class="text-lg font-semibold text-white mb-4">Telemetry Records</h3>
            <div class="overflow-x-auto">
                <table class="w-full text-left text-sm text-slate-300">
                    <thead class="text-xs uppercase bg-slate-800/40 text-slate-400 border-b border-slate-700/50">
                        <tr>
                            <th class="px-6 py-3">Command Category</th>
                            <th class="px-6 py-3">Invocations</th>
                            <th class="px-6 py-3">Total Tokens Saved</th>
                        </tr>
                    </thead>
                    <tbody class="divide-y divide-slate-800/50">
                        <script>
                            const cmds = [{labels_str}];
                            const counts = [{counts_str}];
                            const saved = [{saved_tokens_str}];

                            if (cmds.length === 0) {{
                                document.write('<tr><td colspan="3" class="px-6 py-4 text-center text-slate-500">No telemetry data recorded yet.</td></tr>');
                            }} else {{
                                for (let i = 0; i < cmds.length; i++) {{
                                    document.write(`
                                        <tr class="hover:bg-slate-800/20 transition-colors">
                                            <td class="px-6 py-4 font-semibold text-white">${{cmds[i]}}</td>
                                            <td class="px-6 py-4">${{counts[i]}}</td>
                                            <td class="px-6 py-4 text-emerald-400 font-mono">+${{saved[i]}}</td>
                                        </tr>
                                    `);
                                }}
                            }}
                        </script>
                    </tbody>
                </table>
            </div>
        </div>
    </div>

    <!-- Footer -->
    <footer class="max-w-6xl mx-auto w-full text-center text-xs text-slate-600 mt-8">
        AI Token Saver is licensed under the Apache License 2.0. Generated locally.
    </footer>

    <!-- Chart Configuration script -->
    <script>
        const ctxFreq = document.getElementById('frequencyChart').getContext('2d');
        const ctxSave = document.getElementById('savingsChart').getContext('2d');

        const labelData = [{labels_str}];

        new Chart(ctxFreq, {{
            type: 'bar',
            data: {{
                labels: labelData,
                datasets: [{{
                    label: 'Invocations',
                    data: [{counts_str}],
                    backgroundColor: 'rgba(20, 184, 166, 0.6)',
                    borderColor: 'rgb(20, 184, 166)',
                    borderWidth: 1,
                    borderRadius: 8
                }}]
            }},
            options: {{
                responsive: true,
                maintainAspectRatio: false,
                plugins: {{
                    legend: {{ display: false }}
                }},
                scales: {{
                    y: {{
                        beginAtZero: true,
                        grid: {{ color: 'rgba(255,255,255,0.05)' }},
                        ticks: {{ color: '#94a3b8' }}
                    }},
                    x: {{
                        grid: {{ display: false }},
                        ticks: {{ color: '#94a3b8' }}
                    }}
                }}
            }}
        }});

        new Chart(ctxSave, {{
            type: 'doughnut',
            data: {{
                labels: labelData,
                datasets: [{{
                    data: [{saved_tokens_str}],
                    backgroundColor: [
                        'rgba(16, 185, 129, 0.6)',
                        'rgba(6, 182, 212, 0.6)',
                        'rgba(59, 130, 246, 0.6)',
                        'rgba(139, 92, 246, 0.6)',
                        'rgba(236, 72, 153, 0.6)',
                        'rgba(245, 158, 11, 0.6)'
                    ],
                    borderColor: '#0f172a',
                    borderWidth: 2
                }}]
            }},
            options: {{
                responsive: true,
                maintainAspectRatio: false,
                plugins: {{
                    legend: {{
                        position: 'right',
                        labels: {{ color: '#e2e8f0' }}
                    }}
                }}
            }}
        }});
    </script>
</body>
</html>
"#
    );

    fs::write(report_path, html_content).with_context(|| {
        format!(
            "failed to write dashboard report file: {}",
            report_path.display()
        )
    })?;

    println!("✅ Dashboard report created successfully.");

    // Auto-launch browser
    open_browser(report_path);

    Ok(())
}

fn open_browser(path: &Path) {
    let path_str = path.to_string_lossy().to_string();
    println!("🌐 Opening dashboard in browser...");
    #[cfg(target_os = "windows")]
    {
        let _ = std::process::Command::new("cmd")
            .args(["/C", "start", &path_str])
            .status();
    }
    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("open").arg(&path_str).status();
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        let _ = std::process::Command::new("xdg-open")
            .arg(&path_str)
            .status();
    }
}
