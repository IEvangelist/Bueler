use clap::{Parser, Subcommand};
use std::fs;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

/// 🔥 The Oxide web framework CLI
#[derive(Parser)]
#[command(name = "oxide", version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new Oxide project
    New {
        /// Project name
        name: String,
    },
    /// Start development server with live reload and WASM debugging
    Dev {
        /// Port to serve on
        #[arg(short, long, default_value_t = 8080)]
        port: u16,
        /// Build in release mode (disables WASM debug info)
        #[arg(long)]
        release: bool,
    },
    /// Build for production
    Build,
    /// Serve the production build
    Serve {
        /// Port to serve on
        #[arg(short, long, default_value_t = 8080)]
        port: u16,
    },
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::New { name } => cmd_new(&name),
        Commands::Dev { port, release } => cmd_dev(port, release),
        Commands::Build => cmd_build(),
        Commands::Serve { port } => cmd_serve(port),
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// oxide new
// ═══════════════════════════════════════════════════════════════════════════

fn cmd_new(name: &str) {
    let root = PathBuf::from(name);
    if root.exists() {
        eprintln!("  ✗ Directory '{}' already exists", name);
        std::process::exit(1);
    }

    println!("\n  🔥 Creating Oxide project: {}\n", name);

    let crate_name = name.replace('-', "_");
    let src = root.join("src");
    fs::create_dir_all(&src).unwrap();

    // Cargo.toml
    write_file(
        &root.join("Cargo.toml"),
        &TEMPLATE_CARGO.replace("{{name}}", name),
    );

    // src/lib.rs
    write_file(&src.join("lib.rs"), TEMPLATE_LIB);

    // index.html
    write_file(
        &root.join("index.html"),
        &TEMPLATE_HTML
            .replace("{{name}}", name)
            .replace("{{crate_name}}", &crate_name),
    );

    // .gitignore
    write_file(&root.join(".gitignore"), TEMPLATE_GITIGNORE);

    println!("  ✓ Created {}/Cargo.toml", name);
    println!("  ✓ Created {}/src/lib.rs", name);
    println!("  ✓ Created {}/index.html", name);
    println!("  ✓ Created {}/.gitignore", name);
    println!();
    println!("  Project ready! Next steps:");
    println!();
    println!("    cd {}", name);
    println!("    oxide dev");
    println!();
}

fn write_file(path: &Path, content: &str) {
    fs::write(path, content).unwrap_or_else(|e| {
        eprintln!("  ✗ Failed to write {}: {}", path.display(), e);
        std::process::exit(1);
    });
}

// ═══════════════════════════════════════════════════════════════════════════
// oxide dev
// ═══════════════════════════════════════════════════════════════════════════

fn cmd_dev(port: u16, release: bool) {
    check_wasm_pack();

    println!();
    println!("  🔥 Oxide Dev Server");
    println!();

    // Initial build
    let mode = if release { "release" } else { "dev (debug info enabled)" };
    println!("  ⚡ Building ({})...", mode);
    if !run_build(!release) {
        eprintln!("  ✗ Initial build failed. Fix errors above and try again.");
        std::process::exit(1);
    }
    println!("  ✓ Build complete");
    println!();

    if !release {
        println!("  🐛 Debug tip: Install the Chrome DWARF extension for Rust source debugging:");
        println!("     chrome://extensions → search \"C/C++ DevTools Support (DWARF)\"");
        println!();
    }

    let version = Arc::new(AtomicU64::new(1));

    // Start file watcher in a thread
    let ver_watch = version.clone();
    let release_watch = release;
    std::thread::spawn(move || {
        watch_and_rebuild(ver_watch, release_watch);
    });

    // Start HTTP server
    println!("  🌐 Serving at http://localhost:{}", port);
    println!("  👀 Watching for changes...");
    println!("  (Press Ctrl+C to stop)");
    println!();

    serve(port, version);
}

fn check_wasm_pack() {
    match Command::new("wasm-pack").arg("--version").output() {
        Ok(o) if o.status.success() => {}
        _ => {
            eprintln!("  ✗ wasm-pack is not installed.");
            eprintln!();
            eprintln!("  Install it with:");
            eprintln!("    cargo install wasm-pack");
            eprintln!();
            eprintln!("  Or: curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh");
            std::process::exit(1);
        }
    }
}

fn run_build(dev: bool) -> bool {
    let mut args = vec!["build", "--target", "web"];
    if dev {
        args.push("--dev");
    } else {
        args.push("--release");
    }

    let result = Command::new("wasm-pack")
        .args(&args)
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .status();

    match result {
        Ok(s) => s.success(),
        Err(e) => {
            eprintln!("  ✗ Failed to run wasm-pack: {}", e);
            false
        }
    }
}

fn watch_and_rebuild(version: Arc<AtomicU64>, release: bool) {
    use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
    use std::sync::mpsc;

    let (tx, rx) = mpsc::channel();
    let mut watcher = RecommendedWatcher::new(tx, Config::default().with_poll_interval(Duration::from_secs(1)))
        .expect("Failed to create file watcher");

    if Path::new("src").exists() {
        watcher.watch(Path::new("src"), RecursiveMode::Recursive).ok();
    }
    if Path::new("Cargo.toml").exists() {
        watcher.watch(Path::new("Cargo.toml"), RecursiveMode::NonRecursive).ok();
    }

    // Debounce: wait for events to settle
    loop {
        match rx.recv() {
            Ok(_) => {
                // Drain any queued events (debounce)
                std::thread::sleep(Duration::from_millis(300));
                while rx.try_recv().is_ok() {}

                println!("  ⚡ Rebuilding...");
                if run_build(!release) {
                    let v = version.fetch_add(1, Ordering::SeqCst) + 1;
                    println!("  ✓ Build #{} complete — auto-reloading", v);
                } else {
                    eprintln!("  ✗ Build failed — waiting for fixes...");
                }
            }
            Err(_) => break,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// oxide build
// ═══════════════════════════════════════════════════════════════════════════

fn cmd_build() {
    check_wasm_pack();

    println!();
    println!("  🔥 Oxide Production Build");
    println!();
    println!("  ⚡ Building (release, optimized)...");

    if !run_build(false) {
        eprintln!("  ✗ Build failed.");
        std::process::exit(1);
    }

    // Report sizes
    if let Ok(_meta) = fs::metadata("pkg") {
        let wasm_path = find_wasm_file("pkg");
        if let Some(wasm) = wasm_path {
            let size = fs::metadata(&wasm).map(|m| m.len()).unwrap_or(0);
            println!();
            println!("  ✓ Build complete!");
            println!("    WASM bundle: {} KB", size / 1024);
            println!("    Output: pkg/");
        } else {
            println!("  ✓ Build complete! Output: pkg/");
        }
    } else {
        println!("  ✓ Build complete!");
    }
    println!();
}

fn find_wasm_file(dir: &str) -> Option<PathBuf> {
    fs::read_dir(dir).ok()?.find_map(|entry| {
        let path = entry.ok()?.path();
        if path.extension().and_then(|e| e.to_str()) == Some("wasm") {
            Some(path)
        } else {
            None
        }
    })
}

// ═══════════════════════════════════════════════════════════════════════════
// oxide serve / HTTP server
// ═══════════════════════════════════════════════════════════════════════════

fn cmd_serve(port: u16) {
    println!();
    println!("  🌐 Serving at http://localhost:{}", port);
    println!("  (Press Ctrl+C to stop)");
    println!();

    serve(port, Arc::new(AtomicU64::new(0)));
}

fn serve(port: u16, version: Arc<AtomicU64>) {
    let addr = format!("0.0.0.0:{}", port);
    let server = TcpListener::bind(&addr).unwrap_or_else(|e| {
        eprintln!("  ✗ Failed to bind to {}: {}", addr, e);
        std::process::exit(1);
    });

    for stream in server.incoming() {
        let Ok(mut stream) = stream else { continue };

        let mut buf = [0u8; 4096];
        let n = stream.read(&mut buf).unwrap_or(0);
        let request = String::from_utf8_lossy(&buf[..n]);

        let path = request
            .lines()
            .next()
            .and_then(|line| line.split_whitespace().nth(1))
            .unwrap_or("/");

        if path == "/__oxide_reload" {
            let v = version.load(Ordering::SeqCst);
            let body = v.to_string();
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nAccess-Control-Allow-Origin: *\r\nContent-Length: {}\r\n\r\n{}",
                body.len(),
                body
            );
            stream.write_all(response.as_bytes()).ok();
            continue;
        }

        let file_path = if path == "/" {
            "index.html".to_string()
        } else {
            path.trim_start_matches('/').to_string()
        };

        match fs::read(&file_path) {
            Ok(mut content) => {
                let mime = guess_mime(&file_path);

                // Inject live-reload script into HTML
                if mime == "text/html" && version.load(Ordering::SeqCst) > 0 {
                    let html = String::from_utf8_lossy(&content);
                    let injected = html.replace(
                        "</body>",
                        &format!("{}</body>", RELOAD_SCRIPT),
                    );
                    content = injected.into_bytes();
                }

                let response = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: {}\r\nContent-Length: {}\r\nCache-Control: no-cache\r\n\r\n",
                    mime,
                    content.len()
                );
                stream.write_all(response.as_bytes()).ok();
                stream.write_all(&content).ok();
            }
            Err(_) => {
                let body = "404 Not Found";
                let response = format!(
                    "HTTP/1.1 404 Not Found\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}",
                    body.len(),
                    body
                );
                stream.write_all(response.as_bytes()).ok();
            }
        }
    }
}

fn guess_mime(path: &str) -> &'static str {
    match path.rsplit('.').next() {
        Some("html") => "text/html",
        Some("js") => "application/javascript",
        Some("wasm") => "application/wasm",
        Some("css") => "text/css",
        Some("json") => "application/json",
        Some("png") => "image/png",
        Some("svg") => "image/svg+xml",
        Some("ico") => "image/x-icon",
        _ => "application/octet-stream",
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Templates
// ═══════════════════════════════════════════════════════════════════════════

const TEMPLATE_CARGO: &str = r#"[package]
name = "{{name}}"
version = "0.1.0"
edition = "2024"

[lib]
crate-type = ["cdylib"]

[dependencies]
oxide = { git = "https://github.com/IEvangelist/Oxide" }
wasm-bindgen = "0.2"
"#;

const TEMPLATE_LIB: &str = "use oxide::prelude::*;\nuse wasm_bindgen::prelude::*;\n\n#[wasm_bindgen(start)]\npub fn main() {\n    mount(\"#app\", || {\n        let mut count = signal(0);\n\n        view! {\n            <div class=\"app\">\n                <h1>\"My Oxide App\"</h1>\n                <p>\"Count: \" {count}</p>\n                <div class=\"buttons\">\n                    <button on:click={move |_: oxide::dom::Event| count -= 1}>\n                        \"-\"\n                    </button>\n                    <button on:click={move |_: oxide::dom::Event| count.set(0)}>\n                        \"Reset\"\n                    </button>\n                    <button on:click={move |_: oxide::dom::Event| count += 1}>\n                        \"+\"\n                    </button>\n                </div>\n            </div>\n        }\n    });\n}\n";

const TEMPLATE_HTML: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>{{name}}</title>
    <style>
        * { margin: 0; padding: 0; box-sizing: border-box; }
        body {
            font-family: system-ui, -apple-system, sans-serif;
            background: #0a0a0a; color: #e0e0e0;
            display: flex; justify-content: center; align-items: center;
            min-height: 100vh;
        }
        .app { text-align: center; }
        h1 { margin-bottom: 1rem; }
        p { font-size: 2rem; margin: 1rem 0; font-variant-numeric: tabular-nums; }
        .buttons { display: flex; gap: 0.5rem; justify-content: center; }
        button {
            padding: 0.6rem 1.4rem; font-size: 1.1rem;
            background: #1a1a1a; color: #e0e0e0;
            border: 1px solid #333; border-radius: 8px;
            cursor: pointer; transition: all 0.15s;
        }
        button:hover { border-color: #f97316; color: #f97316; }
        button:active { transform: scale(0.95); }
    </style>
</head>
<body>
    <div id="app"></div>
    <script type="module">
        import init from './pkg/{{crate_name}}.js';
        init();
    </script>
</body>
</html>
"#;

const TEMPLATE_GITIGNORE: &str = "/target\n/pkg\n*.wasm\nCargo.lock\n";

const RELOAD_SCRIPT: &str = r#"<script>(function(){let v="0";setInterval(async()=>{try{const r=await fetch("/__oxide_reload");const nv=await r.text();if(v!=="0"&&nv!==v)location.reload();v=nv}catch(e){}},500)})()</script>"#;
