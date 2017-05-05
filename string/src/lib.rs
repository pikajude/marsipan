#[macro_export]
macro_rules! string {
    ($x:expr) => { $x.iter().map(|&c| c as char).collect::<String>() }
}
