use serde::{Deserialize, Serialize};
use structopt::StructOpt;

#[derive(Debug, StructOpt, Serialize, Deserialize)]
pub (crate) struct Node {

    #[structopt(long)]
    pub id: u64,
    
    #[structopt(long)]
    pub peers: Vec<u64>,

}