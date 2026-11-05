// Подавляем консольное окно на Windows в релизной сборке.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    fontlord_lib::run()
}
