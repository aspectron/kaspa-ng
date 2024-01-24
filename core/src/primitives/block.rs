use crate::imports::*;
use kaspa_rpc_core::RpcBlock;

#[derive(Clone)]
pub struct BlockDagGraphSettings {
    pub y_scale: f64,
    pub y_dist: f64,
    pub noise: f64,
    pub graph_length_daa: usize,
    pub center_vspc: bool,
    pub balance_vspc: bool,
    pub reset_vspc: bool,
    pub show_vspc: bool,
    pub show_daa: bool,
    pub show_grid: bool,
}

impl Default for BlockDagGraphSettings {
    fn default() -> Self {
        Self {
            y_scale: 10.0,
            y_dist: 7.0,
            noise: 0.0,
            graph_length_daa: 1024,
            center_vspc: false,
            balance_vspc: true,
            reset_vspc: true,
            show_vspc: true,
            show_daa: true,
            show_grid: true,
        }
    }
}

impl BlockDagGraphSettings {
    pub fn new(spread: f64) -> Self {
        Self {
            y_scale: spread,
            ..Default::default()
        }
    }
}

#[derive(Clone, Debug)]
pub struct DagBlock {
    pub data: Arc<RpcBlock>,
    pub src_y: f64,
    pub dst_y: f64,
    pub offset_y: f64,
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
            offset_y: y,
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
        Self {
            blocks: vec![block],
            daa_score,
        }
    }

    pub fn push(&mut self, block: DagBlock, settings: &BlockDagGraphSettings) {
        self.blocks.push(block);
        self.reset(settings);
        self.update(settings);
    }

    pub fn update_vspc(&mut self, hash: KaspaHash, flag: bool, settings: &BlockDagGraphSettings) {
        if let Some(block) = self.blocks.iter_mut().find(|b| b.data.header.hash == hash) {
            block.vspc = flag;
            block.settled = false;
            if flag && settings.center_vspc {
                block.dst_y = 0.0;
            } else {
                block.dst_y = hash_to_y_coord(&block.data.header.hash, settings.y_scale);
            }
        }

        self.update(settings);
    }

    pub fn update(&mut self, settings: &BlockDagGraphSettings) {
        self.blocks
            // .sort_by(|a, b| a.src_y.partial_cmp(&b.src_y).unwrap());
            .sort_by(|a, b| a.dst_y.partial_cmp(&b.dst_y).unwrap());
        let y_distance = settings.y_dist;
        let noise = settings.noise;
        let len = self.blocks.len();

        if settings.balance_vspc {
            #[allow(clippy::collapsible_else_if)]
            if let Some(mut vspc_idx) = self.blocks.iter().position(|block| block.vspc) {
                if settings.center_vspc && len > 2 {
                    let mid = len / 2;
                    if vspc_idx != mid {
                        self.blocks.swap(vspc_idx, mid);
                        vspc_idx = mid;
                        self.blocks.iter_mut().for_each(|block| {
                            block.settled = false;
                        });
                    }
                }

                let vspc_y = if settings.center_vspc {
                    0.0
                } else {
                    self.blocks
                        .get(vspc_idx)
                        .map(|block| block.dst_y)
                        .unwrap_or_default()
                };

                let mut y = vspc_y;
                (0..vspc_idx).rev().for_each(|idx| {
                    let block = &mut self.blocks[idx];
                    y -= y_distance;
                    block.dst_y = y - block.offset_y * noise;
                });
                y = vspc_y;
                ((vspc_idx + 1)..len).for_each(|idx| {
                    let block = &mut self.blocks[idx];
                    y += y_distance;
                    block.dst_y = y + block.offset_y * noise;
                });
            } else {
                if len > 1 {
                    let mut y = -(len as f64 * y_distance / 2.0);
                    (0..len).for_each(|idx| {
                        let block = &mut self.blocks[idx];
                        y += y_distance;
                        block.dst_y = y + block.offset_y * noise;
                    });
                }
            }
        } else {
            (0..len).for_each(|idx| {
                let block = &mut self.blocks[idx];
                block.dst_y =
                    hash_to_y_coord(&block.data.header.hash, settings.y_scale) * y_distance * 0.3;
            });
        }
    }

    pub fn reset(&mut self, settings: &BlockDagGraphSettings) {
        if settings.reset_vspc {
            self.blocks.iter_mut().for_each(|block| {
                block.settled = false;
                if block.vspc && settings.center_vspc {
                    block.dst_y = 0.0;
                } else {
                    block.dst_y = hash_to_y_coord(&block.data.header.hash, settings.y_scale);
                }
            });
        }

        self.update(settings);
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
                    if dist.abs() < 0.001 {
                        block.settled = true;
                    }
                }
                (block.data.clone(), [x, y].into(), block.vspc, block.settled)
            })
            .collect::<Vec<_>>()
    }
}
