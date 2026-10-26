//! Batas dan default yang bisa dinaikkan lewat environment tanpa membangun ulang.
//!
//! Konvensi yang sama dipakai service Go di repo ini: kunci `UPPER_SNAKE`. Nilai yang tidak bisa
//! dibaca atau bernilai 0 TIDAK pernah dipakai — alat jatuh ke default dan menyebut angka yang
//! berlaku di laporan, supaya tidak ada batas yang diam-diam hilang.

use std::env;

/// Inti murni dari `angka_env`: keputusan bulat bisa dites tanpa menyentuh proses lain.
pub(crate) fn angka_dari(nilai: Option<String>, default: u64) -> u64 {
    match nilai {
        Some(mut text) => {
            text = text.trim().replace('_', "");
            match text.parse::<u64>() {
                Ok(angka) if angka > 0 => angka,
                _ => default,
            }
        }
        None => default,
    }
}

pub(crate) fn angka_env(nama: &str, default: u64) -> u64 {
    angka_dari(env::var(nama).ok(), default)
}

/// Inti murni pilihan port: nilai apa yang diterima, tanpa menyentuh environment.
pub(crate) fn port_dari(nilai: Option<String>, default: u16) -> u16 {
    match nilai {
        Some(text) => match text.trim().parse::<u32>() {
            Ok(angka) if angka > 0 && angka <= 65535 => angka as u16,
            _ => default,
        },
        None => default,
    }
}

pub(crate) fn port_env(default: u16) -> u16 {
    port_dari(env::var("PETA_PORT").ok(), default)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn angka_dari_memakai_nilai_env_yang_sah() {
        assert_eq!(angka_dari(Some("50000".to_string()), 12_000), 50_000);
        assert_eq!(angka_dari(Some(" 24_000_000 ".to_string()), 1), 24_000_000);
    }

    #[test]
    fn angka_dari_jatuh_ke_default_untuk_nilai_tidak_sah_dan_nol() {
        // Nol bukan "tanpa batas": memakainya akan membuat sapuan berhenti diam-diam.
        assert_eq!(angka_dari(Some("0".to_string()), 12_000), 12_000);
        assert_eq!(angka_dari(Some("bukan-angka".to_string()), 12_000), 12_000);
        assert_eq!(angka_dari(Some("-5".to_string()), 12_000), 12_000);
        assert_eq!(angka_dari(Some("".to_string()), 12_000), 12_000);
        assert_eq!(angka_dari(None, 12_000), 12_000);
    }

    #[test]
    fn port_dari_menolak_luar_rentang_dan_kembali_ke_default() {
        assert_eq!(port_dari(Some("9000".to_string()), 8787), 9000);
        assert_eq!(port_dari(Some(" 8790 ".to_string()), 8787), 8790);
        assert_eq!(port_dari(Some("0".to_string()), 8787), 8787, "port 0 bukan pilihan");
        assert_eq!(port_dari(Some("65536".to_string()), 8787), 8787);
        assert_eq!(port_dari(Some("-1".to_string()), 8787), 8787);
        assert_eq!(port_dari(Some("http".to_string()), 8787), 8787);
        assert_eq!(port_dari(None, 8787), 8787);
    }
}
