use crate::imports::*;
use kaspa_rpc_core::RpcBlock;

#[derive(Clone)]
pub struct BlockDagGraphSettings {
    y_scale: f64,
    y_dist: f64,
    pub graph_length_daa: usize,
    vspc_center: bool,
}

impl Default for BlockDagGraphSettings {
    fn default() -> Self {
        Self {
            y_scale: 10.0,
            // y_dist: 70.0,
            y_dist: 7.0,
            graph_length_daa: 1024,
            vspc_center: false,
        }
    }
}

#[derive(Clone, Debug)]
pub struct DagBlock {
    pub data: Arc<RpcBlock>,
    pub src_y: f64,
    pub dst_y: f64,
    vspc: bool,
    settled: bool,
}

impl DagBlock {
    pub fn new(data: Arc<RpcBlock>, settings: &BlockDagGraphSettings) -> Self {
        let y = hash_to_y_coord(&data.header.hash, settings.y_scale);
        Self {
            data,
            src_y: y,
            dst_y: y,
            vspc: false,
            settled: false,
        }
    }

    pub fn block(&self) -> &Arc<RpcBlock> {
        &self.data
    }

    pub fn direct_parents(&self) -> &[KaspaHash] {
        self.data.header.direct_parents()
    }
}

pub struct DaaBucket {
    pub blocks: Vec<DagBlock>,
    pub daa_score: f64,
}

impl DaaBucket {
    pub fn new(daa_score: f64, block: DagBlock) -> Self {
        // let daa_score = block.data.header.daa_score as f64;
        Self {
            blocks: vec![block],
            daa_score,
        }
    }

    pub fn push(&mut self, block: DagBlock, settings: &BlockDagGraphSettings) {
        self.blocks.push(block);
        self.update(settings);
    }

    pub fn update_vspc(&mut self, hash: KaspaHash, flag: bool, settings: &BlockDagGraphSettings) {
        if let Some(block) = self.blocks.iter_mut().find(|b| b.data.header.hash == hash) {
            if flag {
                block.vspc = true;
                block.settled = false;
                if settings.vspc_center {
                    block.dst_y = 0.0;
                }
            } else {
                block.vspc = false;
                block.settled = false;
                block.dst_y = hash_to_y_coord(&block.data.header.hash, settings.y_scale);
            }
        }

        self.update(settings);
    }

    pub fn update(&mut self, settings: &BlockDagGraphSettings) {
        self.blocks
            .sort_by(|a, b| a.src_y.partial_cmp(&b.src_y).unwrap());
        let y_distance = settings.y_dist;
        if let Some(mut vspc_idx) = self.blocks.iter().position(|block| block.vspc) {
            let len = self.blocks.len();
            if settings.vspc_center && len > 2 {
                let mid = len / 2;
                if vspc_idx != mid {
                    self.blocks.swap(vspc_idx, mid);
                    vspc_idx = mid;
                    self.blocks.iter_mut().for_each(|block| {
                        block.settled = false;
                    });
                }
            }

            let mut y = 0.0;
            (0..vspc_idx).rev().for_each(|idx| {
                let block = &mut self.blocks[idx];
                y -= y_distance;
                block.dst_y = y;
            });
            y = 0.0;
            ((vspc_idx + 1)..len).for_each(|idx| {
                let block = &mut self.blocks[idx];
                y += y_distance;
                block.dst_y = y;
            });
        }
    }

    pub fn render(&mut self) -> Vec<(Arc<RpcBlock>, PlotPoint, bool, bool)> {
        self.blocks
            .iter_mut()
            .map(|block| {
                let x = self.daa_score;
                let y = block.src_y;
                if !block.settled {
                    let dist = block.src_y - block.dst_y;
                    // block.src_y -= dist * 0.01;
                    block.src_y -= dist * 0.1;
                    if dist.abs() < 0.00001 {
                        block.settled = true;
                    }
                }
                (block.data.clone(), [x, y].into(), block.vspc, block.settled)
            })
            .collect::<Vec<_>>()
    }
}
