//! Bahan mulai pakai alat ini di repo lain: kerangka manifest untuk `init`, dan dataset contoh
//! (`contoh/`) yang membuktikan alat benar-benar berjalan pada data asing tanpa repo induk.
//!
//! `init` sengaja TIDAK mengisi data: alat ini tidak membaca kode sumber dan tidak menemukan fitur,
//! jadi baris fitur selalu hasil kurasi orang yang membacanya dulu (lihat README bagian "Alat ini bukan
//! pencari fitur"). Yang dicetak di sini cuma kontrak kolom supaya bentuknya benar sejak awal.
//!
//! Dataset contoh memakai kosakata yang tidak ada di repo mana pun (`jalan`, `reda`, `kirim`).
//! Kalau satu nama saja masih tertinggal di kode Rust atau `render.js`, tes di modul ini yang menyalakan.

use std::fmt::Write as _;
use std::fs;
use std::path::Path;

use crate::manifest::*;

pub(crate) const TEMPLATE_CONFIG: &str = "\
# config.tsv — kunci|nilai. Ini DATA, bukan konfigurasi alat: ganti dengan kosakata repo Anda.
# Alat tidak mengenal satu pun nama repo dari kodenya; semua label halaman berasal dari berkas ini.
merek|Repo Saya
judul|Peta Fitur Repo Saya
bahasa|id
status|aktif,sebagian,dimatikan,direncanakan
status_peran_utama|aktif
status_peran_separuh|sebagian
status_peran_mati|dimatikan
jenis_relasi|alur,panggil,kelola,kontrol,hasil,data
aktor_ikon|Pengguna:orang,Petugas:penggeser,Sistem:pancar
minimum_fitur_per_service|2
";

pub(crate) const TEMPLATE_SERVICES: &str = "\
# services.tsv — key|nama|stack|deskripsi|folder|dokumen|ikon
# `folder` dan `dokumen` wajib path nyata relatif root. `ikon` harus salah satu nama bentuk di
# render.js; salah ketik akan ditolak `check`.
# contoh: svc-satu|Service Satu|Go|Layanan inti repo ini.|svc-satu|svc-satu/README.md|gerbang
";

pub(crate) const TEMPLATE_APPS: &str = "\
# applications.tsv — id|service|nama|jenis|folder|deskripsi|dokumen|ikon
# `jenis` bebas (mis. aplikasi, binary, web); satu aplikasi boleh melayani banyak produk.
# contoh: api-satu|svc-satu|API Satu|binary|svc-satu|Menyajiskan endpoint inti.|svc-satu/docs/api.md|lengkung
";

pub(crate) const TEMPLATE_PRODUK: &str = "\
# produk.tsv — id|nama|aktor|hasil|ringkasan|dokumen|catatan
# Produk = aktor + hasil, BUKAN service atau aplikasi. Status produk dihitung dari sub-fiturnya.
# contoh: masuk-sesi|Masuk sesi|Pengguna|Sesi aktif|Pengguna masuk lalu sesi berjalan.|docs/produk.md|
";

pub(crate) const TEMPLATE_FEATURES: &str = "\
# features.tsv — id|service|fitur|status|ringkasan|cara_kerja|titik_masuk|bukti|dokumen|catatan
# Satu-satunya berkas yang memegang status. Dua pemisah yang BERBEDA dipakai di baris yang sama:
# kolom daftar (`bukti`, `dokumen`) dipisah koma, sedangkan titik masuk/cara kerja memakai titik koma
# karena `audit` memecahnya menjadi klaim yang dicocokkan ke isi berkas. Salah pakai = path dianggap
# satu berkas panjang lalu `check` menolak.
# contoh: svc-satu-hello|svc-satu|Sapa pengguna|aktif|Menyapa satu pengguna.|Handler GET /hello.|GET /hello; HelloController|svc-satu/src/hello.go,svc-satu/src/route.go|svc-satu/docs/hello.md|
";

pub(crate) const TEMPLATE_PEMETAAN: &str = "\
# pemetaan.tsv — subfitur|produk|aplikasi
# Tiap sub-fitur wajib muncul tepat satu kali di sini, dan aplikasinya harus satu service.
# contoh: svc-satu-hello|masuk-sesi|api-satu
";

pub(crate) const TEMPLATE_EDGES: &str = "\
# edges.tsv — dari|ke|jenis|label
# `jenis` harus anggota daftar `jenis_relasi` di config.tsv. Setiap sub-fitur butuh minimal satu edge.
# contoh: svc-satu-hello|svc-lain|panggil|Minta data pengguna
";

/// Berkas kedelapan dan satu-satunya yang OPSIONAL: repo yang belum memakai papan pekerjaan
/// tidak punya berkas ini, dan hasil `check`-nya tidak berubah karena ketiadaannya.
pub(crate) const TEMPLATE_TUGAS: &str = "\
# tugas.tsv — id|subfitur|judul|status|progres|pemegang|berkas|catatan
# Hanya `catatan` yang boleh kosong. `subfitur` wajib anggota features.tsv dan `berkas` wajib
# task nyata di repo ini: papan ini tidak menerima pekerjaan tanpa berkas.
# `status` wajib anggota `status_tugas` di config.tsv; `progres` bilangan 0..=100, dan status
# terakhir kosakata menuntut 100. Kalau berkas task menulis `Status:`, kata pertama sesudahnya
# harus sama dengan status yang diumumkan di sini — itu yang menjaga angka tidak bisa dikarang.
# contoh: svc-satu-hello-t1|svc-satu-hello|Siapkan rute sapa|selesai|100|agen|plan/hello.md|
";

/// (path relatif root, isi template) untuk tiap manifest.
pub(crate) fn template() -> Vec<(&'static str, &'static str)> {
    vec![
        (CONFIG_FILE, TEMPLATE_CONFIG),
        (SERVICES_FILE, TEMPLATE_SERVICES),
        (PRODUCTS_FILE, TEMPLATE_PRODUK),
        (APPS_FILE, TEMPLATE_APPS),
        (MAPPING_FILE, TEMPLATE_PEMETAAN),
        (FEATURES_FILE, TEMPLATE_FEATURES),
        (EDGES_FILE, TEMPLATE_EDGES),
        (TASKS_FILE, TEMPLATE_TUGAS),
    ]
}

/// Menulis kerangka manifest. Berkas yang sudah ada TIDAK pernah ditimpa — manifest adalah data
/// hasil kurasi, dan menimpanya berarti menghapus pekerjaan orang.
pub(crate) fn init(root: &Path) -> Result<String, String> {
    let folder = root.join("docs").join("view");
    fs::create_dir_all(&folder)
        .map_err(|e| format!("GAGAL membuat {}: {e}", folder.display()))?;

    let mut baru: Vec<String> = Vec::new();
    let mut ada: Vec<String> = Vec::new();
    for (relatif, isi) in template() {
        let tujuan = root.join(relatif);
        if tujuan.exists() {
            ada.push(relatif.to_string());
            continue;
        }
        fs::write(&tujuan, isi).map_err(|e| format!("GAGAL menulis {}: {e}", tujuan.display()))?;
        baru.push(relatif.to_string());
    }

    if baru.is_empty() {
        return Ok(format!(
            "LEWATI: {} sudah berisi manifest, tidak ada yang ditulis ({} berkas dipertahankan).",
            folder.display(),
            ada.len()
        ));
    }

    let mut pesan = String::new();
    let _ = writeln!(
        pesan,
        "DIBUAT {} kerangka manifest di {}:",
        baru.len(),
        folder.display()
    );
    for nama in &baru {
        let _ = writeln!(pesan, "  - {nama}");
    }
    if !ada.is_empty() {
        let _ = writeln!(pesan, "DIPERTAHANKAN (sudah ada): {}", ada.join(", "));
    }
    let _ = writeln!(
        pesan,
        "LANGKAH BERIKUTNYA: isi baris fitur dari kode yang benar-benar dibaca, lalu jalankan `check`. Alat ini tidak menemukan fitur sendiri."
    );
    Ok(pesan)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::aset::*;
    use crate::aturan::*;
    use crate::audit::*;
    use crate::halaman::*;
    use crate::uji::*;

    fn akar_uji() -> std::path::PathBuf {
        std::env::temp_dir().join(format!("fv-init-{}", std::process::id()))
    }

    #[test]
    fn init_menulis_delapan_manifest_dan_tidak_menimpa_hasil_kerja_orang() {
        let akar = akar_uji();
        let _ = fs::remove_dir_all(&akar);
        fs::create_dir_all(&akar).expect("folder uji");

        // Manifest yang sudah diisi orang tidak boleh tertimpa oleh init.
        let sudah_diisi = akar.join(FEATURES_FILE);
        fs::create_dir_all(sudah_diisi.parent().expect("punya induk")).expect("buat folder manifest");
        fs::write(&sudah_diisi, "# saya sudah mengisinya\n").expect("tulis uji");

        let pesan = init(&akar).expect("init jalan");
        assert_eq!(
            template().len(),
            8,
            "kontrak template = delapan manifest (satu di antaranya opsional dipakai)"
        );
        for (relatif, _) in template() {
            let path = akar.join(relatif);
            assert!(path.is_file(), "{relatif} harus tercetak");
            let isi = fs::read_to_string(&path).expect("terbaca");
            assert!(
                isi.starts_with('#'),
                "{relatif} harus dibuka dengan komentar kontrak kolom"
            );
        }
        assert_eq!(
            fs::read_to_string(&sudah_diisi).expect("baca lagi"),
            "# saya sudah mengisinya\n",
            "isi orang tidak boleh hilang"
        );
        assert!(pesan.contains("DIPERTAHANKAN"), "laporan harus menyebut yang dilewati: {pesan}");

        // Jalan kedua = tidak ada yang ditulis sama sekali.
        let pesan_lagi = init(&akar).expect("init kedua jalan");
        assert!(pesan_lagi.starts_with("LEWATI"), "pesan kedua: {pesan_lagi}");
        assert_eq!(fs::read_to_string(akar.join(CONFIG_FILE)).expect("config"), TEMPLATE_CONFIG);

        let _ = fs::remove_dir_all(&akar);
    }

    #[test]
    fn template_config_memakai_kunci_wajib_yang_dikenali_parser() {
        let config = parse_config(TEMPLATE_CONFIG, "template-config").expect("template config valid");
        assert!(config.minimum_fitur >= 1, "ambang minimal harus ada");
    }

    #[test]
    fn template_config_memakai_empat_status_dan_enam_jenis_relasi() {
        let config = parse_config(TEMPLATE_CONFIG, "template-config").expect("template config valid");
        assert_eq!(config.statuses.len(), 4, "empat status bawaan");
        assert_eq!(config.kinds.len(), 6, "enam jenis relasi bawaan");
        assert_eq!(
            config.aktor_ikon.len(),
            3,
            "tiga aktor bawaan: {:?}",
            config.aktor_ikon
        );
    }

    // ---- Dataset contoh: bukti bahwa alat berjalan di repo asing, tanpa repo induk. ----

    #[test]
    fn penjaga_merek_membuang_isi_kerangka_bukan_kodenya() {
        // Sapuan penjaga hanya kode produksi; tes ini memakai potongan yang sama dengannya.
        let sumber = fs::read_to_string(dir_aset().join("template.rs")).expect("sumber template.rs terbaca");
        let produksi = produksi_dari(&sumber);
        assert!(
            produksi.contains("Repo Saya"),
            "kalau placeholder ini hilang, tes tidak membuktikan apa-apa"
        );
        let bersih = tanpa_kerangka(produksi);
        assert!(!bersih.contains("Repo Saya"), "isi kerangka ikut tersapu");
        assert!(
            bersih.contains("pub(crate) fn init"),
            "potongannya kelewat lebar: fungsi `init` ikut terbuang"
        );
        // Negative control: nama yang ditanam di kode (bukan di kerangka) tetap tertangkap.
        assert!(
            tanpa_kerangka(&format!("{produksi}\nconst MERK: &str = \"Repo Saya\";",)).contains("Repo Saya"),
            "literal di luar kerangka tidak boleh lolos"
        );
    }

    #[test]
    fn contoh_lulus_validasi_dan_terbangun_jadi_halaman() {
        let root = root_contoh();
        assert!(
            root.join(CONFIG_FILE).is_file(),
            "dataset contoh wajib ikut di repo alat: {} hilang",
            root.join(CONFIG_FILE).display()
        );
        let model = muat_model(&root).expect("manifest contoh harus valid");
        let galat = validate(&model, &root);
        assert!(galat.is_empty(), "contoh harus lolos validasi: {galat:?}");
        let html = bangun_html(&model, "cap-uji");
        assert!(html.contains(&model.config.merek), "merek contoh harus muncul di halaman");
        assert!(html.contains("cap-uji"), "cap yang diberikan harus tertanam di halaman");
    }

    #[test]
    fn halaman_contoh_di_repo_computed_dari_render_js_yang_sekarang() {
        // `contoh/` adalah satu-satunya bukti bahwa alat ini jalan di repo asing, dan README
        // menjanjikan dataset ini lulus `check`. Selama tes cuma membangun HTML di memori,
        // halaman yang TERCOMMIT bisa tertinggal satu putaran perubahan render.js: clone baru
        // langsung gagal `check --root contoh` tanpa ada tes yang memberi tahu. Bandingkan
        // byte demi byte dengan hasil susun hari ini.
        let root = root_contoh();
        let model = muat_model(&root).expect("manifest contoh harus valid");
        let cap = cap_manifest(&root).expect("cap manifest contoh terbaca");
        let sekarang = bangun_html(&model, &cap);
        let di_repo = fs::read_to_string(root.join(OUTPUT_FILE))
            .unwrap_or_else(|e| panic!("halaman contoh harus tercommit: {}: {e}", root.join(OUTPUT_FILE).display()));
        // Pembandangnya kesamaan string, sama seperti yang `check` pakai; `beda_baris_pertama`
        // cuma dipakai untuk menunjuk barisnya di pesan galat (ia tidak pernah mengembalikan 0).
        if di_repo != sekarang {
            panic!(
                "{} basi: halaman itu hasil susun render.js/view.css PUTARAN LAMA (baris {} berbeda; \
                 {} byte di disk vs {} byte hasil susun hari ini). Segarkan sebelum lanjut:\n  cargo run -- generate --root contoh",
                root.join(OUTPUT_FILE).display(),
                beda_baris_pertama(&di_repo, &sekarang),
                di_repo.len(),
                sekarang.len()
            );
        }
    }

    #[test]
    fn contoh_lulus_audit_tanpa_merah() {
        let root = root_contoh();
        let model = muat_model(&root).expect("manifest contoh harus valid");
        assert_eq!(
            audit(&model, &root).merah,
            0,
            "dataset contoh adalah rujukan orang lain; ia tidak boleh membuat `audit` exit 1"
        );
    }

    #[test]
    fn contoh_memakai_kosakata_yang_berbeda_dari_repo_asli() {
        let config = parse_config(
            &fs::read_to_string(root_contoh().join(CONFIG_FILE)).expect("config contoh terbaca"),
            CONFIG_FILE,
        )
        .expect("config contoh valid");
        assert!(
            !config.statuses.contains(&"aktif".to_string()),
            "contoh harus memakai status asing, bukan kosakata repo asli: {:?}",
            config.statuses
        );
        assert!(
            !config.kinds.contains(&"alur".to_string()),
            "contoh harus memakai jenis relasi asing: {:?}",
            config.kinds
        );
        // Empat peran status harus terwakili, supaya contoh melatih seluruh jalan di `audit`.
        let model = muat_model(&root_contoh()).expect("manifest contoh harus valid");
        for status in &config.statuses {
            assert!(
                model.features.iter().any(|f| &f.status == status),
                "status {status} tidak terwakili di contoh"
            );
        }
    }

    #[test]
    fn nama_contoh_tidak_muncul_di_render_js() {
        // Aturan yang sama untuk nama repo asli, sekarang untuk dataset contoh: kalau kunci bentuk
        // atau teks di render.js meniru nama contoh, alat ini masih "tahu" datanya.
        let model = muat_model(&root_contoh()).expect("manifest contoh harus valid");
        let sumber_lower = fs::read_to_string(dir_aset().join("render.js"))
            .expect("sumber render.js terbaca")
            .to_lowercase();
        let mut nama: Vec<String> = Vec::new();
        for s in &model.services {
            nama.push(s.key.clone());
            nama.push(s.stack.clone());
        }
        for a in &model.apps {
            nama.push(a.id.clone());
        }
        for p in &model.products {
            nama.push(p.id.clone());
            nama.push(p.actor.clone());
        }
        // Sengaja BUKAN status/jenis relasi: kata seperti `kirim` atau `jalan` adalah kata biasa yang
        // juga dipakai kode tampilan sebagai identifier-nya sendiri (`kirimAksi`, `jalanGenerate`).
        // Yang dijaga di sini nama entitas, bukan kosakata — persis aturan pada data repo asli.
        let menyangkut: Vec<&String> = nama
            .iter()
            .filter(|n| n.len() > 3 && sumber_lower.contains(&n.to_lowercase()))
            .collect();
        assert!(
            menyangkut.is_empty(),
            "kode tampilan mengenal nama dataset contoh: {menyangkut:?}"
        );
        let banding: Vec<String> = nama.iter().map(|n| n.to_lowercase()).collect();
        let tabrakan: Vec<String> = bentuk_terdaftar(JS)
            .into_iter()
            .filter(|b| banding.contains(&b.to_lowercase()))
            .collect();
        assert!(
            tabrakan.is_empty(),
            "kamus bentuk memakai kunci yang sama dengan nama contoh: {tabrakan:?}"
        );
    }
}
