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

    // ---------------------------------------------------------------------------
    // Penjaga skala tampilan. Halaman ini dulu tumbuh nilai per elemen: 16 ukuran
    // huruf berbeda, 5 radius, 9 jarak, dan tiga tinggi kontrol dalam satu baris.
    // Kerapihan itu tidak bisa dijaga oleh mata — hanya oleh tes yang menghitung
    // ulang isinya, supaya satu baris `font-size: 0.79rem` baru tidak lolos diam-diam.
    // ---------------------------------------------------------------------------

    /// Buang komentar CSS supaya contoh sintaks di dalamnya tidak dibaca sebagai aturan.
    fn buang_komentar(css: &str) -> String {
        let mut hasil = String::with_capacity(css.len());
        let mut sisa = css;
        while let Some(awal) = sisa.find("/*") {
            hasil.push_str(&sisa[..awal]);
            match sisa[awal + 2..].find("*/") {
                Some(akhir) => sisa = &sisa[awal + 2 + akhir + 2..],
                None => return hasil,
            }
        }
        hasil.push_str(sisa);
        hasil
    }

    /// Pecah CSS menjadi daftar deklarasi `properti: nilai`. Kurung dikorbankan jadi
    /// pemisah karena sebagian aturan di berkas ini ditulis satu baris dengan beberapa
    /// deklarasi sekaligus; memeriksa awal baris akan kehilangan semuanya.
    fn deklarasi(teks: &str, properti: &str) -> Vec<String> {
        let awalan = format!("{properti}:");
        buang_komentar(teks)
            .replace(['{', '}'], ";")
            .split(';')
            .map(str::trim)
            .filter_map(|bagian| bagian.strip_prefix(&awalan))
            .map(|v| v.trim().to_string())
            .collect()
    }

    #[test]
    fn tipografi_view_css_berhenti_pada_enam_langkah_token() {
        // `100%` (akar html) dan `inherit` (teks yang ikut induknya) bukan langkah
        // tipografi, jadi keduanya diizinkan sebagai struktur, bukan sebagai ukuran.
        let nilai = deklarasi(CSS, "font-size");
        assert!(nilai.len() >= 30, "parser kehilangan deklarasi: hanya {} terbaca", nilai.len());
        let langkah: std::collections::BTreeSet<&str> = nilai
            .iter()
            .map(|v| v.trim_start_matches("var(").trim_end_matches(')'))
            .filter(|v| v.starts_with("--fs-"))
            .collect();
        assert!(langkah.len() <= 6, "lebih dari enam langkah huruf: {langkah:?}");
        for v in &nilai {
            assert!(
                v.starts_with("var(--fs-") || v == "100%" || v == "inherit",
                "ukuran huruf di luar skala: {v}"
            );
        }
        for i in 1..=6 {
            assert!(CSS.contains(&format!("--fs-{i}:")), "token --fs-{i} dipakai tapi tidak didefinisikan");
        }
    }

    #[test]
    fn radius_dan_jarak_view_css_hanya_token() {
        for v in deklarasi(CSS, "border-radius") {
            assert!(
                matches!(
                    v.as_str(),
                    "var(--r-sm)" | "var(--r-md)" | "var(--r-lg)" | "var(--r-pill)" | "50%" | "0"
                ),
                "radius di luar tangga: {v}"
            );
        }
        // Jarak: nilai pecahan (4.8px, 2.56px) adalah tanda ada orang menyetel per elemen.
        for properti in ["padding", "gap", "margin"] {
            for v in deklarasi(CSS, properti) {
                assert!(
                    !v.chars().any(|c| c == '.') || v.contains("rem") || v.contains("var("),
                    "jarak {properti} memakai angka pecahan: {v}"
                );
            }
        }
    }

    #[test]
    fn setiap_variabel_di_view_css_punya_definisi() {
        // Variabel yatim tidak error: ia diam-diam jatuh ke nilai cadangan atau `unset`,
        // jadi halaman tetap ter-render padahal warnanya sudah bukan dari skala.
        // Yang boleh tidak terdefinisi di CSS hanyalah variabel yang JS suntikkan sendiri
        // lewat style.setProperty (penanda grup dan jenis relasi per simpul) — dan itu
        // hanya sah kalau penulisannya memakai bentuk bercadangan `var(--x, cadangan)`.
        let dipakai: Vec<(String, bool)> = CSS
            .match_indices("var(--")
            .map(|(i, _)| {
                let sisa = &CSS[i + 6..];
                let akhir = sisa
                    .find(|c: char| c == ')' || c == ',' || c.is_whitespace())
                    .unwrap_or(0);
                let bercadangan = sisa.as_bytes().get(akhir) == Some(&b',');
                (sisa[..akhir].to_string(), bercadangan)
            })
            .collect();
        assert!(dipakai.len() > 100, "parser kehilangan sebagian pemakaian variabel: {}", dipakai.len());
        let didefinisikan: Vec<String> = CSS
            .match_indices("\n  --")
            .map(|(i, _)| {
                CSS[i + 2..]
                    .split(':')
                    .next()
                    .unwrap_or("")
                    .trim()
                    .trim_start_matches("--")
                    .to_string()
            })
            .collect();
        assert!(didefinisikan.len() > 40, "parser kehilangan definisi token: {}", didefinisikan.len());
        let hilang: Vec<&String> = dipakai
            .iter()
            .filter(|(n, cadangan)| !cadangan && !didefinisikan.contains(n))
            .map(|(n, _)| n)
            .collect();
        assert!(hilang.is_empty(), "variabel dipakai tanpa definisi dan tanpa cadangan: {hilang:?}");
    }

    /// Selector dari setiap aturan yang menyembunyikan sesuatu (`display: none`). Penjaga mode
    /// sembunyi membacanya sebagai daftar anggota, bukan sebagai potongan teks: pencocokan
    /// longgar meloloskan `.papan-apa-aja` dan membuat penjaganya sendiri terlihat hijau
    /// padahal tidak mengikat apa pun.
    fn daftar_sembunyi() -> Vec<Vec<String>> {
        CSS.split('}')
            .filter(|bagian| bagian.contains("display: none"))
            .map(|bagian| {
                bagian
                    .split('{')
                    .next()
                    .unwrap_or("")
                    .split(',')
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                    .map(str::to_string)
                    .collect()
            })
            .collect()
    }

    fn disembunyikan(pilihan: &str) -> bool {
        daftar_sembunyi().iter().any(|daftar| daftar.iter().any(|s| s == pilihan))
    }

    #[test]
    fn papan_ikut_disembunyikan_saat_rail_diperkecil() {
        // Rail kecil hanya menampung ikon. Satu blok teks yang tertinggal di dalamnya membuat
        // seluruh kolom menolak menyusut: sudah diukur, sidebar 59px jadi 281px hanya karena
        // panel papan tidak ikut daftar sembunyi.
        assert!(
            disembunyikan(".pohon.kecil .papan"),
            ".pohon.kecil tidak menyembunyikan .papan, jadi mode kecil ikut terbebani teks papan"
        );
    }

    #[test]
    fn info_jumlah_ikut_disembunyikan_saat_rail_diperkecil() {
        // Sejak 0.4.4 lima pil jumlah duduk di rel kiri, bukan lagi di header. Pil itu teks:
        // kalau ia tidak ikut disembunyikan, rail 52px menolak menyusut persis seperti papan
        // di penjaga di atas. Tab rel kena aturan yang sama dan ikut diperiksa di sini.
        for pilihan in [".pohon.kecil .sisi-info", ".pohon.kecil .sisi-tab-tombol"] {
            assert!(
                disembunyikan(pilihan),
                "{pilihan} tidak ikut disembunyikan, jadi rail kecil tidak bisa menyusut ke 52px"
            );
        }
    }

    #[test]
    fn mode_cip_menyembunyikan_tanda_progres() {
        // Pada mode cip kartu dirender 54 unit lebarnya sementara tata letaknya masih memakai
        // lebar kartu penuh; tanda persen yang menempel di tepi kanan kartu lama ikut tertinggal
        // di luar kartunya — sudah diukur: teks 10px melayang 116px dari cip-nya.
        assert!(
            disembunyikan("#jejaring.simpl .node-progres"),
            "#jejaring.simpl tidak menyembunyikan .node-progres, jadi angka persen melayang di luar kartu"
        );
    }

    /// Isi satu aturan CSS, dipilih lewat selector yang sama persis. Dipakai penjaga yang
    /// harus membaca deklarasi milik satu kelas, bukan seluruh lembar gaya.
    fn deklarasi_dari(pilihan: &str) -> String {
        buang_komentar(CSS)
            .split('}')
            .find_map(|bagian| {
                let (selektor, isi) = bagian.split_once('{')?;
                selektor
                    .split(',')
                    .any(|s| s.trim() == pilihan)
                    .then(|| isi.trim().to_string())
            })
            .unwrap_or_default()
    }

    #[test]
    fn papan_tidak_menambah_tinggi_saat_daftar_penuh() {
        // Panel yang tingginya ikut isi merampas ruang blok di atasnya: sebelum 0.4.4 tab
        // selesai dengan tiga belas baris menaikkan panel sekitar 1.100px dan daftar struktur
        // hilang dari layar. Sekarang kedua daftar bergantian lewat tab, jadi janji yang sama
        // dijaga mekanisme yang berbeda: panel HANYA boleh setinggi kolomnya (flex 1 + min-height 0
        // di dalam kolom yang dipatok `.app { height: 100dvh }`), dan daftarnya sendiri yang
        // menggulir. Langit-langit angka 168px sengaja tidak dipertahankan: kolom sudah
        // membatasi lebih keras daripada angka tetap, dan angka itu tidak pernah bisa tumbuh
        // bersama tinggi jendela.
        let punya = |teks: &str, properti: &str, nilai: &str| {
            teks.split(';').any(|d| match d.split_once(':') {
                Some((k, v)) => k.trim() == properti && v.trim() == nilai,
                None => false,
            })
        };
        let panel = deklarasi_dari(".papan");
        for (properti, nilai) in [("flex", "1 1 auto"), ("min-height", "0")] {
            assert!(
                punya(&panel, properti, nilai),
                ".papan tidak lagi `{properti}: {nilai}`, jadi tingginya bisa ikut jumlah baris: {panel}"
            );
        }
        let daftar = deklarasi_dari(".papan-isi");
        for (properti, nilai) in [("flex", "1 1 auto"), ("min-height", "0"), ("overflow", "auto")] {
            assert!(
                punya(&daftar, properti, nilai),
                ".papan-isi tidak lagi `{properti}: {nilai}`, jadi daftar panjang mendorong panel keluar kolom: {daftar}"
            );
        }
        // Dua panel rel berbagi kolom yang sama; kalau salah satu tidak min-height 0,
        // `flex-shrink` tidak bisa menembus ukuran isinya dan rail melebar.
        let struktur = deklarasi_dari(".sisi-panel");
        assert!(
            punya(&struktur, "min-height", "0"),
            ".sisi-panel kehilangan `min-height: 0`, jadi panel struktur mendorong papan keluar layar: {struktur}"
        );
    }

    #[test]
    fn teks_baris_tidak_menahan_lebar_rail() {
        // Judul yang panjang tidak boleh ikut dihitung sebagai lebar minimum kolom. Tanpa
        // `width: 0` di samping `flex: 1 1 0`, satu judul 323px memaksa sidebar naik dari
        // 321px ke 390px dan basis 286px tidak pernah tercapai — diukur di Chrome.
        // `min-width: 0` saja tidak cukup, dan mencocokkan teks "width: 0" secara longgar
        // akan selalu lulus karena `min-width: 0` mengandungnya.
        let punya_lebar_nol = |selektor: &str| {
            deklarasi_dari(selektor)
                .split(';')
                .any(|d| match d.split_once(':') {
                    Some((k, v)) => k.trim() == "width" && v.trim() == "0",
                    None => false,
                })
        };
        for selektor in [".papan-nama", ".pohon-nama", ".pohon-grup h3 span", ".sisi-tab-tombol span"] {
            assert!(
                punya_lebar_nol(selektor),
                "{selektor} tidak lagi menyetel `width: 0`, jadi teks nowrap memaksa rail melebar"
            );
        }
    }

    #[test]
    fn kunci_tema_sama_di_cangkang_dan_di_skrip() {
        // Cangkang HTML memasang data-tema sebelum lukisan pertama; skrip membaca kunci
        // yang sama nanti. Kalau salah satu berganti, pilihan user hilang tanpa pesan.
        let skrip = aset("render.js", JS).0;
        let cangkang = std::fs::read_to_string(dir_aset().join("halaman.rs")).expect("halaman.rs terbaca");
        assert!(skrip.contains("var KUNCI_TEMA = 'fv-tema'"), "render.js kehilangan kunci tema");
        assert!(cangkang.contains("var k='fv-tema'"), "cangkang HTML memakai kunci tema lain");
        // CSS tidak boleh lagi memilih skema sendiri: satu sumber kebenaran adalah atribut.
        assert!(
            !CSS.contains("prefers-color-scheme"),
            "view.css masih memilih tema lewat @media, jadi saklar tema bisa berbeda dari yang tampil"
        );
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
