use syntax::check::App;
use syntax::Result;

pub fn app(app: &App) -> Result<()> {
    if !cfg!(feature = "timer-queue") {
        if !app.init.async_after.is_empty()
            || app.tasks.values().any(|task| !task.async_after.is_empty())
        {
            return Err(format_err!(
                "async_after is not supported. Enable the 'timer-queue' feature to use it"
            ));
        }
    }
    Ok(())
}
