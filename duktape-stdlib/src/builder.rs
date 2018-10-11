bitflags!{
    pub struct Modules: i32 {
        const Io = 1 << 0;
        const Fs= 1 << 1;
        const Utils = 1 << 2;
    }
}

impl Default for Modules {
    fn default() -> Modules {
        Modules::Io | Modules::Fs | Modules::Utils
    }
}