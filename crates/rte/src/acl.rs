use rte_error::ReturnValue as _;

use crate::{
    lcore,
    utils::{self, AsCString, AsRaw},
    Result,
};

pub type RawAclContext = ffi::rte_acl_ctx;
raw!(pub AclCtx(RawAclContext));

impl AclCtx {
    pub fn create<S: AsRef<str>>(name: S, rule_size: u32, max_rules: u32) -> Result<Self> {
        let name = name.as_c_str();
        let params = ffi::rte_acl_param {
            max_rule_num: max_rules,
            rule_size,
            name: name.as_ptr(),
            socket_id: lcore::SOCKET_ID_ANY,
        };
        let ptr = unsafe { ffi::rte_acl_create(&params) }.rte_ok()?;
        Ok(ptr.as_ptr().into())
    }

    pub fn add_rules<T>(&mut self, rules: &[T], num_of_rules: u32) -> Result<()> {
        unsafe { ffi::rte_acl_add_rules(self.as_raw(), rules.as_ptr() as *const ffi::rte_acl_rule, num_of_rules) }
            .rte_ok()?;
        Ok(())
    }

    pub fn build_rules(&mut self, fields: &[ffi::rte_acl_field_def]) -> Result<()> {
        let mut acl_cfg = ffi::rte_acl_config {
            max_size: 0,
            num_categories: 1,
            num_fields: fields.len() as u32,
            defs: [ffi::rte_acl_field_def { ..Default::default() }; 64usize],
        };

        for (cfg, rule) in acl_cfg.defs.iter_mut().zip(fields.iter()) {
            *cfg = *rule
        }
        unsafe { ffi::rte_acl_build(self.as_raw(), &acl_cfg) }.rte_ok()?;
        Ok(())
    }

    pub fn evaluate_packet(&mut self, payload: &[u8]) -> u32 {
        let mut rv: u32 = 0;
        unsafe {
            ffi::rte_acl_classify(self.as_raw(), &mut payload.as_ptr() as *mut *const u8, &mut rv as *mut u32, 1, 1)
        }
        .rte_ok()
        .unwrap();
        rv
    }

    pub fn free(&mut self) {
        unsafe { ffi::rte_acl_free(self.as_raw()) }
    }
}

impl Drop for AclCtx {
    fn drop(&mut self) {
        Self::free(self);
    }
}
