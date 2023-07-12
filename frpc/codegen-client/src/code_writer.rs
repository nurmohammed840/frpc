use super::*;

pub struct CodeWriter<'a> {
    codegen: CodeGen<'a>,
}

impl<'a> From<&'a TypeDef> for CodeWriter<'a> {
    fn from(td: &'a TypeDef) -> Self {
        Self {
            codegen: CodeGen::from(td),
        }
    }
}

impl CodeWriter<'_> {
    pub fn generate_typescript_binding(&self, config: &typescript::Config) -> Result {
        fs::create_dir_all(&config.out_dir)?;

        let prelude_path = config.out_dir.join("databuf.lib.ts");
        if !prelude_path.exists() {
            fs::write(
                prelude_path,
                include_bytes!("../client/typescript/databuf.ts"),
            )?;
            fs::write(
                config.out_dir.join("http.transport.ts"),
                include_bytes!("../client/typescript/http.transport.ts"),
            )?;
        }
        let ext = match config.preserve_import_extension {
            true => ".ts",
            false => "",
        };
        let mut code = format!("import * as use from './databuf.lib{ext}'\n");
        write!(code, "{}", self.codegen.typescript())?;
        let filename = format!("{}.ts", self.codegen.type_def.name);
        Ok(fs::write(config.out_dir.join(filename), code)?)
    }
}
