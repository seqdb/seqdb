use std::collections::BTreeMap;

use allocative::Allocative;

use crate::{Error, Result};

use super::{Region, Regions};

#[derive(Debug, Default, Allocative)]
pub struct Layout {
    start_to_region: BTreeMap<u64, Region>,
    start_to_hole: BTreeMap<u64, u64>,
    start_to_reserved: BTreeMap<u64, u64>,
}

impl From<&Regions> for Layout {
    fn from(regions: &Regions) -> Self {
        let mut start_to_region = BTreeMap::new();

        regions
            .index_to_region()
            .iter()
            .flatten()
            .for_each(|region| {
                start_to_region.insert(region.meta().read().start(), region.clone());
            });

        let mut start_to_hole = BTreeMap::new();

        let mut prev_end = 0;

        start_to_region.iter().for_each(|(&start, region)| {
            if prev_end != start {
                start_to_hole.insert(prev_end, start - prev_end);
            }
            let reserved = region.meta().read().reserved();
            prev_end = start + reserved;
        });

        Self {
            start_to_region,
            start_to_hole,
            start_to_reserved: BTreeMap::default(),
        }
    }
}

impl Layout {
    pub fn start_to_region(&self) -> &BTreeMap<u64, Region> {
        &self.start_to_region
    }

    pub fn start_to_hole(&self) -> &BTreeMap<u64, u64> {
        &self.start_to_hole
    }

    pub fn len(&self) -> u64 {
        let mut len = 0;
        let mut start = 0;
        if let Some((start_reserved, reserved)) = self.get_last_reserved() {
            start = start_reserved;
            len = start + reserved;
        }
        if let Some((hole_start, gap)) = self.get_last_hole()
            && hole_start > start
        {
            start = hole_start;
            len = start + gap;
        }
        if let Some((region_start, region)) = self.get_last_region()
            && region_start > start
        {
            len = region_start + region.meta().read().reserved();
        }
        len
    }

    pub fn get_last_region(&self) -> Option<(u64, &Region)> {
        self.start_to_region
            .last_key_value()
            .map(|(start, region)| (*start, region))
    }

    fn get_last_hole(&self) -> Option<(u64, u64)> {
        self.start_to_hole
            .last_key_value()
            .map(|(start, gap)| (*start, *gap))
    }

    fn get_last_reserved(&self) -> Option<(u64, u64)> {
        self.start_to_reserved
            .last_key_value()
            .map(|(start, reserved)| (*start, *reserved))
    }

    pub fn is_last_anything(&self, region: &Region) -> bool {
        if let Some((last_start, last_region)) = self.get_last_region()
            && last_region.index() == region.index()
            && self
                .get_last_hole()
                .is_none_or(|(hole_start, _)| last_start > hole_start)
            && self
                .get_last_reserved()
                .is_none_or(|(reserved_start, _)| last_start > reserved_start)
        {
            true
        } else {
            false
        }
    }

    pub fn insert_region(&mut self, start: u64, region: &Region) {
        assert!(self.start_to_region.insert(start, region.clone()).is_none())
        // TODO: Other checks related to holes and reserved ?
    }

    pub fn move_region(&mut self, new_start: u64, region: &Region) -> Result<()> {
        self.remove_region(region)?;
        self.insert_region(new_start, region);
        Ok(())
    }

    pub fn remove_region(&mut self, region: &Region) -> Result<()> {
        let region_meta = region.meta().read();
        let start = region_meta.start();
        let mut reserved = region_meta.reserved();

        let removed = self.start_to_region.remove(&start);

        if removed
            .as_ref()
            .is_none_or(|region_| region.index() != region_.index())
        {
            dbg!((region, removed));
            return Err(Error::Str(
                "Something went wrong, indexes of removed region should be the same",
            ));
        }

        reserved += self
            .start_to_hole
            .remove(&(start + reserved))
            .unwrap_or_default();

        if let Some((&hole_start, gap)) = self.start_to_hole.range_mut(..start).next_back()
            && hole_start + *gap == start
        {
            *gap += reserved;
        } else {
            self.start_to_hole.insert(start, reserved);
        }

        Ok(())
    }

    pub fn get_hole(&self, start: u64) -> Option<u64> {
        self.start_to_hole.get(&start).copied()
    }

    pub fn find_smallest_adequate_hole(&self, reserved: u64) -> Option<u64> {
        let mut best_gap = None;

        for (&start, &gap) in &self.start_to_hole {
            if gap >= reserved {
                match best_gap {
                    None => best_gap = Some((gap, start)),
                    Some((best_gap_val, best_start)) => {
                        if gap < best_gap_val || (gap == best_gap_val && start < best_start) {
                            best_gap = Some((gap, start));
                        }
                    }
                }
            }
        }

        best_gap.map(|(_, s)| s)
    }

    pub fn remove_or_compress_hole(&mut self, start: u64, compress_by: u64) {
        if let Some(gap) = self.start_to_hole.remove(&start)
            && gap != compress_by
        {
            if gap > compress_by {
                self.start_to_hole
                    .insert(start + compress_by, gap - compress_by);
            } else {
                panic!("Hole too small");
            }
        }
    }

    pub fn reserve(&mut self, start: u64, reserved: u64) {
        if self.start_to_reserved.insert(start, reserved).is_some() {
            unreachable!();
        }
    }

    pub fn reserved(&mut self, start: u64) -> Option<u64> {
        self.start_to_reserved.remove(&start)
    }
}
