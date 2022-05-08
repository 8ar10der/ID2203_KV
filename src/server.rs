use anyhow::Result;

use crate::utils::runnable::Runnable;

pub struct Server {}

impl Runnable for Server {
    fn run(&self) -> Result<()> {
        self.start()
    }
}

impl Server {
    pub fn new() -> Server {
        let server = Server {};

        server
    }

    pub fn start(&self) -> Result<()> {
        loop {}
    }
}
