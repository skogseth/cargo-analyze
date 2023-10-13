use std::path::Path;

use anyhow::{anyhow, Context};
use goblin::mach::Mach;
use goblin::Object;

pub fn analyze(filepath: &Path) -> Result<(), anyhow::Error> {
    let buffer = std::fs::read(filepath)
        .with_context(|| format!("Failed to read file at {} to buffer", filepath.display()))?;
    let object = Object::parse(&buffer).context("Failed to parse buffer as goblin object")?;

    match object {
        Object::Elf(elf) => {
            println!("Elf");
            println!("Libs: {:#?}", elf.libraries);
            Ok(())
        }
        Object::Mach(Mach::Binary(mach)) => {
            println!("Binary mach");
            println!("Libs: {:#?}", mach.libs);
            Ok(())
        }
        //
        // --------------------------
        //
        Object::PE(_) => Err(anyhow!("Object was unexpected goblin object of type `PE`")),
        Object::Mach(Mach::Fat(_)) => Err(anyhow!(
            "Object was unexpected goblin object of type `Fat mach`"
        )),
        Object::Archive(_) => Err(anyhow!(
            "Object was unexpected goblin object of type `Archive`"
        )),
        Object::Unknown(_) => Err(anyhow!("Object was unknown goblin object")),
    }
}
