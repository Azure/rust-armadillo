use std::{
    collections::HashMap,
    ffi::CStr,
    ptr::{null, null_mut},
};

use rte_error::ReturnValue;

use super::EthDev;
use crate::Result;

/// The names of all xstats for a device, should be creates once using [`EthDev::get_xstats_def`]
/// and then re-used with [`EthDev::get_xstats`] for retrieving the current xstats.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct XStatsDefs(Vec<String>);

impl EthDev {
    fn get_xstats_count(&self) -> Result<u32> {
        let count = unsafe { ffi::rte_eth_xstats_get_names_by_id(self.port_id, null_mut(), 0, null_mut()) }.rte_ok()?;
        Ok(count as u32)
    }

    /// Returns an [`XStatsDefs`] that contains the names of all available xstats for this device.
    pub fn get_xstats_def(&self) -> Result<XStatsDefs> {
        // first retrieve just the number of available xstats
        let stats_count = self.get_xstats_count()?;

        let mut stats_names = Vec::<ffi::rte_eth_xstat_name>::with_capacity(stats_count as usize);

        let names = unsafe {
            let names_written = ffi::rte_eth_xstats_get_names_by_id(
                self.port_id,
                stats_names.spare_capacity_mut().as_mut_ptr().cast(),
                stats_count,
                null_mut(),
            )
            .rte_ok()?;

            // Sanity check
            assert!(names_written as u32 <= stats_count);

            stats_names.set_len(names_written as usize);

            stats_names
                .iter()
                .map(|ffi::rte_eth_xstat_name { name }| CStr::from_ptr(name.as_ptr()).to_str().unwrap().to_string())
                .collect()
        };

        Ok(XStatsDefs(names))
    }

    /// Returns a mapping of xstats (by name), to their current values.
    pub fn get_xstats<'x>(&self, XStatsDefs(defs): &'x XStatsDefs) -> Result<HashMap<&'x str, u64>> {
        let mut values = Vec::<u64>::with_capacity(defs.len());

        unsafe {
            let values_written = ffi::rte_eth_xstats_get_by_id(
                self.port_id,
                null(),
                values.spare_capacity_mut().as_mut_ptr().cast(),
                defs.len() as u32,
            )
            .rte_ok()?;

            // Sanity check
            assert!(values_written as usize <= defs.len());

            values.set_len(values_written as usize);
        }

        Ok(defs.iter().zip(values).map(|(id, value)| (id.as_str(), value)).collect())
    }
}
