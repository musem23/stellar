mod config;
use config::load_config;
use std::env;
use std::fs;
use std::io;

fn main() {
    println!(
        r"
  \|/
 --*--  Stellar - Organize your files in a snap
  /|\
"
    );

    let config = load_config().unwrap();
    let protected_user = config.protected.user;
    let protected_system = config.protected.system;
    let protected_dev = config.protected.dev;
    let categories = config.categories;
    let home_dir = env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let mut folders: Vec<String> = Vec::new();


    println!("Available folders:\n");

    for entry in fs::read_dir(&home_dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();

        // Only folders, no hidden files
        if path.is_dir() {
            if let Some(name) = path.file_name() {
                let name = name.to_string_lossy();
                if !name.starts_with('.')
                    && !protected_user.contains(&name.to_string())
                    && !protected_system.contains(&name.to_string())
                    && !protected_dev.contains(&name.to_string())
                {
                    folders.push(name.to_string());
                }
            }
        }
    }

    for (i, folder) in folders.iter().enumerate() {
        println!("  [{}] 📁 {}", i + 1, folder);
    }

    let mut input = String::new();
    println!("\nSelect a folder to organize (1-{}):", folders.len());
    io::stdin().read_line(&mut input).unwrap();

    let choice = input.trim().parse::<usize>().unwrap_or(0);

    if choice < 1 || choice > folders.len() {
        println!("Invalid choice");
        return;
    }

    let selected_folder = &folders[choice - 1];
    println!("\nYou selected: {}", selected_folder);

    let source_dir = format!("{}/{}", home_dir, selected_folder);

    for entry in fs::read_dir(&source_dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();

        if path.is_file() {
            if let Some(ext) = path.extension() {
                let ext = ext.to_string_lossy().to_lowercase();
                let category = config::find_category(&categories, &ext)
                    .unwrap_or_else(|| "Others".to_string());
                println!("{:?} -> {}", path.file_name().unwrap(), category);
            }
        }
    }
}
