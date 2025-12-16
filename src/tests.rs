// Stellar - Unit Tests
// @musem23
//
// Unit tests for the renamer module's slugify functionality.

#[cfg(test)]
mod renamer_tests {
    use crate::renamer::rename_file;
    use std::path::Path;

    fn slugify_via_rename(name: &str) -> String {
        let path = Path::new(name);
        rename_file(path, &crate::renamer::RenameMode::Clean)
    }

    #[test]
    fn test_slugify_accents() {
        assert_eq!(slugify_via_rename("élève"), "eleve");
        assert_eq!(slugify_via_rename("café"), "cafe");
        assert_eq!(slugify_via_rename("naïve"), "naive");
        assert_eq!(slugify_via_rename("über"), "uber");
    }

    #[test]
    fn test_slugify_spaces() {
        assert_eq!(slugify_via_rename("hello world"), "hello-world");
        assert_eq!(
            slugify_via_rename("  multiple   spaces  "),
            "multiple-spaces"
        );
    }

    #[test]
    fn test_slugify_special_chars() {
        assert_eq!(slugify_via_rename("file@name#test"), "filenametest");
        assert_eq!(slugify_via_rename("hello world test"), "hello-world-test");
    }

    #[test]
    fn test_remove_copy_suffixes() {
        assert_eq!(slugify_via_rename("rapport-1"), "rapport");
        assert_eq!(slugify_via_rename("fichier-copy"), "fichier");
        assert_eq!(slugify_via_rename("document-copie"), "document");
    }
}
