use anyhow::Result;

pub(crate) trait Runnable: Sync {
    fn run(&self) -> Result<()>;
}