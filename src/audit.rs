//! Mesin audit: mengubah tiga kolom klaim menjadi token, membuktikan tiap token lewat tiga
//! pintu (isi berkas, nama berkas, keberadaan berkas), dan menghitung vonis per baris.

use std::collections::HashMap;
use std::fs;
use std::path::Path;
use crate::data::*;
use crate::setelan::*;

/// Default batas baca satu berkas bukti; dinaikkan lewat env `PETA_BATAS_BACA_BUKTI`.
pub(crate) const BATAS_BACA_BUKTI: u64 = 4_000_000;

/// Batas yang berlaku sekarang untuk pembacaan berkas bukti (default kecuali env mengubahnya).
pub(crate) fn batas_baca_bukti() -> u64 {
    angka_env("PETA_BATAS_BACA_BUKTI", BATAS_BACA_BUKTI)
}

/// Bangun HTML dari model. CSS dan JS dibaca dari folder `public/` (lihat `aset`), supaya
/// ubahan tampilan langsung ikut ke file hasil generate tanpa perlu build ulang biner.
/// Kata layak jadi bukti: cukup panjang dan berbentuk identitas, bukan kata biasa.
/// Huruf besar di awal saja TIDAK cukup — kalimat Indonesia mulai dengan huruf besar
/// ("Layar", "Halaman"), dan mencocokkannya ke sumber akan menghasilkan merah palsu.
/// Yang diterima: mengandung angka, mengandung `_ . / :`, huruf besar di tengah
/// (ConfigProvider), atau huruf besar semua (QRIS).
pub(crate) fn layak_token(kata: &str) -> bool {
    let huruf: Vec<char> = kata.chars().collect();
    if huruf.len() < 3 {
        return false;
    }
    if huruf.iter().any(|c| c.is_ascii_digit()) {
        return true;
    }
    if kata.chars().any(|c| matches!(c, '_' | '.' | ':' | '/')) {
        return true;
    }
    if huruf
        .iter()
        .skip(1)
        .any(|c| c.is_uppercase())
    {
        return true;
    }
    huruf.iter().all(|c| !c.is_lowercase())
}

/// Token yang diturunkan mekanis dari kolom `titik_masuk`: belah per titik koma, ambil
/// rangkaian huruf/angka/`_ . / : -`, buang yang tidak berbentuk identitas, deduplikasi.
/// Pangkas tanda baca di ujung rangkaian. Tanpa ini, kata prosa yang kebetulan mengakhiri kalimat
/// (`pengguna.`) sempat lolos sebagai token: `layak_token` memang menerima `.` di tengah
/// nama berkas, jadi tanda baca ujung harus dibuang lebih dulu.
pub(crate) fn rapikan_token(kata: &str) -> String {
    kata.trim_matches(|c: char| matches!(c, '.' | ':' | '-' | '_'))
        .to_string()
}

pub(crate) fn token_uji(entry: &str) -> Vec<String> {
    let mut mentah: Vec<String> = Vec::new();
    for bagian in entry.split(';') {
        let mut kata = String::new();
        for c in bagian.chars() {
            if c.is_ascii_alphanumeric() || matches!(c, '_' | '.' | '/' | ':' | '-') {
                kata.push(c);
            } else if !kata.is_empty() {
                mentah.push(std::mem::take(&mut kata));
            }
        }
        if !kata.is_empty() {
            mentah.push(kata);
        }
    }
    let hasil: Vec<String> = mentah
        .iter()
        .map(|k| rapikan_token(k))
        .filter(|k| layak_token(k))
        .collect();
    let mut hasil = hasil;
    hasil.sort();
    hasil.dedup();
    hasil
}

/// Pemetaan vonis, murni supaya aturannya bisa dites tanpa menyentuh disk.
///
/// Enam vonis dan artinya:
/// - `hijau-isi` : semua token klaim, dan minimal satu terbukti dari ISI berkas bukti;
/// - `hijau-nama` : semua token klaim terbukti, tapi tidak ada yang lewat isi — hanya nama berkas
///   yang dicantumkan sebagai bukti. Ini tidak boleh disamakan dengan `hijau-isi`;
/// - `kuning` : sebagian token terbukti, sebagian tidak;
/// - `kuning-hampa` : baris belum lengkap DAN ada token yang tidak ada di mana pun dalam service;
/// - `merah-sempit` : baris aktif yang nol token terbukti, tapi tokennya memang ada di service
///   (yang salah adalah kolom bukti yang terlalu sempit);
/// - `merah-hampa` : baris aktif dengan klaim yang tidak ditemukan di berkas mana pun dalam service.
pub(crate) fn vonis(
    peran: Peran,
    jumlah_token: usize,
    cocok: usize,
    cocok_isi: usize,
    hampa: usize,
    ada_bukti: bool,
) -> &'static str {
    if jumlah_token == 0 {
        return "tidak-diuji";
    }
    if peran != Peran::Utama && peran != Peran::Separuh {
        return "tidak-diuji";
    }
    if hampa > 0 {
        return if peran == Peran::Utama { "merah-hampa" } else { "kuning-hampa" };
    }
    if cocok == 0 {
        return if peran == Peran::Utama {
            // Tanpa berkas bukti yang terbaca pun klaimnya tetap tidak terbukti.
            let _ = ada_bukti;
            "merah-sempit"
        } else {
            "kuning"
        };
    }
    if cocok < jumlah_token {
        return "kuning";
    }
    if cocok_isi > 0 {
        "hijau-isi"
    } else {
        "hijau-nama"
    }
}

/// Isi berkas bukti, sekali baca per path. Berkas > batas dianggap tidak terbaca dan
/// itu dicatat sebagai alasan, bukan diam-diam dilewati.
pub(crate) fn baca_bukti(
    root: &Path,
    path: &str,
    cache: &mut HashMap<String, Option<String>>,
) -> Option<String> {
    if let Some(sudah) = cache.get(path) {
        return sudah.clone();
    }
    let penuh = root.join(path);
    let batas = batas_baca_bukti();
    let hasil = fs::metadata(&penuh).ok().and_then(|meta| {
        if meta.len() > batas {
            return None;
        }
        fs::read_to_string(&penuh).ok()
    });
    cache.insert(path.to_string(), hasil.clone());
    hasil
}

pub(crate) struct BarisAudit {
    pub(crate) service: String,
    pub(crate) id: String,
    pub(crate) line: usize,
    pub(crate) status: String,
    pub(crate) vonis: &'static str,
    pub(crate) jumlah: usize,
    pub(crate) cocok: usize,
    pub(crate) cocok_isi: usize,
    pub(crate) gagal: Vec<GagalToken>,
    pub(crate) tanpa_token: bool,
    pub(crate) tanpa_baca: bool,
    pub(crate) luar: Vec<String>,
}

/// Satu klaim yang tidak terbukti dari berkas yang ditunjuk, beserta sejauh mana ia ditemukan
/// di tempat lain: `dalam` = berkas service sendiri, `silang` = service lain yang menyebutnya.
pub(crate) struct GagalToken {
    pub(crate) token: String,
    pub(crate) dalam: usize,
    pub(crate) kolom: String,
    pub(crate) silang: Option<(String, usize)>,
}

/// Klaim sebuah baris = token identitas dari TIGA kolom yang dibaca orang: titik masuk, ringkasan,
/// dan cara kerja. Kolom asal ikut disimpan supaya laporan bisa menunjuk ke mana klaimnya ditulis.
pub(crate) fn klaim_baris(fitur: &Feature) -> Vec<(String, String)> {
    let sumber = [
        ("titik_masuk", fitur.entry.clone()),
        ("ringkasan", fitur.summary.clone()),
        ("cara_kerja", fitur.how.clone()),
    ];
    let mut hasil: Vec<(String, String)> = Vec::new();
    for (kolom, teks) in sumber {
        for token in token_uji(&teks) {
            if hasil.iter().any(|(_, sudah)| *sudah == token) {
                continue;
            }
            hasil.push((kolom.to_string(), token));
        }
    }
    hasil
}

/// Batas penyapuan folder service saat mencari sokongan sebuah token gagal. Sapuan dibatasi agar
/// `audit` tetap jalan di repo besar; kalau batas tersentuh, hasilnya dilaporkan sebagai terpotong,
/// bukan disimpulkan nol.
pub(crate) const INDEKS_LEWATI: [&str; 8] = [
    "target",
    "build",
    "node_modules",
    ".dart_tool",
    "vendor",
    "dist",
    ".git",
    "storage",
];

pub(crate) const INDEKS_EKSTENSI: [&str; 12] = [
    "rs", "go", "php", "js", "jsx", "ts", "tsx", "dart", "py", "html", "ini", "md",
];

pub(crate) const INDEKS_BATAS_BERKAS: usize = 12_000;

pub(crate) const INDEKS_BATAS_BYTE: u64 = 24_000_000;

/// Batas sapu folder service yang berlaku: default kecuali env mengubahnya. Sapuan yang kena batas
/// dilaporkan sebagai `terbatas`, jadi angkanya tetap jujur sebagai batas bawah.
pub(crate) fn batas_indeks() -> (usize, u64) {
    (
        angka_env("PETA_INDEKS_BATAS_BERKAS", INDEKS_BATAS_BERKAS as u64) as usize,
        angka_env("PETA_INDEKS_BATAS_BYTE", INDEKS_BATAS_BYTE),
    )
}

#[derive(Debug)]
pub(crate) struct Indeks {
    pub(crate) isi: Vec<String>,
    pub(crate) berkas: usize,
    pub(crate) terbatas: bool,
    /// Target yang semestinya ikut tersapu tapi tidak bisa: folder yang gagal dibuka dan
    /// berkas berekstensi indeks yang gagal dibaca atau bukan UTF-8. Tanpa dihitung, satu
    /// klaim yang hanya hidup di berkas seperti itu dituduh hampa tanpa satu pun tanda bahwa
    /// yang bolong adalah alat ini. Terukur pada root yang sungguh disapu repo pertama: 3 target
    /// dari 8.794 berkas berekstensi indeks.
    pub(crate) bolong: usize,
}

impl Indeks {
    /// Indeks tanpa isi: pembanding saat sebuah baris menunjuk service yang tidak punya indeks.
    pub(crate) fn kosong() -> Self {
        Indeks {
            isi: Vec::new(),
            berkas: 0,
            terbatas: false,
            bolong: 0,
        }
    }
}

pub(crate) fn indeks_service(root: &Path, folder: &str) -> Indeks {
    let (batas_berkas, batas_byte) = batas_indeks();
    let batas_baca = batas_baca_bukti();
    let mut indeks = Indeks {
        isi: Vec::new(),
        berkas: 0,
        terbatas: false,
        bolong: 0,
    };
    if folder.is_empty() {
        return indeks;
    }
    let mut tumpuk = vec![root.join(folder)];
    while let Some(dir) = tumpuk.pop() {
        if indeks.berkas >= batas_berkas || total_byte(&indeks) >= batas_byte {
            indeks.terbatas = true;
            break;
        }
        let entri = match fs::read_dir(&dir) {
            Ok(e) => e,
            Err(_) => {
                indeks.bolong += 1;
                continue;
            }
        };
        for entri in entri.flatten() {
            let path = entri.path();
            let nama = entri.file_name().to_string_lossy().to_string();
            if path.is_dir() {
                if !INDEKS_LEWATI.contains(&nama.as_str()) {
                    tumpuk.push(path);
                }
                continue;
            }
            let ekstensi = path
                .extension()
                .map(|e| e.to_string_lossy().to_lowercase())
                .unwrap_or_default();
            if !INDEKS_EKSTENSI.contains(&ekstensi.as_str()) {
                continue;
            }
            match fs::metadata(&path) {
                Ok(meta) if meta.len() <= batas_baca => {}
                _ => {
                    indeks.terbatas = true;
                    continue;
                }
            }
            match fs::read_to_string(&path) {
                Ok(teks) => {
                    indeks.isi.push(teks);
                    indeks.berkas += 1;
                }
                // Berkas biner atau UTF-8 cacat: bukan berarti klaimnya bohong, tapi alat ini
                // tidak bisa membacanya. Dicatat sebagai bolong supaya vonis hampa bisa
                // dibedakan dari "tidak sampai terbaca".
                Err(_) => indeks.bolong += 1,
            }
        }
    }
    indeks
}

pub(crate) fn total_byte(indeks: &Indeks) -> u64 {
    indeks.isi.iter().map(|s| s.len() as u64).sum()
}

/// Berapa berkas lain di service ini yang menyebut token. Ini yang memisahkan "buktinya sempit"
/// dari "klaimnya tidak ada di kode mana pun".
pub(crate) fn sokong_token(token: &str, indeks: &Indeks) -> usize {
    indeks.isi.iter().filter(|s| s.contains(token)).count()
}

/// Klaim sebuah baris boleh menunjuk kode service lain — di monorepo, baris satu service yang
/// menyebut controller milik service lain itu sah. Kalau token tidak ada di service sendiri tetapi
/// ada di service lain, ia tidak boleh disebut hampa; ia dilaporkan sebagai klaim lintas service.
pub(crate) fn sokong_lintas(
    token: &str,
    semua: &HashMap<String, Indeks>,
    service_sendiri: &str,
) -> Option<(String, usize)> {
    // Kunci diurutkan dulu: tanpa itu service yang sama-sama mendukung satu token terpilih
    // menurut urutan acak HashMap, jadi laporan yang sama berubah antar dua pembacaan.
    let mut kunci: Vec<&String> = semua.keys().filter(|k| k.as_str() != service_sendiri).collect();
    kunci.sort();
    let mut terbaik: Option<(String, usize)> = None;
    for key in kunci {
        let ada = sokong_token(token, &semua[key]);
        if ada > 0 && terbaik.as_ref().map(|(_, b)| ada > *b).unwrap_or(true) {
            terbaik = Some((key.clone(), ada));
        }
    }
    terbaik
}

/// Service yang penyapuannya terpotong, TERURUT. Sumbernya HashMap dan urutan iterasinya acak
/// per proses, jadi tanpa penyortiran baris CATATAN yang sama berubah antara dua pembacaan dan
/// laporan tidak bisa dibandingkan atau dipakai sebagai bukti.
pub(crate) fn senarai_terbatas(semua: &HashMap<String, Indeks>) -> Vec<String> {
    let mut hasil: Vec<String> = semua
        .iter()
        .filter(|(_, i)| i.terbatas)
        .map(|(k, _)| k.clone())
        .collect();
    hasil.sort();
    hasil
}

/// Berapa target yang semestinya tersapu tapi tidak bisa, dan di service mana saja.
/// Dipakai untuk satu baris CATATAN: tanpa itu vonis hampa pada berkas biner terbaca seperti
/// tuduhan bahwa barisnya mengarang.
pub(crate) fn senarai_bolong(semua: &HashMap<String, Indeks>) -> (usize, Vec<String>) {
    let mut nama: Vec<String> = semua
        .iter()
        .filter(|(_, i)| i.bolong > 0)
        .map(|(k, _)| k.clone())
        .collect();
    nama.sort();
    let jumlah = semua.values().map(|i| i.bolong).sum();
    (jumlah, nama)
}

/// Bukti yang menunjuk ke luar folder service barisnya sendiri. Dipakai untuk satu baris
/// CATATAN (tidak mengubah vonis). Pembandangnya per KOMPNEN, bukan awalan teks: dengan
/// `starts_with` string, bukti `svc-satuLain/x.go` disangka milik folder `svc-satu`, jadi
/// bukti yang sungguh keluar service tidak pernah dilaporkan sama sekali.
pub(crate) fn bukti_keluar_service(evidence: &[String], folder: &str) -> Vec<String> {
    if folder.is_empty() {
        return Vec::new();
    }
    let induk = Path::new(folder);
    evidence
        .iter()
        .filter(|b| !Path::new(b.as_str()).starts_with(induk))
        .cloned()
        .collect()
}

/// Cocokkan klaim tiap sub-fitur terhadap ISI berkas bukti-nya, dan terhadap seluruh folder
/// service-nya. `check` membuktikan path ada; ini membuktikan isinya masih menyebut apa yang
/// barisnya klaim.
/// Hasil `audit` yang dibutuhkan CLI: berapa baris yang membuat exit 1 dan berapa yang masih
/// kuning, supaya pesan "OK" tidak pernah terdengar lebih luas dari yang sebenarnya diperiksa.
pub(crate) struct RingkasAudit {
    pub(crate) merah: usize,
    pub(crate) kuning: usize,
    pub(crate) tak_teruji: usize,
}

pub(crate) fn audit(model: &Model, root: &Path) -> RingkasAudit {
    let folder_service: HashMap<&str, &str> = model
        .services
        .iter()
        .map(|s| (s.key.as_str(), s.folder.as_str()))
        .collect();
    let mut cache: HashMap<String, Option<String>> = HashMap::new();
    let mut indeks_cache: HashMap<String, Indeks> = HashMap::new();
    for service in &model.services {
        indeks_cache.insert(service.key.clone(), indeks_service(root, &service.folder));
    }
    let mut baris: Vec<BarisAudit> = Vec::new();
    let indeks_kosong = Indeks::kosong();

    for fitur in &model.features {
        let klaim = klaim_baris(fitur);
        let isi: Vec<String> = fitur
            .evidence
            .iter()
            .filter_map(|p| baca_bukti(root, p, &mut cache))
            .collect();
        let ada_bukti = !isi.is_empty();
        let folder = folder_service
            .get(fitur.service.as_str())
            .copied()
            .unwrap_or("");
        // Baris yang menunjuk service tanpa indeks (sudah ditolak `validate`, tapi `audit` tidak boleh
        // panic di tengah sapuan) dihitung dari indeks kosong: klainya jatuh ke hampa dan dilaporkan.
        let indeks = indeks_cache
            .get(fitur.service.as_str())
            .unwrap_or(&indeks_kosong);
        let mut cocok = 0usize;
        let mut cocok_isi = 0usize;
        let mut hampa = 0usize;
        let mut gagal: Vec<GagalToken> = Vec::new();
        for (kolom, t) in &klaim {
            let lewat_isi = isi.iter().any(|s| s.contains(t));
            let lewat_nama = fitur.evidence.iter().any(|b| b.contains(t));
            let lewat_ada = cocok_token_berkas(root, t, folder);
            if lewat_isi {
                cocok += 1;
                cocok_isi += 1;
            } else if lewat_nama || lewat_ada {
                cocok += 1;
            } else {
                let dalam = sokong_token(t, indeks);
                let silang = if dalam == 0 {
                    sokong_lintas(t, &indeks_cache, fitur.service.as_str())
                } else {
                    None
                };
                if dalam == 0 && silang.is_none() {
                    hampa += 1;
                }
                gagal.push(GagalToken {
                    token: t.clone(),
                    dalam,
                    kolom: kolom.clone(),
                    silang,
                });
            }
        }
        let peran = peran_status(&fitur.status, &model.config);
        let keputusan = vonis(peran, klaim.len(), cocok, cocok_isi, hampa, ada_bukti);
        let luar = bukti_keluar_service(&fitur.evidence, folder);
        baris.push(BarisAudit {
            service: fitur.service.clone(),
            id: fitur.id.clone(),
            line: fitur.line,
            status: fitur.status.clone(),
            vonis: keputusan,
            jumlah: klaim.len(),
            cocok,
            cocok_isi,
            gagal,
            tanpa_token: klaim.is_empty(),
            tanpa_baca: !ada_bukti,
            luar,
        });
    }

    let hitung = |v: &str| baris.iter().filter(|b| b.vonis == v).count();
    let hampa_baris = hitung("merah-hampa");
    let sempit_baris = hitung("merah-sempit");
    let merah = hampa_baris + sempit_baris;
    let aktif = model.config.status_peran("utama");

    println!(
        "AUDIT klaim terhadap isi kode — {} sub-fitur diperiksa, klaim diambil dari titik_masuk + ringkasan + cara_kerja",
        baris.len()
    );
    println!();
    println!(
        "{:<34}{:>6}{:>10}{:>11}{:>8}{:>8}{:>9}{:>13}",
        "service", "baris", "hijau-isi", "hijau-nama", "kuning", "merah", "hampa", "tidak diuji"
    );
    for service in &model.services {
        let milik: Vec<&BarisAudit> =
            baris.iter().filter(|b| b.service == service.key).collect();
        if milik.is_empty() {
            continue;
        }
        let satu = |v: &str| milik.iter().filter(|b| b.vonis == v).count();
        println!(
            "{:<34}{:>6}{:>10}{:>11}{:>8}{:>8}{:>9}{:>13}",
            service.key,
            milik.len(),
            satu("hijau-isi"),
            satu("hijau-nama"),
            satu("kuning") + satu("kuning-hampa"),
            satu("merah-hampa") + satu("merah-sempit"),
            milik
                .iter()
                .filter(|b| b.vonis != "tidak-diuji")
                .map(|b| b.gagal.iter().filter(|g| g.dalam == 0 && g.silang.is_none()).count())
                .sum::<usize>(),
            satu("tidak-diuji"),
        );
    }
    println!(
        "{:<34}{:>6}{:>10}{:>11}{:>8}{:>8}{:>9}{:>13}",
        "TOTAL",
        baris.len(),
        hitung("hijau-isi"),
        hitung("hijau-nama"),
        hitung("kuning") + hitung("kuning-hampa"),
        merah,
        baris
            .iter()
            .filter(|b| b.vonis != "tidak-diuji")
            .map(|b| b.gagal.iter().filter(|g| g.dalam == 0 && g.silang.is_none()).count())
            .sum::<usize>(),
        hitung("tidak-diuji"),
    );
    // Kolom terakhir tidak sesatuan dengan yang lain dan tanpa kalimat ini pembaca tabel menyangka
    // semuanya baris: `hampa` menghitung KLAIM, bukan baris. Satu baris berstatus separuh boleh
    // menunjuk tujuh klaim tak terbukti sementara kolom `merah`-nya 0.
    println!("  Keterangan kolom: `hampa` = jumlah KLAIM yang tidak disebut berkas mana pun di service-nya; kolom lain = jumlah BARIS.");

    let cetak = |judul: &str, nama: &str, catatan: &str| {
        let senarai: Vec<&BarisAudit> = baris.iter().filter(|b| b.vonis == nama).collect();
        println!();
        println!("{} — {}: {} baris", judul, catatan, senarai.len());
        if senarai.is_empty() {
            println!("  tidak ada");
        }
        for b in senarai.iter().take(30) {
            println!(
                "  {}:{} [{}] {} dari {} terbukti dari isi, {} lewat nama berkas",
                b.id,
                b.line,
                b.status,
                b.cocok_isi,
                b.jumlah,
                b.cocok - b.cocok_isi
            );
            for g in &b.gagal {
                let di_lain = match &g.silang {
                    Some((service_lain, n)) => format!("ada di service `{service_lain}` ({n} berkas)"),
                    None => format!("{} berkas service menyebutnya", g.dalam),
                };
                println!("      `{}` ({}) — {}", g.token, g.kolom, di_lain);
            }
        }
        if senarai.len() > 30 {
            println!("  ... {} lainnya", senarai.len() - 30);
        }
    };
    cetak(
        "MERAH HAMPA",
        "merah-hampa",
        "klaim baris aktif yang tidak disebut di berkas mana pun dalam service-nya",
    );
    cetak(
        "MERAH SEMPIT",
        "merah-sempit",
        "baris aktif tanpa satu pun token terbukti dari berkas yang DITUNJUK, meski tokennya ada di service",
    );
    cetak(
        "HIJAU TANPA ISI",
        "hijau-nama",
        "semua token terbukti, tapi tidak satu pun ada di ISI berkas bukti (isinya dibaca, tokennya tidak ditemukan di sana) — cuma nama berkas bukti yang mengandung token itu, atau tokennya path dan berkasnya memang ada",
    );

    let kuning: Vec<&BarisAudit> = baris
        .iter()
        .filter(|b| b.vonis == "kuning" || b.vonis == "kuning-hampa")
        .collect();
    println!();
    println!("KUNING — sebagian klaim terbukti, sebagian tidak: {} baris", kuning.len());
    for b in kuning.iter().take(30) {
        let ringkas: Vec<String> = b
            .gagal
            .iter()
            .map(|g| match &g.silang {
                Some((service_lain, n)) => {
                    format!("`{}` ({}, {} berkas di service `{}`)", g.token, g.kolom, n, service_lain)
                }
                None => format!("`{}` ({}, {} berkas)", g.token, g.kolom, g.dalam),
            })
            .collect();
        println!("  {}:{} [{}] {}/{} — {}", b.id, b.line, b.status, b.cocok, b.jumlah, ringkas.join(", "));
    }
    if kuning.len() > 30 {
        println!("  ... {} lainnya", kuning.len() - 30);
    }

    let silang: Vec<(&BarisAudit, &GagalToken, &String, usize)> = baris
        .iter()
        .flat_map(|b| b.gagal.iter().filter_map(move |g| g.silang.as_ref().map(|s| (b, g, &s.0, s.1))))
        .collect();
    println!();
    println!(
        "KLAIM LINTAS SERVICE — token tidak ada di service barisnya, tapi ada di service lain: {} klaim. Tidak bikin exit 1; yang perlu diperbaiki adalah kolom bukti barisnya.",
        silang.len()
    );
    for (b, g, service_lain, n) in silang.iter().take(25) {
        println!(
            "  {}:{} `{}` ({}) ketemu di service `{}` ({} berkas)",
            b.id, b.line, g.token, g.kolom, service_lain, n
        );
    }
    if silang.len() > 25 {
        println!("  ... {} lainnya", silang.len() - 25);
    }

    let tanpa: Vec<&BarisAudit> = baris.iter().filter(|b| b.tanpa_token).collect();
    let tidak_dituntut = hitung("tidak-diuji") - tanpa.len();
    println!();
    println!(
        "TIDAK TERUJI — ketiga kolom klaim murni prosa, tool tidak boleh mengaku memverifikasi: {} baris (+{} baris berstatus yang memang tidak dituntut cocok)",
        tanpa.len(), tidak_dituntut
    );
    for b in tanpa.iter().take(40) {
        println!("  {}:{} [{}]", b.id, b.line, b.status);
    }
    if tanpa.len() > 40 {
        println!("  ... {} lainnya", tanpa.len() - 40);
    }

    let tak_terbaca: Vec<&BarisAudit> = baris.iter().filter(|b| b.tanpa_baca).collect();
    println!();
    println!(
        "CATATAN — bukti tidak terbaca (hilang, folder, gagal didekode, atau lewat batas {}{}): {} baris",
        sebut_byte(batas_baca_bukti()),
        if batas_baca_bukti() == BATAS_BACA_BUKTI { "" } else { ", env PETA_BATAS_BACA_BUKTI" },
        tak_terbaca.len()
    );
    for b in tak_terbaca.iter().take(15) {
        println!("  {}:{} [{}] {}", b.id, b.line, b.status, b.vonis);
    }
    if tak_terbaca.len() > 15 {
        println!("  ... {} lainnya", tak_terbaca.len() - 15);
    }

    let luar_baris: Vec<&BarisAudit> = baris.iter().filter(|b| !b.luar.is_empty()).collect();
    println!();
    println!(
        "CATATAN — bukti menunjuk keluar folder service barisnya sendiri: {} baris (tidak mengubah vonis)",
        luar_baris.len()
    );
    for b in luar_baris.iter().take(15) {
        println!("  {} -> {}", b.id, b.luar.join(", "));
    }
    if luar_baris.len() > 15 {
        println!("  ... {} lainnya", luar_baris.len() - 15);
    }
    let terbatas = senarai_terbatas(&indeks_cache);
    if !terbatas.is_empty() {
        let (b_berkas, b_byte) = batas_indeks();
        println!(
            "CATATAN — penyapuan folder service terpotong untuk: {} (batas {} berkas / {}{}); angka sokongan untuk service itu adalah batas bawah",
            terbatas.join(", "),
            b_berkas,
            sebut_byte(b_byte),
            if b_berkas != INDEKS_BATAS_BERKAS || b_byte != INDEKS_BATAS_BYTE {
                ", env PETA_INDEKS_*"
            } else {
                ""
            }
        );
    }

    let (n_bolong, service_bolong) = senarai_bolong(&indeks_cache);
    if n_bolong > 0 {
        println!(
            "CATATAN — {} target tidak ikut tersapu di service: {} (folder gagal dibuka, berkas gagal dibaca atau bukan UTF-8). Klaim yang hanya hidup di sana terbaca HAMPA padahal yang bolong adalah penyapuannya, bukan barisnya.",
            n_bolong,
            service_bolong.join(", ")
        );
    }

    let diuji = baris.iter().filter(|b| !b.tanpa_token).count();
    println!();
    println!(
        "Ringkas: {} hijau-isi, {} hijau-nama, {} kuning, {} merah ({} hampa + {} sempit), {} tidak teruji dari {} baris.",
        hitung("hijau-isi"),
        hitung("hijau-nama"),
        hitung("kuning") + hitung("kuning-hampa"),
        merah,
        hampa_baris,
        sempit_baris,
        hitung("tidak-diuji"),
        baris.len()
    );
    println!(
        "Cakupan: {diuji} baris punya klaim teruji; {} baris berstatus `{aktif}` terbukti seluruh klaimnya dan minimal satu dari ISI berkas, {} seluruhnya hanya lewat nama berkas bukti atau keberadaan berkas; semua status bersama: {} hijau-isi + {} hijau-nama.",
        baris
            .iter()
            .filter(|b| b.vonis == "hijau-isi" && b.status == aktif)
            .count(),
        baris
            .iter()
            .filter(|b| b.vonis == "hijau-nama" && b.status == aktif)
            .count(),
        hitung("hijau-isi"),
        hitung("hijau-nama")
    );
    println!(
        "Arti exit 1: ada baris `{aktif}` dengan klaim yang tidak disebut berkas mana pun di service-nya (hampa) ATAU nol klaimnya terbukti dari berkas yang ditunjuk (sempit). Exit 1 TIDAK berarti semua fitur bekerja."
    );
    RingkasAudit {
        merah,
        kuning: hitung("kuning") + hitung("kuning-hampa"),
        tak_teruji: hitung("tidak-diuji"),
    }
}

/// Sebut batas byte seperti adanya. Dibagi 1e6 membuat 1.000.000 dan 1.500.000 sama-sama
/// terbaca "1 MB", dan nilai kecil terbaca "0 MB" — angka laporan yang tidak bisa dibedakan.
pub(crate) fn sebut_byte(byte: u64) -> String {
    if byte >= 1_000_000 {
        format!("{}.{:03} MB", byte / 1_000_000, (byte % 1_000_000) / 1_000)
    } else if byte >= 1_000 {
        format!("{} KB", byte / 1_000)
    } else {
        format!("{byte} byte")
    }
}

/// Token path hanya boleh menunjuk ke dalam repo. Path dengan drive (`C:/...`), UNC
/// (`\\server\share`) atau yang menyelinap keluar lewat `..` bukan bukti: tanpa larangan ini
/// baris yang mengklaim `C:/Windows/...` jadi hijau dan alat ini membaca di luar root-nya.
/// Yang dinilai adalah bentuk HASIL pemotongan awalan, bukan bentuk mentahnya — lihat
/// `potong_awalan`.
pub(crate) fn path_klaim_dalam_repo(token: &str) -> bool {
    // UNC ditolak pada bentuk mentahnya: pemotongan awalan akan mengubah `\\server\share\x`
    // menjadi `server\share\x`, dan itu bukan path lagi meski penggabungannya aman.
    if token.starts_with("\\\\") {
        return false;
    }
    let bersih = potong_awalan(token);
    if ada_drive(bersih.as_bytes()) || bersih.starts_with("\\\\") {
        return false;
    }
    !bersih.split(['/', '\\']).any(|b| b == "..")
}

/// Manifest kadang menulis path dengan awalan `/` seperti URL. Awalan itu dipotong sebelum
/// digabung ke root, dan pemotongan harus terjadi lebih dulu daripada pemeriksaan. Alasannya:
/// `/C:/Windows/win.ini` lolos penjaga drive karena byte pertamanya `/`, lalu berubah menjadi
/// `C:/Windows/win.ini` dan `Path::join` di Windows memaknainya absolut. Terukur: satu baris
/// `jalan` menjadi hijau-nama hanya karena berkas sistem itu ada, sementara `check` dan
/// `generate` keluar dengan exit 0.
pub(crate) fn potong_awalan(token: &str) -> &str {
    token.trim_start_matches(&['/', '\\'][..])
}

fn ada_drive(byte: &[u8]) -> bool {
    byte.len() > 1 && byte[0].is_ascii_alphabetic() && byte[1] == b':'
}

/// Token berbentuk path terbukti kalau berkasnya sungguh ada — dari root, lalu dari folder
/// service barisnya (baris milik service `svc-satu` boleh menulis `cmd/server/main.go`, yang
/// berarti `svc-satu/cmd/server/main.go`).
pub(crate) fn cocok_token_berkas(root: &Path, token: &str, folder: &str) -> bool {
    if !token.contains('/') || !path_klaim_dalam_repo(token) {
        return false;
    }
    let relatif = potong_awalan(token);
    if root.join(relatif).exists() {
        return true;
    }
    !folder.is_empty() && root.join(folder).join(relatif).exists()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::uji::*;
    #[test]
    fn token_uji_ambil_identitas_buang_prosa() {
        let token = token_uji("Layar SC12; tombol mulai; ConfigProvider; QRIS eksternal; Halaman lain aktif");
        assert!(token.contains(&"SC12".to_string()), "SC12 harus ikut: {token:?}");
        assert!(token.contains(&"ConfigProvider".to_string()));
        assert!(token.contains(&"QRIS".to_string()));
        assert!(!token.contains(&"Layar".to_string()), "prosa bermodal huruf besar awal tidak boleh jadi bukti: {token:?}");
        assert!(!token.contains(&"tombol".to_string()));
        assert!(!token.contains(&"aktif".to_string()));
        assert!(!token.contains(&"mulai".to_string()));
    }

    #[test]
    fn token_uji_menahan_route_path_dan_nama_berkas() {
        let token = token_uji("POST /api/v1/sessions; svc-muka/src/tampilan.tsx; session_id");
        assert!(token.contains(&"POST".to_string()));
        assert!(token.contains(&"/api/v1/sessions".to_string()));
        assert!(token.contains(&"svc-muka/src/tampilan.tsx".to_string()));
        assert!(token.contains(&"session_id".to_string()));
    }

    #[test]
    fn token_uji_membuang_duplikat() {
        let token = token_uji("ConfigProvider; ConfigProvider; ConfigProvider");
        assert_eq!(token, vec!["ConfigProvider".to_string()]);
    }

    #[test]
    fn vonis_utama_membedakan_terbukti_dari_isi_dan_dari_nama_berkas() {
        assert_eq!(vonis(Peran::Utama, 3, 3, 2, 0, true), "hijau-isi");
        // Semua cocok TAPI tidak satu pun lewat isi: tidak boleh disebut hijau-isi.
        assert_eq!(vonis(Peran::Utama, 3, 3, 0, 0, true), "hijau-nama");
        assert_eq!(vonis(Peran::Utama, 3, 2, 1, 0, true), "kuning");
        assert_eq!(vonis(Peran::Utama, 3, 0, 0, 0, true), "merah-sempit");
        assert_eq!(vonis(Peran::Utama, 3, 0, 0, 0, false), "merah-sempit");
    }

    #[test]
    fn vonis_klaim_hampa_menang_dari_cocok_sebagian() {
        // Satu klaim yang tidak disebut berkas mana pun di service ini membuat baris aktif jadi
        // merah, meski klaim lain terbukti. Ini yang membuat exit 1 seluas kalimatnya.
        assert_eq!(vonis(Peran::Utama, 3, 2, 2, 1, true), "merah-hampa");
        // Status separuh memang belum lengkap: hampa dicatat, tidak dipermalukan jadi merah.
        assert_eq!(vonis(Peran::Separuh, 3, 2, 2, 1, true), "kuning-hampa");
    }

    #[test]
    fn vonis_prosa_dan_status_non_aktif_tidak_diuji() {
        // Tool tidak boleh mengklaim memverifikasi baris yang tidak punya token identitas.
        assert_eq!(vonis(Peran::Utama, 0, 0, 0, 0, true), "tidak-diuji");
        assert_eq!(vonis(Peran::Mati, 4, 0, 0, 4, true), "tidak-diuji");
        assert_eq!(vonis(Peran::Belum, 4, 0, 0, 4, true), "tidak-diuji");
        assert_eq!(vonis(Peran::Separuh, 4, 4, 3, 0, true), "hijau-isi");
        // Sebagian berarti memang belum lengkap: tidak boleh jadi merah.
        assert_eq!(vonis(Peran::Separuh, 4, 0, 0, 0, true), "kuning");
    }

    #[test]
    fn rapikan_token_membuang_artefak_tanda_baca_di_ujung_rangkaian() {
        // Terukur di kolom prosa: 155 dari 226 kandidat adalah kata Indonesia berakhiran titik.
        assert_eq!(rapikan_token("pengguna."), "pengguna");
        assert_eq!(rapikan_token("CspStat.dll"), "CspStat.dll");
        assert_eq!(rapikan_token("/api/v1/sessions/"), "/api/v1/sessions/");
        // Setelah dirapikan, kata prosa biasa harus ditolak aturan token.
        assert!(!layak_token(&rapikan_token("pengguna.")));
        assert!(layak_token(&rapikan_token("CspStat.dll")));
    }

    #[test]
    fn klaim_baris_menggabungkan_tiga_kolom_dan_menyebut_asalnya() {
        let fitur = Feature {
            entry: "ConfigProvider".to_string(),
            summary: "Memakai SessionProvider untuk antrean".to_string(),
            how: "Menulis KioskPrefs lalu ConfigProvider".to_string(),
            ..fitur("a", "app", "aktif", "x")
        };
        let klaim = klaim_baris(&fitur);
        assert!(
            klaim.contains(&("titik_masuk".to_string(), "ConfigProvider".to_string())),
            "token titik_masuk harus ikut: {klaim:?}"
        );
        assert!(
            klaim.contains(&("ringkasan".to_string(), "SessionProvider".to_string())),
            "klaim di ringkasan tidak boleh tidak diperiksa: {klaim:?}"
        );
        assert!(
            klaim.contains(&("cara_kerja".to_string(), "KioskPrefs".to_string())),
            "klaim di cara_kerja tidak boleh tidak diperiksa: {klaim:?}"
        );
        // ConfigProvider ada di dua kolom: hanya dihitung sekali.
        assert_eq!(
            klaim.iter().filter(|(_, t)| t == "ConfigProvider").count(),
            1,
            "token yang muncul di dua kolom tidak boleh dihitung dua kali"
        );
    }

    #[test]
    fn klaim_lintas_service_bukan_klaim_hampa() {
        // Bentuk kasus yang sah di monorepo mana pun: baris satu service menyebut controller
        // service lain. Tokennya tidak ada di service barisnya, tapi ada di service lain -> tidak
        // boleh divonis tanpa sokongan.
        let mut semua: HashMap<String, Indeks> = HashMap::new();
        semua.insert(
            "svc-muka".to_string(),
            Indeks { isi: vec!["LAYAR05 halaman antre".to_string()], berkas: 1, terbatas: false, bolong: 0 },
        );
        semua.insert(
            "svc-inti".to_string(),
            Indeks {
                isi: vec!["class BayarV2Controller".to_string(), "routes".to_string()],
                berkas: 2,
                terbatas: false,
                bolong: 0,
            },
        );
        let lintas = sokong_lintas("BayarV2Controller", &semua, "svc-muka");
        assert_eq!(lintas, Some(("svc-inti".to_string(), 1)));
        // Token yang tidak ada di mana pun tetap None -> baris boleh disebut hampa.
        assert!(sokong_lintas("KlaimHantu9", &semua, "svc-muka").is_none());
        // Service sendiri tidak dihitung sebagai lintas.
        assert!(sokong_lintas("LAYAR05", &semua, "svc-muka").is_none());
    }

    #[test]
    fn bukti_keluar_service_dibanding_per_komponen_bukan_per_teks() {
        let bukti = |daftar: &[&str]| -> Vec<String> {
            daftar.iter().map(|s| s.to_string()).collect()
        };
        assert!(
            bukti_keluar_service(&bukti(&["svc-inti/cmd/x.go"]), "svc-inti").is_empty(),
            "bukti di folder sendiri tidak boleh dilaporkan keluar"
        );
        assert_eq!(
            bukti_keluar_service(&bukti(&["svc-muka/a.php"]), "svc-inti"),
            vec!["svc-muka/a.php".to_string()],
            "bukti ke service lain wajib dilaporkan"
        );
        // Inti temuan: awalan teks membuat nama yang hampir sama disangka folder sendiri.
        assert_eq!(
            bukti_keluar_service(&bukti(&["svc-intiLain/x.go"]), "svc-inti"),
            vec!["svc-intiLain/x.go".to_string()],
            "`svc-intiLain` bukan anak `svc-inti`"
        );
        assert_eq!(
            bukti_keluar_service(&bukti(&["svc/a.go", "svc2/b.go"]), "svc/"),
            vec!["svc2/b.go".to_string()],
            "folder dengan garis miring di ujung tetap sebanding per komponen"
        );
        // Baris yang service-nya tidak punya folder: tidak ada yang bisa dinyatakan keluar.
        assert!(
            bukti_keluar_service(&bukti(&["apa/pun.go"]), "").is_empty(),
            "tanpa folder service tidak ada yang bisa dinyatakan keluar"
        );
    }

    #[test]
    fn indeks_service_mencatat_target_yang_gagal_didekode() {
        // Berkas berekstensi indeks yang bukan UTF-8 selama ini dilewati tanpa jejak, jadi
        // klaim yang cuma hidup di dalamnya dituduh HAMPA (exit 1) seolah barisnya mengarang.
        // Terukur di repo ini: 13 dari 10.491 berkas, salah satunya `blinking_cursor_field.dart`.
        let akar = std::env::temp_dir().join(format!("fv-bolong-{}", std::process::id()));
        let _ = fs::remove_dir_all(&akar);
        fs::create_dir_all(akar.join("svc/src")).expect("buat folder uji");
        fs::write(akar.join("svc/src/bagus.go"), "package svc\nfunc Halo() {}\n").expect("tulis bagus");
        fs::write(akar.join("svc/src/cacat.go"), [0xFFu8, 0xFE, 0x01, b'H', b'a', b'l', b'o'])
            .expect("tulis cacat UTF-8");

        let indeks = indeks_service(&akar, "svc");
        assert_eq!(indeks.berkas, 1, "hanya berkas yang terbaca yang dihitung: {indeks:?}");
        assert_eq!(indeks.bolong, 1, "yang gagal didekode wajib tercatat, bukan disenyapkan");
        assert!(!indeks.terbatas, "gagal dekode bukan pemotongan batas");
        assert_eq!(sokong_token("Halo", &indeks), 1, "Halo hanya dari berkas yang terbaca");
        let _ = fs::remove_dir_all(&akar);
    }

    #[test]
    fn senarai_bolong_menjumlah_dan_menyortir_service() {
        for putaran in 0..4 {
            let (pertama, kedua) = if putaran % 2 == 0 {
                ("svc-zzz", "svc-aaa")
            } else {
                ("svc-aaa", "svc-zzz")
            };
            let bolong = |nama: &str, n: usize| {
                (
                    nama.to_string(),
                    Indeks { isi: Vec::new(), berkas: 0, terbatas: false, bolong: n },
                )
            };
            let semua: HashMap<String, Indeks> = vec![
                bolong(pertama, 3),
                bolong(kedua, 2),
                bolong("svc-nol", 0),
            ]
            .into_iter()
            .collect();
            assert_eq!(
                senarai_bolong(&semua),
                (5, vec!["svc-aaa".to_string(), "svc-zzz".to_string()]),
                "putaran {putaran} (sisip {pertama} lalu {kedua}) memberi laporan lain"
            );
        }
    }

    #[test]
    fn indeks_service_membedakan_bukti_sempit_dari_klaim_hampa() {
        let akar = std::env::temp_dir().join(format!("fv-indeks-{}", std::process::id()));
        fs::create_dir_all(akar.join("svc/src")).expect("buat folder uji");
        fs::write(akar.join("svc/src/a.go"), "package svc\nfunc Halo() {}\n").expect("tulis a");
        fs::write(akar.join("svc/src/b.go"), "package svc\n// Halo disebut juga di sini\n").expect("tulis b");
        fs::create_dir_all(akar.join("svc/node_modules")).expect("folder lewati");
        fs::write(akar.join("svc/node_modules/x.go"), "Halo Halo Halo").expect("tulis skip");

        let indeks = indeks_service(&akar, "svc");
        assert_eq!(indeks.berkas, 2, "node_modules wajib dilewati: {indeks:?}");
        assert_eq!(sokong_token("Halo", &indeks), 2, "dua berkas menyebut Halo");
        assert_eq!(sokong_token("TidakAdaDisini", &indeks), 0);
        assert!(!indeks.terbatas, "sapuan kecil tidak boleh dianggap terpotong");
        assert_eq!(indeks.bolong, 0, "sapuan yang berhasil tidak boleh menyumbang bolong");
        let _ = fs::remove_dir_all(&akar);
    }

    #[test]
    fn cocok_token_berkas_cek_langsung_lalu_lewat_folder_service() {        let root = Path::new(".");
        assert!(cocok_token_berkas(root, "src/main.rs", ""), "path repo-relatif harus terbukti langsung");
        // Kata biasa tidak diuji sebagai berkas: "ConfigProvider" bukan path.
        assert!(!cocok_token_berkas(root, "ConfigProvider", ""));
        assert!(!cocok_token_berkas(root, "src/tidak-ada.rs", ""));

        let hidup = std::env::temp_dir().join(format!("fv-audit-folder-{}", std::process::id()));
        let dalam = hidup.join("cmd").join("setup");
        fs::create_dir_all(&dalam).expect("buat struktur uji");
        fs::write(dalam.join("app.go"), "package main").expect("tulis berkas uji");
        // Dari root repo token ini tidak ada; dari folder service `cmd` ia ada.
        assert!(!cocok_token_berkas(&hidup, "setup/app.go", ""), "tidak boleh lolos tanpa folder");
        assert!(cocok_token_berkas(&hidup, "setup/app.go", "cmd"), "folder service ikut dicoba");
        let _ = fs::remove_dir_all(&hidup);
    }

    #[test]
    fn cocok_token_berkas_menolak_path_di_luar_root() {
        let root = Path::new(".");
        // Berkas ini sungguh ada di mesin Windows, jadi tanpa penolakan ini satu klaim
        // `C:/Windows/...` cukup membuat baris mana pun jadi hijau.
        assert!(
            !cocok_token_berkas(root, "C:/Windows/System32/drivers/etc/hosts", ""),
            "path dengan drive letter bukan bukti di dalam repo"
        );
        assert!(
            !cocok_token_berkas(root, "src/../../Windows/System32/drivers/etc/hosts", ""),
            "selundupan keluar lewat .. bukan bukti"
        );
        assert!(
            !cocok_token_berkas(root, "\\\\server\\share\\sapa.go", ""),
            "path UNC bukan bukti"
        );
        assert!(cocok_token_berkas(root, "src/main.rs", ""), "relatif biasa tetap terbukti");
    }

    #[test]
    fn cocok_token_berkas_membaca_awalan_garis_miring_dari_root() {
        // Manifest kadang menulis `/src/main.rs` seperti URL. `root.join("/x")` di Rust
        // memaknainya absolut, jadi berkas di repo tidak pernah ditemukan dan barisnya
        // jadi merah palsu.
        assert!(
            cocok_token_berkas(Path::new("."), "/src/main.rs", ""),
            "awalan garis miring berarti path dari root repo"
        );
    }

    #[test]
    fn awalan_garis_miring_tidak_menyelundupkan_drive_letter() {
        // Penjaga dulu menilai bentuk MENTAH dan `join` memakai bentuk HASIL POTONGAN, jadi
        // `/C:/Windows/win.ini` lolos (byte pertamanya `/`) lalu terbukti ada karena
        // `Path::join` di Windows memaknainya absolut. Terukur pada dataset contoh: baris
        // `jalan` menjadi hijau-nama dari berkas sistem itu sementara `check` exit 0.
        for bentuk in [
            "/C:/Windows/win.ini",
            "//C:/Windows/win.ini",
            "\\C:/Windows/win.ini",
            "/c:/windows/win.ini",
        ] {
            assert!(
                !path_klaim_dalam_repo(bentuk),
                "drive letter di balik awalan tetap bukan bukti dalam repo: {bentuk}"
            );
            assert!(
                !cocok_token_berkas(Path::new("."), bentuk, ""),
                "dan tidak boleh membuktikan apa pun: {bentuk}"
            );
        }
        // Sisi lain dari pemotongan yang sama: path repo berawalan `/` harus tetap terbukti.
        assert!(path_klaim_dalam_repo("/src/main.rs"));
        assert!(cocok_token_berkas(Path::new("."), "/src/main.rs", "src"));
    }

    #[test]
    fn sokong_lintas_memilih_key_terurut_saat_seri() {
        let isi = |teks: &str| Indeks {
            isi: vec![teks.to_string()],
            berkas: 1,
            terbatas: false,
            bolong: 0,
        };
        // Dua service sama-sama mendukung satu token: seri harus dimenangkan key pertama
        // secara alfabet. Peta dibangun ULANG tiap putaran dengan urutan sisip dibalik,
        // karena benih HashMap acak per peta — tanpa penyortiran, jawaban berubah antar
        // pembacaan dan laporan tidak bisa direproduksi.
        for putaran in 0..12 {
            let (pertama, kedua) = if putaran % 2 == 0 {
                ("svc-b", "svc-a")
            } else {
                ("svc-a", "svc-b")
            };
            let mut semua = HashMap::new();
            semua.insert(pertama.to_string(), isi("Token9 disebut di sini"));
            semua.insert(kedua.to_string(), isi("Token9 disebut di sini juga"));
            assert_eq!(
                sokong_lintas("Token9", &semua, "svc-x").map(|(k, _)| k),
                Some("svc-a".to_string()),
                "putaran {putaran} (sisip {pertama} lalu {kedua}) memberi jawaban lain dari urutan key"
            );
        }
    }

    #[test]
    fn senarai_terbatas_menyebut_service_terpotong_secara_terurut() {
        // Peta dibangun ULANG tiap putaran dengan urutan sisip dibalik, karena benih HashMap
        // acak per peta: tanpa penyortiran, satu dataset bisa tercetak dua urutan berbeda.
        for putaran in 0..12 {
            let (pertama, kedua) = if putaran % 2 == 0 {
                ("svc-zulu", "svc-alpha")
            } else {
                ("svc-alpha", "svc-zulu")
            };
            let potong = |nama: &str| (nama.to_string(), Indeks { isi: Vec::new(), berkas: 0, terbatas: true, bolong: 0 });
            let utuh = |nama: &str| (nama.to_string(), Indeks { isi: Vec::new(), berkas: 0, terbatas: false, bolong: 0 });
            let semua: HashMap<String, Indeks> = vec![
                potong(pertama),
                potong(kedua),
                utuh("svc-utuh"),
                potong("svc-mid"),
            ]
            .into_iter()
            .collect();
            assert_eq!(
                senarai_terbatas(&semua),
                vec!["svc-alpha".to_string(), "svc-mid".to_string(), "svc-zulu".to_string()],
                "putaran {putaran} (sisip {pertama} lalu {kedua}) memberi urutan lain: {}",
                senarai_terbatas(&semua).join(", ")
            );
        }
    }

    #[test]
    fn sebut_byte_memakai_satuan_yang_membedakan_nilai() {
        assert_eq!(sebut_byte(900), "900 byte");
        assert_eq!(sebut_byte(1_000), "1 KB");
        assert_eq!(sebut_byte(1_000_000), "1.000 MB");
        assert_eq!(sebut_byte(1_500_000), "1.500 MB");
        assert_ne!(
            sebut_byte(1_000_000),
            sebut_byte(1_500_000),
            "dua batas berbeda harus terbaca berbeda, bukan sama-sama `1 MB`"
        );
    }

}
