// Stellar - Organization and Rename Modes
// @musem23
//
// Type-safe enums for organization and rename modes.
// Replaces magic numbers (usize) with proper types.

use std::fmt;

/// How files are organized into folders
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OrganizationMode {
    /// Group by file type: Documents/, Images/, Videos/
    #[default]
    Category,
    /// Group by date: 2024/01-january/
    Date,
    /// Hybrid: Documents/2024/, Images/2024/
    Hybrid,
}

impl OrganizationMode {
    pub fn from_index(idx: usize) -> Self {
        match idx {
            0 => Self::Category,
            1 => Self::Date,
            2 => Self::Hybrid,
            _ => Self::Category,
        }
    }

    pub fn to_index(self) -> usize {
        match self {
            Self::Category => 0,
            Self::Date => 1,
            Self::Hybrid => 2,
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "category" | "cat" | "c" => Self::Category,
            "date" | "d" => Self::Date,
            "hybrid" | "h" => Self::Hybrid,
            _ => Self::Category,
        }
    }
}

impl fmt::Display for OrganizationMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Category => write!(f, "Category"),
            Self::Date => write!(f, "Date"),
            Self::Hybrid => write!(f, "Hybrid"),
        }
    }
}

/// How files are renamed during organization
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[allow(dead_code)]
pub enum RenameMode {
    /// Clean: lowercase, dashes, remove accents (élève → eleve)
    #[default]
    Clean,
    /// Prefix with date: 2024-01-15-filename.pdf
    DatePrefix,
    /// Skip renaming entirely
    Skip,
}

#[allow(dead_code)]
impl RenameMode {
    pub fn from_index(idx: usize) -> Option<Self> {
        match idx {
            0 => Some(Self::Clean),
            1 => Some(Self::DatePrefix),
            2 => None, // Skip
            _ => Some(Self::Clean),
        }
    }

    pub fn to_index(self) -> usize {
        match self {
            Self::Clean => 0,
            Self::DatePrefix => 1,
            Self::Skip => 2,
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "clean" | "c" => Some(Self::Clean),
            "date-prefix" | "date" | "d" => Some(Self::DatePrefix),
            "skip" | "none" | "s" => None,
            _ => Some(Self::Clean),
        }
    }
}

impl fmt::Display for RenameMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Clean => write!(f, "Clean"),
            Self::DatePrefix => write!(f, "Date prefix"),
            Self::Skip => write!(f, "Skip"),
        }
    }
}

// Conversion to renamer::RenameMode
impl From<RenameMode> for crate::renamer::RenameMode {
    fn from(mode: RenameMode) -> Self {
        match mode {
            RenameMode::Clean => crate::renamer::RenameMode::Clean,
            RenameMode::DatePrefix => crate::renamer::RenameMode::DatePrefix,
            RenameMode::Skip => crate::renamer::RenameMode::Clean, // Fallback, won't be used
        }
    }
}
