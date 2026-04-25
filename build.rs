fn main() {
    let rp2040   = cfg!(feature = "rp2040");
    let rp235xa  = cfg!(feature = "rp235xa");
    let rp235xb  = cfg!(feature = "rp235xb");

    let count = [rp2040, rp235xa, rp235xb]
        .iter()
        .filter(|&&x| x)
        .count();

    if count == 0 {
        panic!(
            "\n[embassy-hall-analog] Aucune cible sélectionnée.\n\
             Activez exactement une feature parmi : rp2040, rp235xa, rp235xb.\n"
        );
    }

    if count > 1 {
        panic!(
            "\n[embassy-hall-analog] Plusieurs cibles sélectionnées en même temps.\n\
             Ces features sont mutuellement exclusives : rp2040, rp235xa, rp235xb.\n\
             Désactivez les features par défaut et n'en activez qu'une seule.\n\
             Exemple : embassy-hall-analog = {{ version = \"0.1.0\", \
             default-features = false, features = [\"rp235xa\"] }}\n"
        );
    }

    // Expose un cfg unifié utilisable dans lib.rs
    if rp2040  { println!("cargo:rustc-cfg=hall_target=\"rp2040\""); }
    if rp235xa { println!("cargo:rustc-cfg=hall_target=\"rp235xa\""); }
    if rp235xb { println!("cargo:rustc-cfg=hall_target=\"rp235xb\""); }
}