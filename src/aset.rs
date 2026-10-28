//! Aset tampilan (CSS dan JS) dibaca dari folder `public/` milik alat ini, dan kamus bentuk ikon
//! dibaca dari isi `render.js` yang sama. `src/` hanya berisi Rust: aset sengaja dipisah supaya
//! bisa disunting, dipecah, atau dihapus tanpa menyentuh kode program.

use std::fs;
use std::path::{Path, PathBuf};

/// Direktori sumber Rust. Dipakai uji yang membaca berkas `.rs` (hitungan baris, peta fungsi,
/// penjaga anti-hardcode) — bukan untuk aset tampilan. Hanya hidup di build tes: binari produksi
/// tidak pernah membaca sumbernya sendiri.
#[cfg(test)]
pub(crate) fn dir_src() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("src")
}

/// Direktori aset tampilan. Tidak diikat ke bentuk repo pemakai: di repo mana pun alat ini
/// dipakai, `--root` tidak mengubah folder ini. `public/` wajib ikut bila alat disalin ke mesin
/// lain; tidak ada lagi salinan terbaking di dalam binari, jadi folder ini satu-satunya sumber.
pub(crate) fn dir_publik() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("public")
}

/// Nama bentuk ikon yang tersedia di kode tampilan. Dibaca dari isi `render.js` supaya `check`
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

/// Seluruh CSS di `public/`, digabung menurut urutan NAMA berkas. Urutan nama, bukan waktu ubah:
/// hasil `generate` harus sama di dua mesin, dan `check` membandingkan byte. Pemisah per berkas
/// dicetak supaya orang yang membaca `index.html` tahu dari berkas mana satu aturan datang.
pub(crate) fn css() -> Result<String, String> {
    css_dari(&dir_publik())
}

pub(crate) fn css_dari(dir: &Path) -> Result<String, String> {
    let isi_dir = fs::read_dir(dir).map_err(|e| {
        format!(
            "GAGAL membaca folder aset tampilan {}: {e} (folder public/ wajib ikut bersama alat ini)",
            dir.display()
        )
    })?;
    let mut berkas: Vec<PathBuf> = isi_dir
        .filter_map(|b| b.ok())
        .map(|b| b.path())
        .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("css"))
        .collect();
    if berkas.is_empty() {
        return Err(format!(
            "GAGAL: tidak ada satu pun *.css di {} (folder public/ wajib ikut bersama alat ini)",
            dir.display()
        ));
    }
    berkas.sort_by_key(|p| p.file_name().map(|n| n.to_os_string()));
    // Penanda hanya berguna kalau ada lebih dari satu berkas. Dengan satu berkas ia cuma
    // menambah byte tanpa memberi tahu apa pun, jadi tidak dicetak.
    let berpenanda = berkas.len() > 1;
    let mut gabungan = String::new();
    for path in berkas {
        let isi = fs::read_to_string(&path)
            .map_err(|e| format!("GAGAL membaca aset tampilan {}: {e}", path.display()))?;
        if berpenanda {
            let nama = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("tanpa-nama");
            gabungan.push_str(&format!("/* ==== {nama} ==== */\n"));
        }
        gabungan.push_str(&isi);
        if !isi.ends_with('\n') {
            gabungan.push('\n');
        }
    }
    Ok(gabungan)
}

/// Skrip halaman: tepat satu berkas `render.js`. Berkas kedua ditolak, bukan diabaikan: `render.js`
/// adalah satu IIFE, dan dua berkas yang digabung ke satu `<script>` membuat berkas kedua tidak
/// bisa melihat satu pun fungsi di dalam IIFE berkas pertama — kode mati yang tampak hidup.
pub(crate) fn js() -> Result<String, String> {
    js_dari(&dir_publik())
}

pub(crate) fn js_dari(dir: &Path) -> Result<String, String> {
    let isi_dir = fs::read_dir(dir).map_err(|e| {
        format!(
            "GAGAL membaca folder aset tampilan {}: {e} (folder public/ wajib ikut bersama alat ini)",
            dir.display()
        )
    })?;
    let mut lain: Vec<String> = isi_dir
        .filter_map(|b| b.ok())
        .map(|b| b.path())
        .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("js"))
        .filter_map(|p| p.file_name().and_then(|n| n.to_str()).map(str::to_string))
        .filter(|n| n != "render.js")
        .collect();
    if !lain.is_empty() {
        lain.sort();
        return Err(format!(
            "GAGAL: skrip halaman harus satu berkas (public/render.js), tapi ada juga: {}",
            lain.join(", ")
        ));
    }
    let path = dir.join("render.js");
    fs::read_to_string(&path).map_err(|e| {
        format!(
            "GAGAL membaca aset tampilan {}: {e} (folder public/ wajib ikut bersama alat ini)",
            path.display()
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Aset dibaca dari `public/` untuk tiap uji. Dibaca lewat fungsi, bukan `const`, karena
    /// sejak 0.5.0 tidak ada lagi salinan terbaking: aset hanya hidup di disk.
    fn css_teks() -> String {
        css().expect("css terbaca dari public/")
    }

    fn js_teks() -> String {
        js().expect("js terbaca dari public/")
    }
    #[test]
    fn css_digabung_urut_nama_bukan_urutan_folder() {
        // Urutan gabungan ditentukan NAMA berkas, bukan urutan `read_dir`. Dua mesin harus
        // menghasilkan halaman yang sama byte demi byte, dan `check` membandingkan byte.
        let dir = std::env::temp_dir().join(format!("fv-uji-css-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("buat folder uji");
        fs::write(dir.join("20-komponen.css"), "B{}").expect("tulis b");
        fs::write(dir.join("10-token.css"), "A{}").expect("tulis a");
        fs::write(dir.join("catatan.txt"), "bukan css").expect("tulis txt");
        let gabungan = css_dari(&dir).expect("css terbaca");
        let harapan = "/* ==== 10-token.css ==== */\nA{}\n/* ==== 20-komponen.css ==== */\nB{}\n";
        assert_eq!(
            gabungan, harapan,
            "gabungan harus urut nama, berpemisah, dan mengabaikan berkas non-css"
        );
        // Satu berkas: tanpa penanda. Penanda yang tidak memberi tahu apa pun hanya menambah byte
        // dan membuat halaman berubah padahal isinya sama.
        let tunggal = std::env::temp_dir().join(format!("fv-uji-css-satu-{}", std::process::id()));
        let _ = fs::remove_dir_all(&tunggal);
        fs::create_dir_all(&tunggal).expect("buat folder uji");
        fs::write(tunggal.join("10-token.css"), "A{}").expect("tulis a");
        assert_eq!(
            css_dari(&tunggal).expect("css tunggal terbaca"),
            "A{}\n",
            "satu berkas tidak boleh diberi penanda"
        );
        let _ = fs::remove_dir_all(&tunggal);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn berkas_css_publik_berprefix_angka() {
        // Cascade bergantung urutan, dan urutan gabungan adalah urutan NAMA berkas. Prefix angka
        // membuat urutan itu terbaca dari nama, bukan tersembunyi di kepala orang. Penjaga ini
        // memeriksa folder publik yang sungguh dipakai, bukan folder sementara.
        let dir = dir_publik();
        let mut nama: Vec<String> = fs::read_dir(&dir)
            .expect("public/ terbaca")
            .filter_map(|b| b.ok())
            .map(|b| b.file_name().to_string_lossy().to_string())
            .filter(|n| n.ends_with(".css"))
            .collect();
        nama.sort();
        assert!(!nama.is_empty(), "public/ harus punya minimal satu berkas CSS");
        for n in &nama {
            let huruf: Vec<char> = n.chars().collect();
            let sah = huruf.len() > 4
                && huruf[0].is_ascii_digit()
                && huruf[1].is_ascii_digit()
                && huruf[2] == '-'
                && huruf.iter().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || *c == '-' || *c == '.');
            assert!(
                sah,
                "berkas CSS publik harus berprefix dua angka dan huruf kecil ({n}); urutan gabungan dibaca dari nama"
            );
        }
    }

    #[test]
    fn js_hanya_satu_berkas() {
        // render.js satu IIFE: berkas kedua yang digabung tidak bisa melihat fungsinya, jadi
        // keberadaannya ditolak — lebih jujur daripada membiarkan kode mati tampak hidup.
        let dir = std::env::temp_dir().join(format!("fv-uji-js-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("buat folder uji");
        fs::write(dir.join("render.js"), "(function(){})();").expect("tulis render");
        assert!(js_dari(&dir).is_ok(), "satu berkas harus diterima");
        fs::write(dir.join("tambahan.js"), "var x = 1;").expect("tulis tambahan");
        let galat = js_dari(&dir).expect_err("berkas kedua harus ditolak");
        assert!(galat.contains("tambahan.js"), "galat harus menyebut berkasnya: {galat}");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn aset_hilang_menyebut_path() {
        // Tanpa salinan terbaking, folder aset yang hilang harus berhenti dengan pesan yang
        // menyebut path — bukan halaman tanpa gaya dan bukan panic.
        let dir = std::env::temp_dir().join(format!("fv-uji-kosong-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("buat folder uji");
        let galat_css = css_dari(&dir).expect_err("css kosong harus galat");
        assert!(galat_css.contains("*.css"), "galat css harus menyebut pola berkas: {galat_css}");
        assert!(
            galat_css.contains(&dir.display().to_string()),
            "galat css harus menyebut folder yang dibaca: {galat_css}"
        );
        let galat_js = js_dari(&dir).expect_err("js hilang harus galat");
        assert!(galat_js.contains("render.js"), "galat js harus menyebut berkasnya: {galat_js}");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn dir_publik_menunjuk_aset_tampilan_sendiri_dan_src_hanya_rust() {
        // Aset dibaca dari folder crate, bukan dari bentuk repo pemakai: di repo mana pun alat ini
        // dipakai, `--root` tidak boleh mengubahnya. Dan `src/` tidak memuat aset tampilan lagi.
        let css = css().expect("css terbaca dari public/");
        let js = js().expect("js terbaca dari public/");
        assert!(!css.is_empty(), "css terbaca kosong");
        assert!(!js.is_empty(), "js terbaca kosong");
        assert!(js.contains("var IKON = {"), "js yang terbaca bukan render.js");
        let sisa: Vec<String> = fs::read_dir(dir_src())
            .expect("src terbaca")
            .filter_map(|b| b.ok())
            .map(|b| b.file_name().to_string_lossy().to_string())
            .filter(|n| n.ends_with(".css") || n.ends_with(".js"))
            .collect();
        assert!(sisa.is_empty(), "src/ masih memuat aset tampilan: {sisa:?}");
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
        let nilai = deklarasi(&css_teks(), "font-size");
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
            assert!(css_teks().contains(&format!("--fs-{i}:")), "token --fs-{i} dipakai tapi tidak didefinisikan");
        }
    }

    #[test]
    fn radius_dan_jarak_view_css_hanya_token() {
        for v in deklarasi(&css_teks(), "border-radius") {
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
            for v in deklarasi(&css_teks(), properti) {
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
        let dipakai: Vec<(String, bool)> = css_teks()
            .match_indices("var(--")
            .map(|(i, _)| {
                let sisa = &css_teks()[i + 6..];
                let akhir = sisa
                    .find(|c: char| c == ')' || c == ',' || c.is_whitespace())
                    .unwrap_or(0);
                let bercadangan = sisa.as_bytes().get(akhir) == Some(&b',');
                (sisa[..akhir].to_string(), bercadangan)
            })
            .collect();
        assert!(dipakai.len() > 100, "parser kehilangan sebagian pemakaian variabel: {}", dipakai.len());
        let didefinisikan: Vec<String> = css_teks()
            .match_indices("\n  --")
            .map(|(i, _)| {
                css_teks()[i + 2..]
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
        css_teks().split('}')
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
        buang_komentar(&css_teks())
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
        let skrip = js_teks();
        let cangkang = std::fs::read_to_string(dir_src().join("halaman.rs")).expect("halaman.rs terbaca");
        assert!(skrip.contains("var KUNCI_TEMA = 'fv-tema'"), "render.js kehilangan kunci tema");
        assert!(cangkang.contains("var k='fv-tema'"), "cangkang HTML memakai kunci tema lain");
        // CSS tidak boleh lagi memilih skema sendiri: satu sumber kebenaran adalah atribut.
        assert!(
            !css_teks().contains("prefers-color-scheme"),
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
        let nyata = bentuk_terdaftar(&js_teks());
        assert!(
            nyata.len() >= 40,
            "kamus bentuk ikon harus terbaca dari render.js, dapat {}",
            nyata.len()
        );
    }

}
