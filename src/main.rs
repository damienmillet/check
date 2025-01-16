use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

fn main() {
    // Récupération des arguments
    let args: Vec<String> = env::args().collect();

    // Vérification des options
    let mut path = Path::new(".");
    let mut user = String::new(); // Valeur par défaut
    let mut human_readable = false;
    let mut verbose = false; // Ajout du flag verbose

    if args.len() < 2 || args[1] == "-h" || args[1] == "--help" {
        println!("Check - A simple tool to check file permissions for a specific user.");
        println!("Usage: {} [-H] [-u <user>] [-v] <folder_path>", args[0]);
        println!("Options:");
        println!("  -H  Display human-readable names instead of full paths.");
        println!("  -u <user>  Specify the user for permission checking.");
        println!("  -v  Display verbose information.");
        println!("  -h, --help  Display this help message.");
        println!("\nExample: {} /var/www -u user1", args[0]);
        std::process::exit(1);
    }

    // Traitement des options
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-H" => {
                human_readable = true;
                i += 1;
            }
            "-v" | "--verbose" => {
                verbose = true; // Si -v ou --verbose, on active verbose
                i += 1;
            }
            "-u" => {
                if i + 1 < args.len() {
                    user = args[i + 1].clone();
                    i += 2;
                } else {
                    eprintln!("{}", red("Error: Missing argument for -u option.", None));
                    std::process::exit(2);
                }
            }
            _ => {
                path = Path::new(&args[i]);
                i += 1;
            }
        }
    }

    // Si aucun utilisateur n'est spécifié, utiliser l'utilisateur courant
    if user.is_empty() {
        user = whoami();
    }

    // Affiche les informations uniquement si verbose est activé
    if verbose {
        println!("path : {}", path.display());
        println!("user : {}", user);
    }
    println!();

    // Vérification d'existence
    if !path.exists() {
        eprintln!("{}", red("Error: The specified path doesn't exist.", None));
        std::process::exit(2);
    }

    // Si c'est un répertoire, on lance la vérification récursive
    if path.is_dir() {
        traverse_directory(path, &user, 0, human_readable);
    } else {
        check_rights(path, &user, 0, human_readable);
    }

    println!();
}

// Vérifie les droits d'accès au chemin donné pour un utilisateur spécifique
fn check_rights(path: &Path, user: &str, level: usize, human_readable: bool) {
    // Ne pas ajouter d'indentation pour le premier élément
    let indentation = if level > 0 {
        "    ".repeat(level - 1) // Utiliser des espaces pour les indentations
    } else {
        String::new()
    };

    // Vérification si c'est le dernier enfant pour ajuster l'affichage
    let first_child = level == 0;
    let last_child = if let Some(parent) = path.parent() {
        if let (Some(parent_name), Some(file_name)) = (parent.file_name(), path.file_name()) {
            parent_name == file_name
        } else {
            false
        }
    } else {
        false
    };

    // Affiche seulement le nom de l'élément si l'option -H est activée
    let display_name = if human_readable {
        if path == Path::new("/") {
            "/".to_string()
        } else {
            match path.file_name() {
                Some(name) => name.to_str().unwrap_or("Unknown").to_string(),
                None => "Unknown".to_string(),
            }
        }
    } else {
        path.display().to_string()
    };

    // Affichage avec les chemins sous forme de "tree"
    if last_child {
        print!("{}└── {}", indentation, bold(&blue(&display_name, None))); // Ajustement pour l'élément final
    } else if first_child {
        print!("{}{}", indentation, bold(&blue(&display_name, None))); // Premier élément sans indentation
    } else {
        print!("{}├── {}", indentation, bold(&blue(&display_name, None))); // Autres éléments avec ├──
    }

    // Vérifie les permissions RWT
    print!(" ");
    if is_readable(path, user) {
        print!("{}", bold(&green("R", None)));
    } else {
        print!("{}", bold(&red("R", None)));
    }

    if is_writable(path, user) {
        print!("{}", bold(&green("W", None)));
    } else {
        print!("{}", bold(&red("W", None)));
    }

    if path.is_dir() {
        if is_traversable(path, user) {
            print!("{}", bold(&green("T", None)));
        } else {
            print!("{}", bold(&red("T", None)));
        }
    }

    println!(); // Retour à la ligne après chaque chemin et permissions
}

// Vérifie si le chemin est lisible par l'utilisateur
fn is_readable(path: &Path, user: &str) -> bool {
    let output = Command::new("sudo")
        .arg("-u")
        .arg(user)
        .arg("test")
        .arg("-r")
        .arg(path.to_str().unwrap())
        .output();

    output.map(|o| o.status.success()).unwrap_or(false)
}

// Vérifie si le chemin est modifiable par l'utilisateur
fn is_writable(path: &Path, user: &str) -> bool {
    let output = Command::new("sudo")
        .arg("-u")
        .arg(user)
        .arg("test")
        .arg("-w")
        .arg(path.to_str().unwrap())
        .output();

    output.map(|o| o.status.success()).unwrap_or(false)
}

// Vérifie si un répertoire est traversable par l'utilisateur
fn is_traversable(path: &Path, user: &str) -> bool {
    if path.is_dir() {
        let output = Command::new("sudo")
            .arg("-u")
            .arg(user)
            .arg("test")
            .arg("-x")
            .arg(path.to_str().unwrap())
            .output();

        output.map(|o| o.status.success()).unwrap_or(false)
    } else {
        false
    }
}

// Fonction récursive pour parcourir les sous-répertoires
fn traverse_directory(path: &Path, user: &str, level: usize, human_readable: bool) {
    if path.is_dir() {
        // Vérification des permissions du répertoire actuel
        check_rights(path, user, level, human_readable);

        // Parcours des éléments dans le répertoire
        if let Ok(entries) = fs::read_dir(path) {
            let entries_vec: Vec<_> = entries.filter_map(|entry| entry.ok()).collect();
            let last_index = entries_vec.len() - 1; // Dernier indice

            for (i, entry) in entries_vec.iter().enumerate() {
                let entry_path = entry.path();
                // Si c'est un répertoire, on l'explore récursivement
                if entry_path.is_dir() {
                    if i == last_index {
                        // Dernier élément du répertoire, affichage avec └──
                        traverse_directory(&entry_path, user, level + 1, human_readable);
                    } else {
                        // Autres éléments avec ├──
                        traverse_directory(&entry_path, user, level + 1, human_readable);
                    }
                } else {
                    // Si c'est un fichier, on vérifie ses permissions
                    check_rights(&entry_path, user, level + 1, human_readable);
                }
            }
        }
    }
}

fn whoami() -> String {
    env::var("USER").unwrap_or_else(|_| "Unknown".to_string())
}

// COLORATION
macro_rules! color {
    // Si des paramètres supplémentaires sont fournis, les formater avec `params`
    ($color:expr, $text:expr, $($arg:tt)*) => {
        format!("\x1b[{}m{}\x1b[0m", $color, format!($text, $($arg)*))
    };
    // Si aucun paramètre supplémentaire n'est fourni, on applique juste la couleur sans formater
    ($color:expr, $text:expr) => {
        format!("\x1b[{}m{}\x1b[0m", $color, $text)
    };
}
fn red(text: &str, params: Option<&[&str]>) -> String {
    match params {
        Some(p) => color!(31, "{} {:?}", text, p),
        None => color!(31, text),
    }
}

fn green(text: &str, params: Option<&[&str]>) -> String {
    match params {
        Some(p) => color!(32, "{} {:?}", text, p),
        None => color!(32, text),
    }
}

fn blue(text: &str, params: Option<&[&str]>) -> String {
    match params {
        Some(p) => color!(34, "{} {:?}", text, p),
        None => color!(34, text),
    }
}

fn bold(text: &str) -> String {
    format!("\x1b[1m{}\x1b[0m", text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_red() {
        assert_eq!(
            red("Error: The specified path doesn't exist.", None),
            "\x1b[31mError: The specified path doesn't exist.\x1b[0m"
        );
    }

    #[test]
    fn test_green() {
        assert_eq!(green("Done!", None), "\x1b[32mDone!\x1b[0m");
    }

    #[test]
    fn test_blue() {
        assert_eq!(blue("└──", None), "\x1b[34m└──\x1b[0m");
    }

    #[test]
    fn test_whoami() {
        print!("whoami : {}", whoami());
        assert_eq!(whoami(), "damien");
    }
}
