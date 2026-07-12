// Popush entry point. Built by twostep.
//
// Prevents an extra console window on non-Linux targets; harmless on Linux, where
// Popush is the only supported platform (D1).
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    popush_lib::run();
}
