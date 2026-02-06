$lines = Get-Content crates/mapmap-media/src/player.rs
$new_lines = $lines[0..192] + $lines[195..($lines.Length-1)]
$new_lines | Set-Content crates/mapmap-media/src/player.rs
