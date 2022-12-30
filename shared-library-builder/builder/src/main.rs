use std::error::Error;

use shared_library_builder::build_standalone;

use libfilewatcher_library::latest_libfilewatcher;

fn main() -> Result<(), Box<dyn Error>> {
    build_standalone(|_| Ok(Box::new(latest_libfilewatcher())))
}
