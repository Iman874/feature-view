//! Kode tampilan (CSS dan JS) dan kamus bentuk ikon, dibaca dari sumber crate itu sendiri.
//! Kalau sumber tidak ada (binari hasil install), jatuh ke salinan terbaking dalam binary.

use std::fs;
use std::path::{Path, PathBuf};

/// Direktori tempat sumber alat ini berada, diketahui saat kompilasi. Tidak diikat ke bentuk repo
/// pemakai: alat yang disalin ke project mana pun membaca `render.js`/`view.css` dari checkout-nya
/// sendiri, dan jatuh ke salinan terbaking kalau dijalankan sebagai binari hasil `cargo install`.
pub(crate) fn dir_aset() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("src")
}

/// Nama bentuk ikon yang tersedia di kode tampilan. Dibaca dari sumber `render.js` supaya `check`
/// bisa membuktikan tiap baris manifest menunjuk bentuk yang sungguh ada — dulu ini tidak terbaca
/// dan aplikasi tanpa entri ikon lolos senyap dengan ikon generik.
pub(crate) fn bentuk_terdaftar(js: &str) -> Vec<String> {
    let penanda = "var IKON = {";
    let awal = match js.find(penanda) {
        Some(i) => i + penanda.len(),
        None => return Vec::new(),
    };
    let sisa = &js[awal..];
    let akhir = match sisa.find("\n  };") {
        Some(i) => i,
        None => sisa.len(),
    };
    let mut nama: Vec<String> = Vec::new();
    for baris in sisa[..akhir].lines() {
        let baris = baris.trim();
        if baris.is_empty() || baris.starts_with("//") {
            continue;
        }
        if let Some(pos) = baris.find(':') {
            let kunci = baris[..pos]
                .trim()
                .trim_matches('\'')
                .trim_matches('"')
                .to_string();
            if !kunci.is_empty() && !kunci.contains(' ') && !kunci.contains('{') {
                nama.push(kunci);
            }
        }
    }
    nama.sort();
    nama.dedup();
    nama
}

/// serve mengambil CSS dan JS dari disk bila ketemu, supaya ubahan gaya atau logika
/// graf terlihat pada permintaan berikutnya tanpa build ulang. generate memakai sumber yang sama,
/// jadi halaman yang ditulis dan halaman yang disajikan tidak pernah beda versi.
pub(crate) fn aset(nama: &str, baked: &str) -> (String, &'static str) {
    aset_dari(&dir_aset(), nama, baked)
}

/// Bentuk murni dari `aset`, dengan folder bisa ditentukan sendiri supaya jalur "disk vs baking"
/// bisa dites tanpa menunggu binari dipasang di mesin lain.
pub(crate) fn aset_dari(dir: &Path, nama: &str, baked: &str) -> (String, &'static str) {
    match fs::read_to_string(dir.join(nama)) {
        Ok(isi) => (isi, "disk"),
        Err(_) => (baked.to_string(), "baking"),
    }
}

pub(crate) const CSS: &str = include_str!("view.css");

pub(crate) const JS: &str = include_str!("render.js");

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn aset_ambil_dari_disk_dan_jatuh_ke_baking() {
        let asal = "view.css";
        let baked = "ISI-BAKING";
        // Folder tanpa berkas aset -> pakai isi yang dibaking di binari.
        let kosong = std::env::temp_dir().join(format!("fv-uji-kosong-{}", std::process::id()));
        let (isi, label) = aset_dari(&kosong, asal, baked);
        assert_eq!(label, "baking");
        assert_eq!(isi, baked);

        // Folder yang punya -> dibaca dari disk, termasuk isi yang baru ditulis barusan.
        let hidup = std::env::temp_dir().join(format!("fv-uji-hidup-{}", std::process::id()));
        fs::create_dir_all(&hidup).expect("buat folder uji");
        fs::write(hidup.join(asal), "ISI-DISK").expect("tulis aset uji");
        let (isi, label) = aset_dari(&hidup, asal, baked);
        assert_eq!(label, "disk");
        assert_eq!(isi, "ISI-DISK");
        let _ = fs::remove_dir_all(&kosong);
        let _ = fs::remove_dir_all(&hidup);
    }

    #[test]
    fn dir_aset_menunjuk_sumber_kode_tampilan_sendiri() {
        // Aset diambil dari folder crate, bukan dari bentuk repo pemakai: di repo mana pun alat ini
        // dipakai, `checkout` selalu punya sumbernya dan `--root` tidak boleh mengubahnya.
        for nama in ["render.js", "view.css"] {
            let (isi, label) = aset(nama, "ISI-BAKING");
            assert_eq!(label, "disk", "sumber {nama} harus terbaca dari checkout alat");
            assert!(!isi.is_empty(), "{nama} terbaca kosong");
        }
        let (_, label_css) = aset("view.css", CSS);
        let (_, label_js) = aset("render.js", JS);
        assert_eq!(label_css, "disk");
        assert_eq!(label_js, "disk");
    }

    #[test]
    fn bentuk_terdaftar_membaca_kamus_ikon_dari_sumber_js() {
        let contoh = "  var IKON = {\n    cari: '<circle/>',\n    'lipat': '<path d=\"M1\"/>',\n  };\n  var LAIN = { x: 1 };";
        let bentuk = bentuk_terdaftar(contoh);
        assert!(bentuk.contains(&"cari".to_string()), "hasil: {bentuk:?}");
        assert!(bentuk.contains(&"lipat".to_string()), "kunci terkutip harus ikut: {bentuk:?}");
        assert!(!bentuk.contains(&"x".to_string()), "blok lain tidak boleh terbaca: {bentuk:?}");

        // Sumber aslinya sendiri harus terbaca, kalau tidak, validasi ikon jadi no-op diam-diam.
        let nyata = bentuk_terdaftar(JS);
        assert!(
            nyata.len() >= 40,
            "kamus bentuk ikon harus terbaca dari render.js, dapat {}",
            nyata.len()
        );
    }

}
