use std::io::{self, Write};

pub(crate) fn prompt_with_default(message: &str, default: &str) -> io::Result<String> {
    let input = prompt_line(&format!("{message} [{default}]: "))?;
    if input.trim().is_empty() {
        Ok(default.to_string())
    } else {
        Ok(input)
    }
}

pub(crate) fn prompt_usize_with_default(
    message: &str,
    default: usize,
    range: std::ops::RangeInclusive<usize>,
) -> io::Result<usize> {
    loop {
        let input = prompt_line(&format!("{message} [{default}]: "))?;
        let value = if input.trim().is_empty() {
            default
        } else {
            match input.trim().parse::<usize>() {
                Ok(value) => value,
                Err(_) => {
                    println!("数値を入力してください。");
                    continue;
                }
            }
        };
        if range.contains(&value) {
            return Ok(value);
        }
        println!("範囲内の番号を入力してください。");
    }
}

pub(crate) fn prompt_yes_no(message: &str, default: bool) -> io::Result<bool> {
    let suffix = if default { "[Y/n]" } else { "[y/N]" };
    loop {
        let input = prompt_line(&format!("{message} {suffix}: "))?;
        let trimmed = input.trim().to_ascii_lowercase();
        if trimmed.is_empty() {
            return Ok(default);
        }
        match trimmed.as_str() {
            "y" | "yes" => return Ok(true),
            "n" | "no" => return Ok(false),
            _ => println!("y または n を入力してください。"),
        }
    }
}

pub(crate) fn prompt_line(message: &str) -> io::Result<String> {
    print!("{message}");
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().to_string())
}
