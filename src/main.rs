
slint::include_modules!();

fn main() -> Result<(), slint::PlatformError> {
    let app = AppWindow::new()?;
    
    app.on_pressed( |double| {
        if double {
            println!("i even got double clicked\n");
        } else {
            println!("the tuff text got clicked\n");
        }
    });

    app.run()
}
