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

#[cfg(test)]
mod vault_crypto_tests {
    use crate::vault::crypto::{decrypt, encrypt, SALT_SIZE};
    use crate::vault::VaultError;

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let data = b"Hello, Vault!";
        let password = "test_password_123";

        let encrypted = encrypt(data, password).unwrap();
        let decrypted = decrypt(&encrypted, password).unwrap();

        assert_eq!(data.to_vec(), decrypted);
    }

    #[test]
    fn test_wrong_password_fails() {
        let data = b"Secret data";
        let encrypted = encrypt(data, "correct_password").unwrap();

        let result = decrypt(&encrypted, "wrong_password");
        assert!(matches!(result, Err(VaultError::InvalidPassword)));
    }

    #[test]
    fn test_corrupted_data_fails() {
        let result = decrypt(&[0u8; 10], "password");
        assert!(matches!(result, Err(VaultError::CorruptedData)));
    }

    #[test]
    fn test_unique_salt_per_encryption() {
        let data = b"Same data";
        let password = "same_password";

        let encrypted1 = encrypt(data, password).unwrap();
        let encrypted2 = encrypt(data, password).unwrap();

        assert_ne!(&encrypted1[..SALT_SIZE], &encrypted2[..SALT_SIZE]);
    }

    #[test]
    fn test_empty_data() {
        let data = b"";
        let password = "password123";

        let encrypted = encrypt(data, password).unwrap();
        let decrypted = decrypt(&encrypted, password).unwrap();

        assert_eq!(data.to_vec(), decrypted);
    }

    #[test]
    fn test_large_data() {
        let data = vec![0u8; 1024 * 1024];
        let password = "password123";

        let encrypted = encrypt(&data, password).unwrap();
        let decrypted = decrypt(&encrypted, password).unwrap();

        assert_eq!(data, decrypted);
    }
}

#[cfg(test)]
mod vault_recovery_tests {
    use crate::vault::crypto::KEY_SIZE;
    use crate::vault::recovery::RecoveryCodes;

    #[test]
    fn test_recovery_codes_format() {
        let codes = RecoveryCodes::generate();

        assert_eq!(codes.code1.len(), 14);
        assert_eq!(codes.code2.len(), 14);
        assert_eq!(codes.code1.chars().filter(|c| *c == '-').count(), 2);
        assert_eq!(codes.code2.chars().filter(|c| *c == '-').count(), 2);
    }

    #[test]
    fn test_recovery_codes_unique() {
        let codes1 = RecoveryCodes::generate();
        let codes2 = RecoveryCodes::generate();

        assert_ne!(codes1.code1, codes2.code1);
        assert_ne!(codes1.code2, codes2.code2);
    }

    #[test]
    fn test_key_encrypt_decrypt() {
        let codes = RecoveryCodes::generate();
        let key: [u8; KEY_SIZE] = [42u8; KEY_SIZE];

        let encrypted = codes.encrypt_key(&key).unwrap();
        let decrypted = RecoveryCodes::decrypt_key(&codes.code1, &codes.code2, &encrypted).unwrap();

        assert_eq!(key, decrypted);
    }

    #[test]
    fn test_wrong_recovery_code_fails() {
        let codes = RecoveryCodes::generate();
        let key: [u8; KEY_SIZE] = [42u8; KEY_SIZE];

        let encrypted = codes.encrypt_key(&key).unwrap();
        let result = RecoveryCodes::decrypt_key("WRONG-CODE-HERE", &codes.code2, &encrypted);

        assert!(result.is_err());
    }
}

#[cfg(test)]
mod password_validation_tests {
    use crate::vault::{validate_password, VaultError};

    #[test]
    fn test_password_too_short() {
        let result = validate_password("Short1!");
        assert!(matches!(result, Err(VaultError::WeakPassword(_))));
    }

    #[test]
    fn test_password_no_uppercase() {
        let result = validate_password("lowercase123!@#");
        assert!(matches!(result, Err(VaultError::WeakPassword(_))));
    }

    #[test]
    fn test_password_no_lowercase() {
        let result = validate_password("UPPERCASE123!@#");
        assert!(matches!(result, Err(VaultError::WeakPassword(_))));
    }

    #[test]
    fn test_password_no_digit() {
        let result = validate_password("NoDigitsHere!@#");
        assert!(matches!(result, Err(VaultError::WeakPassword(_))));
    }

    #[test]
    fn test_password_no_special() {
        let result = validate_password("NoSpecial12345");
        assert!(matches!(result, Err(VaultError::WeakPassword(_))));
    }

    #[test]
    fn test_password_common_pattern() {
        let result = validate_password("MyPassword123!");
        assert!(matches!(result, Err(VaultError::WeakPassword(_))));
    }

    #[test]
    fn test_password_valid() {
        let result = validate_password("SecureP@ss2024!");
        assert!(result.is_ok());
    }

    #[test]
    fn test_password_complex_valid() {
        let result = validate_password("Tr3s-S3cur3!Passw0rd");
        assert!(result.is_ok());
    }
}

#[cfg(test)]
mod modes_tests {
    use crate::modes::{OrganizationMode, RenameMode};

    #[test]
    fn test_organization_mode_from_str() {
        assert_eq!(OrganizationMode::from_str("category"), OrganizationMode::Category);
        assert_eq!(OrganizationMode::from_str("date"), OrganizationMode::Date);
        assert_eq!(OrganizationMode::from_str("hybrid"), OrganizationMode::Hybrid);
        assert_eq!(OrganizationMode::from_str("cat"), OrganizationMode::Category);
        assert_eq!(OrganizationMode::from_str("d"), OrganizationMode::Date);
        assert_eq!(OrganizationMode::from_str("h"), OrganizationMode::Hybrid);
        assert_eq!(OrganizationMode::from_str("invalid"), OrganizationMode::Category);
    }

    #[test]
    fn test_organization_mode_index_roundtrip() {
        assert_eq!(OrganizationMode::from_index(0), OrganizationMode::Category);
        assert_eq!(OrganizationMode::from_index(1), OrganizationMode::Date);
        assert_eq!(OrganizationMode::from_index(2), OrganizationMode::Hybrid);
        assert_eq!(OrganizationMode::Category.to_index(), 0);
        assert_eq!(OrganizationMode::Date.to_index(), 1);
        assert_eq!(OrganizationMode::Hybrid.to_index(), 2);
    }

    #[test]
    fn test_rename_mode_from_str() {
        assert!(matches!(RenameMode::from_str("clean"), Some(RenameMode::Clean)));
        assert!(matches!(RenameMode::from_str("date-prefix"), Some(RenameMode::DatePrefix)));
        assert!(matches!(RenameMode::from_str("skip"), None));
        assert!(matches!(RenameMode::from_str("none"), None));
    }

    #[test]
    fn test_rename_mode_from_index() {
        assert!(matches!(RenameMode::from_index(0), Some(RenameMode::Clean)));
        assert!(matches!(RenameMode::from_index(1), Some(RenameMode::DatePrefix)));
        assert!(matches!(RenameMode::from_index(2), None)); // Skip returns None
    }

    #[test]
    fn test_organization_mode_display() {
        assert_eq!(format!("{}", OrganizationMode::Category), "Category");
        assert_eq!(format!("{}", OrganizationMode::Date), "Date");
        assert_eq!(format!("{}", OrganizationMode::Hybrid), "Hybrid");
    }

    #[test]
    fn test_rename_mode_display() {
        assert_eq!(format!("{}", RenameMode::Clean), "Clean");
        assert_eq!(format!("{}", RenameMode::DatePrefix), "Date prefix");
        assert_eq!(format!("{}", RenameMode::Skip), "Skip");
    }
}
