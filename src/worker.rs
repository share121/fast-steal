use crate::task::Tasks;
use std::sync::mpsc;

pub struct Worker<Idx> {
    pub tx_task: mpsc::Sender<Tasks<Idx>>,
    pub remain: Idx,
    pub tasks: Tasks<Idx>,
}
