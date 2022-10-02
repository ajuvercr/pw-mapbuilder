#[cfg(not(target_family = "wasm"))]
pub mod print {
    #[macro_export]
    macro_rules! printit {
        ($($arg:tt)*) => {{
            println!($($arg)*)
        }};
    }

    #[macro_export]
    macro_rules! eprintit {
        ($($arg:tt)*) => {{
            eprintln!($($arg)*)
        }};
    }
}

#[cfg(target_family = "wasm")]
pub mod print {
    pub mod js {

        use wasm_bindgen::prelude::wasm_bindgen;
        #[wasm_bindgen]
        extern "C" {
            #[wasm_bindgen(js_namespace = console)]
            pub fn log(s: &str);
            #[wasm_bindgen(js_namespace = console)]
            pub fn error(s: &str);
        }
    }

    #[macro_export]
    macro_rules! printit {
        ($($arg:tt)*) => {{
            $crate::utils::print::js::log(&format!($($arg)*))
        }};
    }

    #[macro_export]
    macro_rules! eprintit {
        ($($arg:tt)*) => {{
            $crate::utils::print::js::error(&format!($($arg)*))
        }};
    }
}

pub mod rng {
    use rand::rngs::StdRng;
    use rand::SeedableRng;
    pub type RNG = StdRng;

    pub fn new() -> RNG {
        StdRng::from_entropy()
    }
}
