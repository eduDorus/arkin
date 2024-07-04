use crate::config::GlobalConfig;

pub struct Server {
    config: GlobalConfig,
}

impl Server {
    pub fn new(config: GlobalConfig) -> Self {
        Self { config }
    }

    pub fn run(&self) {
        println!("Running server with config: {:?}", self.config);
    }
}
