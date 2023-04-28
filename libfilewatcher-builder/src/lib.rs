use shared_library_builder::{GitLocation, LibraryLocation, RustLibrary};

pub fn libfilewatcher(version: Option<impl Into<String>>) -> RustLibrary {
    RustLibrary::new(
        "Filewatcher",
        LibraryLocation::Git(
            GitLocation::github("feenkcom", "libfilewatcher").tag_or_latest(version),
        ),
    )
    .package("libfilewatcher")
}

pub fn latest_libfilewatcher() -> RustLibrary {
    let version: Option<String> = None;
    libfilewatcher(version)
}
