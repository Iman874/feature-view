//! Titik masuk: argumen, pemilihan perintah, dan bantuan. Tidak ada aturan data atau render di sini.

mod data;
mod manifest;
mod aset;
mod aturan;
mod halaman;
mod audit;
mod saji;
mod setelan;
mod template;

#[cfg(test)]
mod uji;

use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;
use crate::data::*;
use crate::manifest::*;
use crate::aturan::*;
use crate::halaman::*;
use crate::audit::*;
use crate::saji::*;
use crate::setelan::*;
use crate::template::*;

fn main() -> ExitCode {
    match jalankan() {
        Ok(kode) => kode,
        Err(pesan) => {
            eprintln!("{pesan}");
            ExitCode::from(2)
        }
    }
}

pub(crate) fn jalankan() -> Result<ExitCode, String> {
    let args: Vec<String> = env::args().skip(1).collect();
    let mut perintah = "generate".to_string();
    let mut port = port_env(PORT_BAWAAN);
    let mut root = env::current_dir().map_err(|e| format!("GAGAL membaca direktori kerja: {e}"))?;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "generate" | "check" | "audit" | "serve" | "stop" | "init" => {
                perintah = args[i].clone();
                i += 1;
            }
            "--root" => {
                let nilai = args
                    .get(i + 1)
                    .ok_or_else(|| "PEMAKAIAN: --root memerlukan path".to_string())?;
                root = PathBuf::from(nilai);
                i += 2;
            }
            "--port" => {
                let nilai = args
                    .get(i + 1)
                    .ok_or_else(|| "PEMAKAIAN: --port memerlukan angka 1-65535".to_string())?;
                port = nilai
                    .parse::<u16>()
                    .map_err(|_| format!("GAGAL: --port bukan angka 1-65535: {nilai}"))?;
                if port == 0 {
                    return Err("GAGAL: --port tidak boleh 0".to_string());
                }
                i += 2;
            }
            "-h" | "--help" => {
                cetak_bantuan();
                return Ok(ExitCode::SUCCESS);
            }
            "-V" | "--version" => {
                println!(
                    "feature-view {} (manifest peta fitur)",
                    env!("CARGO_PKG_VERSION")
                );
                return Ok(ExitCode::SUCCESS);
            }
            lain => {
                return Err(format!(
                    "GAGAL: argumen tidak dikenal: {lain}\nPEMAKAIAN: feature-view [generate|check|audit|init|serve|stop] [--root <path>] [--port <angka>]"
                ));
            }
        }
    }

    if perintah == "stop" {
        return hentikan(port);
    }

    // init tidak memuat manifest: justru untuk repo yang belum punya docs/view/.
    if perintah == "init" {
        let pesan = init(&root)?;
        println!("{pesan}");
        return Ok(ExitCode::SUCCESS);
    }

    // serve membaca manifest pada setiap permintaan, jadi proses ini sengaja tidak
    // memvalidasi lebih dulu: manifest yang sedang rusak harus tetap bisa dilihat
    // lewat browser beserta daftar galatnya, bukan membuat server menolak jalan.
    if perintah == "serve" {
        return sajikan(&root, port);
    }

    let model = match muat_model(&root) {
        Ok(model) => model,
        Err(LoadError::Manifest(galat)) => {
            eprintln!("GAGAL validasi manifest:");
            for g in galat {
                eprintln!("- {g}");
            }
            return Ok(ExitCode::from(1));
        }
        Err(LoadError::Io(pesan)) => return Err(pesan),
    };

    let keluaran = root.join(OUTPUT_FILE);

    match perintah.as_str() {
        "generate" => {
            let cap = cap_manifest(&root)?;
            let ringkas = tulis_index(&root, &model, &cap)?;
            println!("OK: {ringkas}");
            Ok(ExitCode::SUCCESS)
        }
        "audit" => {
            let hasil = audit(&model, &root);
            let aktif = model.config.status_peran("utama");
            if hasil.merah > 0 {
                println!();
                println!("GAGAL: {} baris `{aktif}` punya klaim yang tidak terbukti dari berkas yang ditunjuk service-nya.", hasil.merah);
                return Ok(ExitCode::from(1));
            }
            // Exit 0 berarti "tidak ada yang merah", bukan "semua terbukti": daftar kuning dan
            // tidak-teruji tetap ada di atas, jadi kalimat penutup wajib menyebut sisanya.
            println!("OK: tidak ada baris `{aktif}` yang klaimnya gagal terbukti.");
            println!(
                "     Bukan berarti semua fitur bekerja: {} baris masih kuning dan {} baris tidak teruji.",
                hasil.kuning, hasil.tak_teruji
            );
            Ok(ExitCode::SUCCESS)
        }
        "check" => {
            let cap = cap_manifest(&root)?;
            let html = bangun_html(&model, &cap);
            let lama = fs::read_to_string(&keluaran).map_err(|e| {
                format!(
                    "GAGAL membaca {}: {e}\nJalankan generate terlebih dahulu.",
                    keluaran.display()
                )
            })?;
            if lama == html {
                println!(
                    "OK: {} sinkron ({} produk, {} aplikasi, {} sub-fitur, {} service, {} relasi)",
                    keluaran.display(),
                    model.products.len(),
                    model.apps.len(),
                    model.features.len(),
                    model.services.len(),
                    model.edges.len()
                );
                Ok(ExitCode::SUCCESS)
            } else {
                let baris = beda_baris_pertama(&lama, &html);
                eprintln!(
                    "GAGAL: {} tidak sinkron dengan manifest (beda pertama di baris {}).",
                    keluaran.display(),
                    baris
                );
                eprintln!(
                    "Jalankan: {}",
                    petunjuk("generate", &root)
                );
                Ok(ExitCode::from(1))
            }
        }
        _ => Err("GAGAL: perintah tidak dikenal".to_string()),
    }
}

/// Cara menjalankan satu perintah, dengan root yang sungguh dipakai proses ini.
/// Bentuk lamanya (`cargo run -- generate` + keterangan "dari folder alat ini") adalah perintah yang
/// gagal di repo induk yang menyimpan alat di subfolder: dia mencari manifest di dalam folder alat
/// sendiri lalu mati. Yang tidak boleh ditambahkan
/// tetap lokasi alat di repo pemakai (alat ini boleh jadi repo mandiri atau di-vendor): root di sini
/// adalah data waktu jalan, bukan path yang di-baking ke sumber atau ke HTML.
/// Yang menjaga aturan ini: `kode_alat_tidak_menyebut_letaknya_di_repo_pemakai`.
pub(crate) fn petunjuk(perintah: &str, root: &std::path::Path) -> String {
    format!("cargo run -- {perintah} --root {}", root.display())
}

pub(crate) fn cetak_bantuan() {
    println!(
        "feature-view {} — peta produk dan implementasi dari manifest TSV, tanpa dependency",
        env!("CARGO_PKG_VERSION")
    );
    println!();
    println!("PEMAKAIAN:");
    println!("  feature-view [PERINTAH] [--root <path>] [--port <angka>]");
    println!("  PERINTAH kosong = generate.");
    println!();
    println!("PERINTAH:");
    println!("  init       cetak kerangka delapan manifest docs/view/ (tugas.tsv opsional; berkas yang ada tidak ditimpa)");
    println!("  generate   validasi manifest (termasuk config.tsv) lalu tulis docs/view/index.html");
    println!("  check      validasi dan pastikan index.html masih sinkron dengan manifest");
    println!("  audit      cocokkan isi kode terhadap token di titik_masuk + ringkasan + cara_kerja");
    println!("  serve      sajikan halaman dan baca manifest ulang pada tiap permintaan");
    println!("  stop       matikan serve di port ini; serve di port lain dilaporkan, tidak dimatikan (khusus Windows)");
    println!();
    println!("OPSI:");
    println!("  --root <path>    root repo yang dibaca (default: direktori kerja)");
    println!("  --port <angka>   port serve/stop, 1-65535 (default: {PORT_BAWAAN}, env PETA_PORT)");
    println!("  -h, --help       bantuan ini");
    println!("  -V, --version    versi alat");
    println!();
    println!("KELUARAN:");
    println!("  0  sukses (atau tidak ada klaim baris utama yang hampa/sempit pada audit)");
    println!("  1  validasi manifest menolak isi, atau audit menemukan klaim baris utama yang tidak terbukti");
    println!("  2  manifest tidak terbaca, argumen salah, atau kegagalan I/O");
    println!();
    println!("ENV (opsional, nilai tidak sah atau 0 jatuh kembali ke default):");
    println!("  PETA_PORT                        port bawaan serve/stop");
    println!("  PETA_BATAS_BACA_BUKTI            ukuran maksimum satu berkas bukti, byte");
    println!("  PETA_INDEKS_BATAS_BERKAS         batas berkas saat menyapu folder service");
    println!("  PETA_INDEKS_BATAS_BYTE           batas total byte saat menyapu folder service");
    println!();
    println!("Yang dijamin dan yang tidak:");
    println!("  check membuktikan struktur, cakupan pemetaan, dan path bukti/dokumen ada;");
    println!("  audit membuktikan isi kode masih menyebut apa yang baris itu klaim.");
    println!("  Keduanya TIDAK membuktikan fitur bekerja — alat ini tidak menjalankan kode, tidak membuka");
    println!("  database, dan tidak memanggil endpoint. Merek, daftar status, dan jenis relasi dibaca dari");
    println!("  config.tsv, bukan dari kode ini. Alat ini bukan pencari fitur: baris manifest adalah hasil");
    println!("  kurasi orang yang membaca kode.");
    println!();
    println!("  Beda generate dan serve: generate membaking data ke satu file yang bisa dibuka dari disk,");
    println!("  serve tidak menulis apa pun dan selalu memakai isi TSV terbaru.");
    println!("  Lupa menutup serve? jalankan: feature-view stop   (tanpa menutup terminalnya).");
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::aset::*;
    use crate::uji::*;
    #[test]
    fn sumber_alat_tidak_memuat_merek_dari_config() {
        use std::collections::HashMap;
        let root = root_uji();
        // Kosakata yang dijaga diambil dari manifest, bukan dari tulisan di tes ini: kalau penjaganya
        // sendiri mengutip `"aktif"`, alat ini melanggar aturannya lewat penjaganya.
        let mut merek: Vec<String> = Vec::new();
        let mut kosakata: Vec<String> = Vec::new();
        for asal in [&root, &root_contoh()] {
            let isi_config = match fs::read_to_string(asal.join(CONFIG_FILE)) {
                Ok(isi) => isi,
                Err(_) => continue,
            };
            let config = parse_config(&isi_config, CONFIG_FILE).expect("config valid");
            merek.push(config.merek.clone());
            kosakata.extend(config.statuses.clone());
            kosakata.extend(config.kinds.clone());
        }
        assert!(
            !merek.is_empty() && !kosakata.is_empty(),
            "penjaga tidak punya satu pun nama untuk diperiksa: {merek:?} / {kosakata:?}"
        );
        let berkas = sumber_alat();
        let mut pemilik: HashMap<String, String> = HashMap::new();
        let mut ganda: Vec<String> = Vec::new();
        let mut item = 0usize;
        for (nama, isi) in &berkas {
            // Yang dijaga hanya kode produksi; modul tes boleh memakai fixture berbahasa Indonesia.
            let produksi = produksi_dari(isi);
            let kode = tanpa_kerangka(produksi);
            for merk in &merek {
                assert!(
                    !kode.contains(merk),
                    "merek `{merk}` adalah DATA; ketemu sebagai literal di {nama}"
                );
            }
            for kata in &kosakata {
                let literal = format!("\"{kata}\"");
                assert!(
                    !kode.contains(&literal),
                    "kosakata {literal} dari config.tsv harus datang dari data, bukan literal di {nama}"
                );
            }
            // Struktur: satu item top-level hanya boleh hidup di satu berkas. Kalau sebuah `fn` disalin ke
            // dua modul, keduanya bisa hidup berdampingan dan salahnya tidak terbaca di mana pun.
            for baris in produksi.lines() {
                let kata: Vec<&str> = baris.split_whitespace().collect();
                if kata.is_empty() {
                    continue;
                }
                let i = usize::from(kata[0] == "pub" || kata[0].starts_with("pub("));
                if kata.len() < i + 2 || !matches!(kata[i], "fn" | "const" | "struct" | "enum" | "trait" | "type")
                {
                    continue;
                }
                if baris.starts_with(' ') {
                    continue;
                }
                let ident = kata[i + 1].trim_end_matches(|c: char| !c.is_alphanumeric() && c != '_');
                if ident.is_empty() {
                    continue;
                }
                item += 1;
                if let Some(sebelum) = pemilik.get(ident) {
                    if sebelum != nama {
                        ganda.push(format!("{ident} hidup di {sebelum} dan {nama}"));
                    }
                } else {
                    pemilik.insert(ident.to_string(), nama.clone());
                }
            }
        }
        assert!(
            item >= 80,
            "item top-level alat harus terbaca dari sumber, hanya {item} — penjaga ini tidak boleh diam-diam jadi no-op"
        );
        assert!(ganda.is_empty(), "satu item hidup di dua modul: {ganda:?}");
    }

    #[test]
    fn render_js_tidak_mengenal_nama_data_repo() {
        let root = root_uji();
        let model = muat_model(&root).expect("manifest root uji harus valid");
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
        for (aktor, _) in &model.config.aktor_ikon {
            nama.push(aktor.clone());
        }
        let menyangkut: Vec<&String> = nama
            .iter()
            .filter(|n| n.len() > 3 && sumber_lower.contains(&n.to_lowercase()))
            .collect();
        assert!(
            menyangkut.is_empty(),
            "kode tampilan tidak boleh mengenal nama data repo ini: {menyangkut:?}"
        );
        // Kunci kamus bentuk juga nama: kalau sebuah kunci sama dengan nama data, kode tampilan
        // masih mengenal isi repo ini walaupun tidak pernah dibaca sebagai teks utuh.
        let banding: Vec<String> = nama.iter().map(|n| n.to_lowercase()).collect();
        let tabrakan: Vec<String> = bentuk_terdaftar(JS)
            .into_iter()
            .filter(|b| banding.contains(&b.to_lowercase()))
            .collect();
        assert!(
            tabrakan.is_empty(),
            "kamus bentuk memakai kunci yang sama dengan nama data: {tabrakan:?}"
        );
    }

    #[test]
    fn sumber_alat_tidak_menyebut_nama_entitas_repo() {
        // Penjaga kelas ini lahir dari sapuan manual: sebuah komentar di `audit.rs` masih mencontoh
        // path service repo induk (`svc-satu/...`) dan tidak ada tes yang menangkapnya. Nama
        // entitas — key service, stack, id aplikasi, id produk, nama aktor — tidak boleh muncul di
        // kode alat mana pun (Rust, JS, CSS), termasuk di komentar.
        let root = root_uji();
        let model = muat_model(&root).expect("manifest root uji harus valid");
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
        let berkas = sumber_alat();
        let mut semua: Vec<(String, String)> = berkas;
        for aset in ["render.js", "view.css"] {
            semua.push((
                aset.to_string(),
                fs::read_to_string(dir_aset().join(aset)).expect("aset tampilan terbaca"),
            ));
        }
        // Hanya nama yang berbentuk identifier (`svc-satu`, `Node.js`, `TypeScript`) yang dituntut
        // absent. Aktor dan id produk yang hanyalah kata Indonesia biasa tidak dituntut: alat ini
        // boleh berkomentar pakai bahasa itu, dan `layak_token` adalah definisi identifier yang sama
        // dipakai `audit` — satu aturan, tidak dua yang saling tumpang tindih.
        let identitas: Vec<&String> = nama.iter().filter(|n| n.len() > 3 && layak_token(n)).collect();
        assert!(
            !identitas.is_empty(),
            "tidak ada satu pun nama identifier untuk diperiksa — penjaga ini jadi no-op"
        );
        let mut ditemukan: Vec<String> = Vec::new();
        for (asal, isi) in &semua {
            let produksi = produksi_dari(isi).to_lowercase();
            for n in identitas.iter() {
                if produksi.contains(&n.to_lowercase()) {
                    ditemukan.push(format!("{asal}:memuat:`{n}`"));
                }
            }
        }
        assert!(
            ditemukan.is_empty(),
            "kode alat mengenal nama entitas data repo ini: {ditemukan:?}"
        );
    }

    #[test]
    fn sitasi_baris_di_dokumen_masih_hidup() {
        // Dokumenalat punya hak yang sama dengan kode: tiap kutipan `nama.rs:NNN` harus masih jatuh di
        // baris `assert` atau `fn`. Nomor baris bergeser diam-diam setiap kali ada tes disisipkan —
        // terukur 2026-10-25, satu sisipan menggeser tiga kutipan dan tidak ada satu tes pun yang tahu.
        // Yang dibaca selalu termasuk README/CHANGELOG milik alat (ada di repo mandiri), plus dokumen
        // provenance repo induk kalau memang ada.
        let mut dok: Vec<(String, String)> = Vec::new();
        for nama in ["README.md", "CHANGELOG.md", "how_to_run.md"] {
            let path = dir_aset().parent().expect("crate punya induk").join(nama);
            if let Ok(isi) = fs::read_to_string(&path) {
                dok.push((nama.to_string(), isi));
            }
        }
        let provenance = root_uji().join("docs").join("view").join("README.md");
        if let Ok(isi) = fs::read_to_string(&provenance) {
            dok.push(("docs/view/README.md".to_string(), isi));
        }
        assert!(
            dok.len() >= 2,
            "dokumen alat sendiri (README + CHANGELOG) harus terbaca, dapat {} — tes ini tidak boleh \
             jadi no-op di repo mandiri",
            dok.len()
        );
        use std::collections::HashMap;
        let mut berkas: HashMap<String, Vec<String>> = HashMap::new();
        for (nama, isi) in sumber_alat() {
            berkas.insert(nama, isi.lines().map(str::to_string).collect());
        }
        for nama in ["render.js", "view.css", "uji.rs"] {
            let isi = fs::read_to_string(dir_aset().join(nama)).expect("aset tampilan terbaca");
            berkas.insert(nama.to_string(), isi.lines().map(str::to_string).collect());
        }
        let mut rusak: Vec<String> = Vec::new();
        let mut jumlah = 0usize;
        for (nama_dok, isi_dok) in &dok {
            let karakter: Vec<char> = isi_dok.chars().collect();
            let mut i = 0usize;
            while i < karakter.len() {
                if !(karakter[i].is_alphanumeric() || karakter[i] == '_' || karakter[i] == '.' || karakter[i] == '/') {
                    i += 1;
                    continue;
                }
                let mut j = i;
                while j < karakter.len()
                    && (karakter[j].is_alphanumeric() || matches!(karakter[j], '_' | '-' | '.' | '/'))
                {
                    j += 1;
                }
                let token: String = karakter[i..j].iter().collect();
                let nama = token.rsplit('/').next().unwrap_or(token.as_str()).to_string();
                if !nama.ends_with(".rs") && !nama.ends_with(".js") && !nama.ends_with(".css") {
                    i = j.max(i + 1);
                    continue;
                }
                if j >= karakter.len() || karakter[j] != ':' {
                    i = j.max(i + 1);
                    continue;
                }
                let mut k = j + 1;
                while k < karakter.len() && karakter[k].is_ascii_digit() {
                    k += 1;
                }
                if k == j + 1 {
                    i = j.max(i + 1);
                    continue;
                }
                let awal: usize = karakter[(j + 1)..k].iter().collect::<String>().parse().expect("angka");
                let mut akhir = awal;
                if k + 1 < karakter.len() && karakter[k] == '-' && karakter[k + 1].is_ascii_digit() {
                    let mut m = k + 1;
                    while m < karakter.len() && karakter[m].is_ascii_digit() {
                        m += 1;
                    }
                    akhir = karakter[(k + 1)..m].iter().collect::<String>().parse().expect("angka");
                }
                jumlah += 1;
                match berkas.get(&nama) {
                    None => rusak.push(format!("{nama_dok}: {nama}:{awal} — berkasnya sendiri tidak dikenal")),
                    Some(baris) => {
                        if akhir > baris.len() {
                            rusak.push(format!(
                                "{nama_dok}: {nama}:{akhir} — berkas hanya punya {} baris",
                                baris.len()
                            ));
                        } else if akhir != awal && !baris[akhir - 1].trim_start().starts_with('}') {
                            rusak.push(format!(
                                "{nama_dok}: {nama}:{akhir} — ujung rentang bukan penutup blok: `{}`",
                                baris[akhir - 1].trim()
                            ));
                        } else if !baris[awal - 1].contains("assert") && !baris[awal - 1].contains("fn ") {
                            rusak.push(format!(
                                "{nama_dok}: {nama}:{awal} — bukan lagi assert atau definisi fn: `{}`",
                                baris[awal - 1].trim()
                            ));
                        }
                    }
                }
                i = k.max(i + 1);
            }
        }
        assert!(
            jumlah >= 5,
            "total kutipan baris di dokumen alat harus minimal 5, dapat {jumlah} — parsingnya rusak",
        );
        assert!(rusak.is_empty(), "kutipan baris di dokumen alat busuk: {rusak:?}");
    }

    #[test]
    fn tiap_penjaga_anti_hardcode_didokumentasikan() {
        // Penjaga yang tidak disebut di dokumen akan dibaca sebagai kebetulan, bukan aturan. Yang dibaca
        // adalah README milik alat ini, jadi aturan ini juga berlaku di repo mandiri — bukan dokumen
        // milik repo induk.
        let readme = dir_aset()
            .parent()
            .expect("crate punya induk")
            .join("README.md");
        let isi_dok = fs::read_to_string(readme).expect("README alat terbaca");
        let awalan = [
            "sumber_alat_",
            "render_js_",
            "kode_alat_",
            "nama_contoh_",
            "bentuk_terdaftar_",
            "penjaga_",
        ];
        let mut hilang: Vec<String> = Vec::new();
        let mut ditemukan = 0usize;
        for (_nama, isi) in sumber_alat() {
            // Penjaga justru hidup di BLOK tes, jadi sapuan memakai isi penuh, bukan wilayah produksi.
            for baris in isi.lines() {
                let bersih = baris.trim();
                let Some(sisa) = bersih.strip_prefix("fn ") else { continue };
                let nama: String = sisa
                    .chars()
                    .take_while(|c| c.is_alphanumeric() || *c == '_')
                    .collect();
                if nama.is_empty() || !awalan.iter().any(|a| nama.starts_with(a)) {
                    continue;
                }
                ditemukan += 1;
                if !isi_dok.contains(&nama) {
                    hilang.push(nama);
                }
            }
        }
        assert!(
            ditemukan >= 6,
            "penjaga anti-hardcode harus benar-benar ada di sumber, dapat {ditemukan} — tes ini tidak \
             boleh lolos diam-diam kalau penamaannya berubah"
        );
        assert!(hilang.is_empty(), "penjaga tidak disebut di README alat: {hilang:?}");
    }

    #[test]
    fn petunjuk_menyebut_root_yang_dipakai_bukan_asumsi_folder() {
        let pesan = petunjuk("generate", std::path::Path::new("/data/proyek-saya"));
        assert!(pesan.contains("generate"), "nama perintahnya hilang: {pesan}");
        assert!(
            pesan.contains("--root /data/proyek-saya"),
            "petunjuk tidak mengulang root yang benar-benar dibaca, jadi pembaca bisa tersesat: {pesan}"
        );
        assert!(
            !pesan.contains("tools/feature-view"),
            "petunjuk tidak boleh mengeras lokasi alat di repo pemakai: {pesan}"
        );
    }

    #[test]
    fn peta_fungsi_di_readme_menunjuk_definisinya_sendiri() {
        // `sitasi_baris_di_dokumen_masih_hidup` hanya menuntut baris NNN memuat `assert` atau `fn`,
        // jadi ia tetap lolos ketika nomornya bergeser ke `fn` lain di berkas yang sama — terukur
        // saat `petunjuk` pindah baris. Tabel "Peta fungsi" menulis NAMA fungsinya di sebelah nomor,
        // jadi di situ nama dan nomor bisa dicocokkan langsung.
        use std::collections::HashMap;
        let readme = dir_aset().parent().expect("crate punya induk").join("README.md");
        let isi = fs::read_to_string(&readme).expect("README alat harus ada, termasuk di repo mandiri");
        let mut sumber: HashMap<String, Vec<String>> = HashMap::new();
        for (nama, teks) in sumber_alat() {
            let kunci = nama.rsplit('/').next().unwrap_or(nama.as_str()).to_string();
            sumber.insert(kunci, teks.lines().map(str::to_string).collect());
        }
        // `sumber_alat()` memang mengabaikan `uji.rs` (isinya fixture berbahasa Indonesia), tapi
        // tabel Peta fungsi juga menunjuk fungsi di dalamnya, jadi berkas itu dibaca sendiri.
        let uji = fs::read_to_string(dir_aset().join("uji.rs")).expect("sumber uji.rs terbaca");
        sumber.insert("uji.rs".to_string(), uji.lines().map(str::to_string).collect());
        let mut diperiksa = 0usize;
        let salah: Vec<String> = isi
            .lines()
            .filter(|b| b.starts_with("| `"))
            .filter_map(|baris| {
                let kolom: Vec<&str> = baris.split('|').map(str::trim).collect();
                if kolom.len() < 3 {
                    return None;
                }
                let nama_fn = kolom[1].trim_matches('`');
                let tautan = kolom[2].trim_matches('`');
                let pos = tautan.find(".rs:")?;
                let berkas = tautan[..pos + 3].to_string();
                let nomor: usize = tautan[pos + 4..]
                    .chars()
                    .take_while(char::is_ascii_digit)
                    .collect::<String>()
                    .parse()
                    .ok()?;
                diperiksa += 1;
                let kunci = berkas.rsplit('/').next().unwrap_or(berkas.as_str()).to_string();
                let baris_sumber = match sumber.get(&kunci).and_then(|b| b.get(nomor - 1)) {
                    Some(b) => b.clone(),
                    None => return Some(format!("{berkas}:{nomor} — berkas atau barisnya tidak terbaca")),
                };
                let fungsi = format!("fn {nama_fn}");
                let rekor = format!("struct {nama_fn}");
                if baris_sumber.contains(&fungsi) || baris_sumber.contains(&rekor) {
                    None
                } else {
                    Some(format!(
                        "{berkas}:{nomor} bukan definisi `{nama_fn}`: `{}`",
                        baris_sumber.trim()
                    ))
                }
            })
            .collect();
        assert!(
            diperiksa >= 6,
            "tabel Peta fungsi harus menyumbang minimal enam baris yang teriksa, dapat {diperiksa} — \
             kalau polanya berubah, penjaga ini tidak boleh diam-diam jadi no-op"
        );
        assert!(salah.is_empty(), "Peta fungsi README menunjuk baris yang salah: {salah:?}");
    }

    #[test]
    fn tabel_modul_di_dokumen_mencocokkan_angka_sahnya() {
        // Kolom "Baris" dan "Tes" pada tabel modul ditulis dengan tangan dan sudah dua kali basi
        // dalam satu hari kerja. Penjaga ini menghitung ulang keduanya dari berkas, jadi dokumen
        // tidak bisa mengklaim dirinya sinkron.
        let sumber_crate = dir_aset();
        let akar_crate = sumber_crate.parent().expect("crate punya induk");
        let mut dokumen: Vec<(String, String)> = Vec::new();
        for nama in ["README.md", "CHANGELOG.md", "how_to_run.md"] {
            if let Ok(isi) = fs::read_to_string(akar_crate.join(nama)) {
                dokumen.push((nama.to_string(), isi));
            }
        }
        let provenance = root_uji().join("docs").join("view").join("README.md");
        if let Ok(isi) = fs::read_to_string(&provenance) {
            dokumen.push(("docs/view/README.md".to_string(), isi));
        }
        let mut diperiksa = 0usize;
        let mut salah: Vec<String> = Vec::new();
        for (nama_dok, isi_dok) in &dokumen {
            for baris in isi_dok.lines() {
                if !baris.starts_with("| `src/") {
                    continue;
                }
                let kolom: Vec<&str> = baris.split('|').map(str::trim).collect();
                if kolom.len() < 5 {
                    continue;
                }
                let berkas = kolom[1].trim_matches('`');
                let Ok(angka_baris) = kolom[3].parse::<usize>() else { continue };
                let Ok(angka_tes) = kolom[4].parse::<usize>() else { continue };
                let nama_berkas = match berkas.rsplit('/').next() {
                    Some(n) if n.ends_with(".rs") => n.to_string(),
                    _ => continue,
                };
                let Ok(isi_berkas) = fs::read_to_string(dir_aset().join(&nama_berkas)) else { continue };
                let nyata_baris = isi_berkas.lines().count();
                // Dithitung per baris, BUKAN per kemunculan substring: jarum ini dulu
                // `matches("#[test]")` dan ia ikut menghitung teks `"#[test]"` di dalam baris
                // ini sendiri, jadi angka main.rs membengkak satu dan penjaganya tidak pernah
                // bisa melihatnya.
                let nyata_tes = isi_berkas
                    .lines()
                    .filter(|b| b.trim_start().starts_with("#[test]"))
                    .count();
                diperiksa += 1;
                if nyata_baris != angka_baris {
                    salah.push(format!(
                        "{nama_dok}: {berkas} tertulis {angka_baris} baris, nyata {nyata_baris}"
                    ));
                }
                if nyata_tes != angka_tes {
                    salah.push(format!(
                        "{nama_dok}: {berkas} tertulis {angka_tes} tes, nyata {nyata_tes}"
                    ));
                }
            }
        }
        assert!(
            diperiksa >= 10,
            "tabel modul harus menyumbang minimal sepuluh baris teriksa, dapat {diperiksa} — \
             kalau bentuk tabel berubah, penjaga ini tidak boleh diam-diam jadi no-op"
        );
        assert!(salah.is_empty(), "angka tabel modul di dokumen tidak sah: {salah:?}");
    }

    #[test]
    fn kode_alat_tidak_menyebut_letaknya_di_repo_pemakai() {
        // Penjaga bentuk repo: alat ini pindah-pindah folder (jadi repo mandiri, di-vendor, diletakkan
        // di `tools/`). Satu pesan yang menyebut `tools/feature-view` sudah cukup membuat petunjuknya
        // salah di tempat lain — dan pesan itu ter-bake ke index.html, jadi salahnya ikut ter-commit.
        let mut sumber = sumber_alat();
        for nama in ["render.js", "view.css"] {
            sumber.push((
                nama.to_string(),
                fs::read_to_string(dir_aset().join(nama)).expect("aset tampilan terbaca"),
            ));
        }
        for (asal, isi) in &sumber {
            let produksi = produksi_dari(isi);
            assert!(
                !produksi.contains("tools/feature-view"),
                "{asal} menyebut path alat di repo ini; petunjuk akan salah di repo lain"
            );
        }
    }
}
