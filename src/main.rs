use whirlwind::App;

fn main() -> anyhow::Result<()> {
    App::new()?.run()?;
    Ok(())
}
