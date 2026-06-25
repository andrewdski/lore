// SPDX-FileCopyrightText: 2026 Epic Games, Inc.
// SPDX-License-Identifier: MIT
#![allow(clippy::disallowed_macros)]
use std::error::Error;

include!("../build-helper.rs");

fn main() -> Result<(), Box<dyn Error>> {
    // Populate environment with build details
    vergen::Emitter::default()
        .add_custom_instructions(&LoreVergen::default())?
        .emit()?;

    // Compile metadata into lore.exe for Windows
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap() == "windows" {
        // Create WindowsResource with defaults from the crate
        let mut winres = winresource::WindowsResource::new();

        // Set language to en-US
        winres.set_language(0x0409);

        winres.compile().unwrap();
    }

    Ok(())
}
