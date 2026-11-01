// Copyright © SixtyFPS GmbH <info@slint.dev>
// SPDX-License-Identifier: MIT

// cSpell: ignore libunwind lunwind mozjs
use std::env;
use std::fs::File;
use std::path::Path;

use gl_generator::{Api, Fallbacks, Profile, Registry, StructGenerator};

fn main() {
    let out = env::var("OUT_DIR").unwrap();
    let out = Path::new(&out);

    {
        let mut file = File::create(&out.join("gl_bindings.rs")).unwrap();

        // Config copied from https://github.com/YaLTeR/bxt-rs/blob/9f621251b8ce5c2af00b67d2feab731e48d1dae9/build.rs.

        Registry::new(
            Api::Gles2,
            (3, 0),
            Profile::Core,
            Fallbacks::All,
            [
                "GL_EXT_memory_object",
                "GL_EXT_memory_object_fd",
            ],
        )
        .write_bindings(StructGenerator, &mut file)
        .unwrap();
    }

    slint_build::compile("ui/main.slint").unwrap();
}
