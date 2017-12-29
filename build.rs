// Build script for the entire project.
#[cfg(target_os = "windows")]
extern crate winres;

// This is the function with stuff needed for the windows build, like adding the icon to the program.
#[cfg(target_os = "windows")]
fn main() {
    let mut res = winres::WindowsResource::new();
    res.set_icon("img/rpfm.ico");
    if let Err(error) = res.compile() {
        println!("Error: {}", std::error::Error::description(&error).to_string());
    }
}

#[cfg(not(target_os = "windows"))]
fn main() {

}