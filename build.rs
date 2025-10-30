use std::io;

fn main() -> io::Result<()> {
    if cfg!(target_os = "windows") {
        let mut res = winres::WindowsResource::new();
        res.set_icon("assets/icon.ico");
        let _ = res.set("ProductName", "ShakesAutomation");
        let _ = res.set("FileDescription", "ShakesAutomation");
        let _ = res.set("CompanyName", "");
        let _ = res.set("ProductVersion", "1.0.0");
        let _ = res.set("FileVersion", "1.0.0");
        res.compile()?;
    }
    Ok(())
}
