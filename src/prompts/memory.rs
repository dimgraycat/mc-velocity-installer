use std::io;

use super::input::{prompt_with_default, prompt_yes_no};

const DEFAULT_XMS: &str = "256M";
const DEFAULT_XMX: &str = "512M";

pub(crate) fn prompt_memory() -> io::Result<(String, String)> {
    loop {
        let xms = prompt_with_default("起動メモリ Xms", DEFAULT_XMS)?;
        let xmx = prompt_with_default("最大メモリ Xmx", DEFAULT_XMX)?;
        let confirm = prompt_yes_no(&format!("Xms={} / Xmx={} でよいですか？", xms, xmx), true)?;
        if confirm {
            return Ok((xms, xmx));
        }
    }
}
