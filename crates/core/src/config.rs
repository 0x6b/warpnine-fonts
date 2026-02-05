//! Configuration constants for Warpnine font builds.

/// Recursive VF source font filename.
pub const RECURSIVE_VF_FILENAME: &str = "Recursive_VF_1.085.ttf";

/// Noto Sans Mono CJK JP VF source font filename.
pub const NOTO_CJK_VF_FILENAME: &str = "NotoSansMonoCJKjp-VF.ttf";

/// JetBrains Mono Regular filename (for box drawing characters).
pub const JETBRAINS_MONO_FILENAME: &str = "JetBrainsMono-Regular.ttf";

/// Recursive font version.
pub const RECURSIVE_VERSION: &str = "1.085";

/// JetBrains Mono version.
pub const JETBRAINS_MONO_VERSION: &str = "2.304";

/// Recursive VF download URL (ZIP archive).
pub const RECURSIVE_ZIP_URL: &str =
    "https://github.com/arrowtype/recursive/releases/download/v1.085/ArrowType-Recursive-1.085.zip";

/// Path to the VF font inside the Recursive ZIP archive.
pub const RECURSIVE_ZIP_PATH: &str =
    "ArrowType-Recursive-1.085/Recursive_Desktop/Recursive_VF_1.085.ttf";

/// JetBrains Mono download URL (ZIP archive).
pub const JETBRAINS_MONO_ZIP_URL: &str =
    "https://github.com/JetBrains/JetBrainsMono/releases/download/v2.304/JetBrainsMono-2.304.zip";

/// Path to the Regular font inside the JetBrains Mono ZIP archive.
pub const JETBRAINS_MONO_ZIP_PATH: &str = "fonts/ttf/JetBrainsMono-Regular.ttf";

/// Noto CJK commit hash for reproducible builds.
pub const NOTO_CJK_COMMIT: &str = "f8d157532fbfaeda587e826d4cd5b21a49186f7c";

/// Noto Sans Mono CJK JP VF download URL.
pub const NOTO_CJK_VF_URL: &str = "https://raw.githubusercontent.com/notofonts/noto-cjk/f8d157532fbfaeda587e826d4cd5b21a49186f7c/Sans/Variable/TTF/Mono/NotoSansMonoCJKjp-VF.ttf";

/// Noto CJK license download URL.
pub const NOTO_CJK_LICENSE_URL: &str = "https://raw.githubusercontent.com/notofonts/noto-cjk/f8d157532fbfaeda587e826d4cd5b21a49186f7c/Sans/LICENSE";

/// Recursive license download URL.
pub const RECURSIVE_LICENSE_URL: &str =
    "https://raw.githubusercontent.com/arrowtype/recursive/refs/tags/v1.085/OFL.txt";

/// JetBrains Mono license download URL.
pub const JETBRAINS_MONO_LICENSE_URL: &str =
    "https://raw.githubusercontent.com/JetBrains/JetBrainsMono/v2.304/OFL.txt";
