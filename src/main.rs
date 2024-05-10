use anyhow::Result;
use chopsticks::tui::{self, model::App};

#[tokio::main]
async fn main() -> Result<()> {
    tui::install_panic_hook();
    let mut terminal = tui::init_terminal()?;
    let mut app = App::new();

    app.init();
    while !app.quit {
        terminal.draw(|f| app.view(f))?;
        if let Some(msg) = app.handle_event().await {
            app.update(msg);
        }
    }

    if !app.terminal_restored {
        tui::restore_terminal()?;
    }

    Ok(())
}
