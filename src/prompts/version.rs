use std::io;

use crate::version::VersionInfo;

use super::input::{prompt_usize_with_default, prompt_yes_no};

pub(crate) fn prompt_version(versions: &[VersionInfo]) -> io::Result<VersionInfo> {
    loop {
        println!();
        println!("利用可能なバージョン一覧:");
        for (idx, version) in versions.iter().enumerate() {
            println!("{:>3}. {}", idx + 1, version.display_label());
        }
        let selection = prompt_usize_with_default("番号で選択してください", 1, 1..=versions.len())?;
        let chosen = versions[selection - 1].clone();
        let confirm = prompt_yes_no(
            &format!("{} を選択しますか？", chosen.display_label()),
            true,
        )?;
        if confirm {
            return Ok(chosen);
        }
    }
}
