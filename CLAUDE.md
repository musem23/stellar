# Stellar - Outil d'organisation de fichiers

## Description
Stellar organise automatiquement les fichiers d'un dossier en les classant et les renommant selon des règles configurables.

## Entrée
- Dossier parent configurable (par défaut : `~/Downloads`)
- Peut être n'importe quel dossier du système (ex: `~/Desktop`, `~/Documents`, `/path/to/folder`)
- Passé en argument : `stellar /chemin/vers/dossier`

## Modes d'organisation (extensible)

### 1. Par catégorie (défaut)
Classe les fichiers par type dans des sous-dossiers :
```
Downloads/
├── Documents/
├── Images/
├── Videos/
├── Audio/
├── Archives/
└── Autres/
```

### 2. Par date (archivage)
Classe les fichiers par année/mois :
```
Downloads/
├── 2024/
│   ├── 01-janvier/
│   ├── 02-fevrier/
│   └── ...
└── 2025/
    ├── 01-janvier/
    └── ...
```

> Architecture pensée pour ajouter facilement d'autres modes plus tard.

## Modes de renommage (extensible)

### 1. Nettoyage (défaut)
- Espaces → tirets
- Tout en minuscules
- Supprime les caractères spéciaux
- Supprime les doublons type "(1)", "(copie)"

```
Avant:  "Rapport FINAL (1).pdf"
Après:  "rapport-final.pdf"
```

### 2. Date + nom
- Préfixe avec la date de modification du fichier

```
Avant:  "rapport.pdf"
Après:  "2024-01-15-rapport.pdf"
```

> Architecture pensée pour ajouter facilement d'autres modes plus tard.

## Logique principale

1. **Vérifier si le dossier est vide**
   - Si vide → mode **watch** (surveille les nouveaux fichiers)
   - Sinon → organiser les fichiers existants

2. **Pour chaque fichier à la racine :**
   - Appliquer le renommage choisi
   - Déplacer dans le bon sous-dossier selon le mode d'organisation

3. **Pour les sous-dossiers existants :**
   - Scanner les fichiers à l'intérieur
   - Renommer le dossier selon son contenu dominant
   - Exemple : un dossier avec 80% d'images → renommé "Images-xxx"

4. **Mode watch**
   - Surveiller le dossier en continu
   - Organiser automatiquement chaque nouveau fichier
   - Peut changer de dossier cible (pour corriger un sous-dossier spécifique)

## Gestion des conflits de noms
Quand un fichier avec le même nom existe déjà dans le dossier destination :
- Ajouter un suffixe numérique automatique
```
rapport.pdf → rapport-1.pdf → rapport-2.pdf
```

## Fichier de configuration

Fichier `stellar.toml` (ou `~/.config/stellar/config.toml`) :

```toml
[general]
default_path = "~/Downloads"

[organization]
mode = "category"  # ou "date"

[rename]
mode = "clean"  # ou "date-prefix"

[categories]
Documents = [".pdf", ".doc", ".docx", ".txt", ".odt"]
Images = [".png", ".jpg", ".jpeg", ".gif", ".webp"]
Videos = [".mp4", ".mkv", ".avi", ".mov"]
Audio = [".mp3", ".wav", ".flac", ".aac"]
Archives = [".zip", ".tar", ".gz", ".rar", ".7z"]
Code = [".rs", ".js", ".py", ".html", ".css", ".json"]
# Facile d'ajouter de nouvelles catégories ici
```

## Sécurité - Dossiers protégés

Stellar **refuse** d'opérer sur certains dossiers sensibles pour éviter de saccager le système :

### Dossiers système bloqués
```
/
/System
/Library
/usr
/bin
/sbin
/etc
/var
/private
~/.config
~/.ssh
~/.gnupg
~/Library
```

### Dossiers de développement bloqués
```
node_modules/
.git/
.svn/
target/          # Rust
venv/            # Python
__pycache__/
.cargo/
```

### Comportement
- Si l'utilisateur cible un dossier bloqué → **erreur + message explicatif**
- Si un sous-dossier bloqué est trouvé pendant le scan → **ignoré automatiquement**
- Liste configurable dans `stellar.toml` pour ajouter/retirer des protections

## Catégories par défaut

| Catégorie   | Extensions                        |
|-------------|-----------------------------------|
| Documents   | .pdf, .doc, .docx, .txt, .odt     |
| Images      | .png, .jpg, .jpeg, .gif, .webp    |
| Videos      | .mp4, .mkv, .avi, .mov            |
| Audio       | .mp3, .wav, .flac, .aac           |
| Archives    | .zip, .tar, .gz, .rar, .7z        |
| Code        | .rs, .js, .py, .html, .css, .json |
| Autres      | tout le reste                     |

## Architecture technique (pour extensibilité)

```
src/
├── main.rs              # Point d'entrée, CLI
├── config.rs            # Lecture/écriture config
├── scanner.rs           # Scanner les dossiers
├── organizer/
│   ├── mod.rs           # Trait Organizer
│   ├── category.rs      # Mode par catégorie
│   └── date.rs          # Mode par date
├── renamer/
│   ├── mod.rs           # Trait Renamer
│   ├── clean.rs         # Mode nettoyage
│   └── date_prefix.rs   # Mode date + nom
└── watcher.rs           # Mode watch
```

> Utiliser des **traits** Rust pour les modes, permettant d'ajouter facilement de nouveaux modes.
