use std::{env, io};

fn main() -> io::Result<()> {
    if cfg!(target_os = "windows") {
        let mut res = winres::WindowsResource::new();
        res.set_icon("assets/icon.ico");
        let _ = res.set("ProductName", "SF Assistant");
        let _ = res.set("FileDescription", "SF Assistant");
        let _ = res.set("CompanyName", "");
        let ver = env::var("CARGO_PKG_VERSION").unwrap_or_else(|_| "0.0.0".into());
        let _ = res.set("ProductVersion", &ver);
        let _ = res.set("FileVersion", &ver);
        res.compile()?;
    }
    Ok(())
}
