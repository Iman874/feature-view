//! Membaca dan mengurai manifest TSV menjadi Model, plus cap isi untuk membuktikan
//! halaman masih sinkron. Semua pesan galat parse lahir di sini.

use std::collections::HashMap;
use std::fmt::Write as _;
use std::fs;
use std::path::Path;
use crate::data::*;
use crate::aturan::*;

pub(crate) const CONFIG_FILE: &str = "docs/view/config.tsv";

pub(crate) const SERVICES_FILE: &str = "docs/view/services.tsv";

pub(crate) const PRODUCTS_FILE: &str = "docs/view/produk.tsv";

pub(crate) const APPS_FILE: &str = "docs/view/applications.tsv";

pub(crate) const MAPPING_FILE: &str = "docs/view/pemetaan.tsv";

pub(crate) const FEATURES_FILE: &str = "docs/view/features.tsv";

pub(crate) const EDGES_FILE: &str = "docs/view/edges.tsv";
/// Opsional: repo yang belum memakai papan pekerjaan tidak punya berkas ini, dan hasilnya harus
/// tetap sama seperti sebelum papan ada.
pub(crate) const TASKS_FILE: &str = "docs/view/tugas.tsv";

pub(crate) const OUTPUT_FILE: &str = "docs/view/index.html";

/// Urutan tetap: sidik jari manifest dihitung dari file-file ini dalam urutan ini.
/// `TASKS_FILE` ikut meski opsional: begitu sebuah repo memakainya, isinya wajib bergerak
/// bersama cap halaman.
pub(crate) const MANIFEST_FILES: [&str; 8] = [
    CONFIG_FILE,
    SERVICES_FILE,
    PRODUCTS_FILE,
    APPS_FILE,
    MAPPING_FILE,
    FEATURES_FILE,
    EDGES_FILE,
    TASKS_FILE,
];

/// Kunci wajib di config.tsv. Nilai kosong = galat, bukan default diam-diam.
pub(crate) const CONFIG_KEYS: [&str; 10] = [
    "merek",
    "judul",
    "bahasa",
    "status",
    "status_peran_utama",
    "status_peran_separuh",
    "status_peran_mati",
    "jenis_relasi",
    "aktor_ikon",
    "minimum_fitur_per_service",
];

pub(crate) fn muat_model(root: &Path) -> Result<Model, LoadError> {
    let config_path = root.join(CONFIG_FILE);
    let isi_config = fs::read_to_string(&config_path)
        .map_err(|e| LoadError::Io(format!("GAGAL membaca {}: {e}", config_path.display())))?;
    let config = parse_config(&isi_config, CONFIG_FILE).map_err(LoadError::Manifest)?;

    let layanan_path = root.join(SERVICES_FILE);
    let produk_path = root.join(PRODUCTS_FILE);
    let aplikasi_path = root.join(APPS_FILE);
    let pemetaan_path = root.join(MAPPING_FILE);
    let fitur_path = root.join(FEATURES_FILE);
    let edge_path = root.join(EDGES_FILE);
    let mut galat = Vec::new();

    let isi_layanan = fs::read_to_string(&layanan_path).map_err(|e| {
        LoadError::Io(format!("GAGAL membaca {}: {e}", layanan_path.display()))
    })?;
    let isi_produk = fs::read_to_string(&produk_path).map_err(|e| {
        LoadError::Io(format!("GAGAL membaca {}: {e}", produk_path.display()))
    })?;
    let isi_aplikasi = fs::read_to_string(&aplikasi_path).map_err(|e| {
        LoadError::Io(format!("GAGAL membaca {}: {e}", aplikasi_path.display()))
    })?;
    let isi_pemetaan = fs::read_to_string(&pemetaan_path).map_err(|e| {
        LoadError::Io(format!("GAGAL membaca {}: {e}", pemetaan_path.display()))
    })?;
    let isi_fitur = fs::read_to_string(&fitur_path).map_err(|e| {
        LoadError::Io(format!("GAGAL membaca {}: {e}", fitur_path.display()))
    })?;
    let isi_edge = fs::read_to_string(&edge_path).map_err(|e| {
        LoadError::Io(format!("GAGAL membaca {}: {e}", edge_path.display()))
    })?;

    let services = match parse_services(&isi_layanan, SERVICES_FILE) {
        Ok(v) => v,
        Err(mut e) => {
            galat.append(&mut e);
            Vec::new()
        }
    };
    let products = match parse_products(&isi_produk, PRODUCTS_FILE) {
        Ok(v) => v,
        Err(mut e) => {
            galat.append(&mut e);
            Vec::new()
        }
    };
    let apps = match parse_apps(&isi_aplikasi, APPS_FILE) {
        Ok(v) => v,
        Err(mut e) => {
            galat.append(&mut e);
            Vec::new()
        }
    };
    let mappings = match parse_mappings(&isi_pemetaan, MAPPING_FILE) {
        Ok(v) => v,
        Err(mut e) => {
            galat.append(&mut e);
            Vec::new()
        }
    };
    let mut features = match parse_features(&isi_fitur, FEATURES_FILE) {
        Ok(v) => v,
        Err(mut e) => {
            galat.append(&mut e);
            Vec::new()
        }
    };
    let edges = match parse_edges(&isi_edge, EDGES_FILE) {
        Ok(v) => v,
        Err(mut e) => {
            galat.append(&mut e);
            Vec::new()
        }
    };

    // Papan pekerjaan opsional dengan sengaja: repo yang belum memakainya tidak boleh tiba-tiba
    // gagal `check` hanya karena sebuah berkas tidak ada. Kalau berkasnya ADA lalu rusak, itu
    // galat nyata dan ia masuk `galat` seperti manifest lain.
    let tugas = match fs::read_to_string(root.join(TASKS_FILE)) {
        Ok(isi) => match parse_tugas(&isi, TASKS_FILE) {
            Ok(v) => v,
            Err(mut e) => {
                galat.append(&mut e);
                Vec::new()
            }
        },
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Vec::new(),
        Err(e) => return Err(LoadError::Io(format!("GAGAL membaca {}: {e}", TASKS_FILE))),
    };

    if !galat.is_empty() {
        return Err(LoadError::Manifest(galat));
    }

    let mut peta = HashMap::new();
    for mapping in &mappings {
        peta.insert(
            mapping.subfeature.clone(),
            (mapping.product.clone(), mapping.app.clone()),
        );
    }
    for fitur in &mut features {
        if let Some((produk, aplikasi)) = peta.get(&fitur.id) {
            fitur.product = produk.clone();
            fitur.app = aplikasi.clone();
        }
    }

    let model = Model {
        config,
        services,
        products,
        apps,
        mappings,
        features,
        edges,
        tugas,
    };

    galat = validate(&model, root);
    if galat.is_empty() {
        Ok(model)
    } else {
        Err(LoadError::Manifest(galat))
    }
}

/// Baca `config.tsv`: pasangan `kunci|nilai`. Kunci wajib ada dan nilainya tidak kosong; daftar
/// dipisah koma. Kosakata status/relasi dan pemetaan aktor ke bentuk ikon tidak lagi ada di kode.
pub(crate) fn parse_config(isi: &str, label: &str) -> Result<Config, Vec<String>> {
    let mut galat = Vec::new();
    let mut nilai: HashMap<String, String> = HashMap::new();

    for (indeks, baris) in isi.lines().enumerate() {
        let nomor = indeks + 1;
        let bersih = baris.trim();
        if bersih.is_empty() || bersih.starts_with('#') {
            continue;
        }
        let field: Vec<&str> = bersih.split('|').map(str::trim).collect();
        if field.len() != 2 {
            galat.push(format!(
                "{label}:{nomor}: butuh 2 field (kunci|nilai), dapat {}",
                field.len()
            ));
            continue;
        }
        if field[0].is_empty() || field[1].is_empty() {
            galat.push(format!("{label}:{nomor}: kunci dan nilai wajib terisi"));
            continue;
        }
        if nilai.contains_key(field[0]) {
            galat.push(format!("{label}:{nomor}: kunci duplikat: {}", field[0]));
            continue;
        }
        nilai.insert(field[0].to_string(), field[1].to_string());
    }

    for kunci in CONFIG_KEYS {
        if !nilai.contains_key(kunci) {
            galat.push(format!("{label}: kunci wajib hilang: {kunci}"));
        }
    }
    if !galat.is_empty() {
        return Err(galat);
    }

    let ambil = |k: &str| nilai.get(k).cloned().unwrap_or_default();
    let statuses = split_daftar(&ambil("status"));
    let kinds = split_daftar(&ambil("jenis_relasi"));
    if statuses.is_empty() {
        galat.push(format!("{label}: `status` harus berisi minimal satu nama status"));
    }
    if kinds.is_empty() {
        galat.push(format!("{label}: `jenis_relasi` harus berisi minimal satu nama jenis"));
    }

    let mut peran = Vec::new();
    for (nama, kunci) in [
        ("utama", "status_peran_utama"),
        ("separuh", "status_peran_separuh"),
        ("mati", "status_peran_mati"),
    ] {
        let nilai_peran = ambil(kunci);
        if !statuses.iter().any(|s| s == &nilai_peran) {
            galat.push(format!(
                "{label}: {kunci} `{nilai_peran}` bukan anggota daftar `status` ({})",
                statuses.join(",")
            ));
        }
        peran.push((nama, nilai_peran));
    }

    let minimum_fitur: usize = ambil("minimum_fitur_per_service").parse().unwrap_or(0);
    if minimum_fitur < 1 {
        galat.push(format!(
            "{label}: `minimum_fitur_per_service` harus angka >= 1, dapat {}",
            ambil("minimum_fitur_per_service")
        ));
    }

    let mut aktor_ikon: Vec<(String, String)> = Vec::new();
    for pasangan in split_daftar(&ambil("aktor_ikon")) {
        match pasangan.split_once(':') {
            Some((aktor, ikon)) if !aktor.is_empty() && !ikon.is_empty() => {
                aktor_ikon.push((aktor.to_string(), ikon.to_string()))
            }
            _ => galat.push(format!(
                "{label}: `aktor_ikon` butuh bentuk aktor:ikon, dapat `{pasangan}`"
            )),
        }
    }

    // Kosakata papan pekerjaan. Sengaja TIDAK masuk CONFIG_KEYS: repo yang belum memakai papan
    // tidak boleh gagal hanya karena kunci ini hilang. Daftar kosong berarti "belum dipakai", dan
    // `validate` yang menuntut saat `tugas.tsv` benar-benar berisi.
    let status_tugas = split_daftar(&ambil("status_tugas"));

    if !galat.is_empty() {
        return Err(galat);
    }

    let peran_kontrak: [(&'static str, String); 3] = [
        peran[0].clone(),
        peran[1].clone(),
        peran[2].clone(),
    ];
    Ok(Config {
        merek: ambil("merek"),
        judul: ambil("judul"),
        bahasa: ambil("bahasa"),
        statuses,
        kinds,
        aktor_ikon,
        minimum_fitur,
        peran: peran_kontrak,
        status_tugas,
        judul_papan: ambil("judul_papan"),
    })
}

pub(crate) fn parse_services(isi: &str, label: &str) -> Result<Vec<Service>, Vec<String>> {
    let mut hasil = Vec::new();
    let mut galat = Vec::new();

    for (indeks, baris) in isi.lines().enumerate() {
        let nomor = indeks + 1;
        let bersih = baris.trim();
        if bersih.is_empty() || bersih.starts_with('#') {
            continue;
        }
        let field: Vec<&str> = bersih.split('|').map(str::trim).collect();
        if field.len() != 7 {
            galat.push(format!(
                "{label}:{nomor}: butuh 7 field (key|nama|stack|deskripsi|folder|dokumen|ikon), dapat {}",
                field.len()
            ));
            continue;
        }
        if field.iter().take(5).any(|f| f.is_empty()) {
            galat.push(format!("{label}:{nomor}: field key sampai folder wajib terisi"));
            continue;
        }
        if field[6].is_empty() {
            galat.push(format!(
                "{label}:{nomor}: service {} wajib menyebut nama bentuk ikon",
                field[0]
            ));
            continue;
        }
        hasil.push(Service {
            key: field[0].to_string(),
            name: field[1].to_string(),
            stack: field[2].to_string(),
            description: field[3].to_string(),
            folder: field[4].to_string(),
            docs: split_daftar(field[5]),
            ikon: field[6].to_string(),
        });
    }

    if galat.is_empty() && hasil.is_empty() {
        galat.push(format!("{label}: tidak ada service yang terbaca"));
    }
    if galat.is_empty() {
        Ok(hasil)
    } else {
        Err(galat)
    }
}

pub(crate) fn parse_products(isi: &str, label: &str) -> Result<Vec<Product>, Vec<String>> {
    let mut hasil = Vec::new();
    let mut galat = Vec::new();

    for (indeks, baris) in isi.lines().enumerate() {
        let nomor = indeks + 1;
        let bersih = baris.trim();
        if bersih.is_empty() || bersih.starts_with('#') {
            continue;
        }
        let field: Vec<&str> = bersih.split('|').map(str::trim).collect();
        if field.len() != 7 {
            galat.push(format!(
                "{label}:{nomor}: butuh 7 field (id|nama|aktor|hasil|ringkasan|dokumen|catatan), dapat {}",
                field.len()
            ));
            continue;
        }
        if field.iter().take(5).any(|f| f.is_empty()) {
            galat.push(format!("{label}:{nomor}: field id sampai ringkasan wajib terisi"));
            continue;
        }
        hasil.push(Product {
            id: field[0].to_string(),
            name: field[1].to_string(),
            actor: field[2].to_string(),
            outcome: field[3].to_string(),
            summary: field[4].to_string(),
            docs: split_daftar(field[5]),
            notes: if field[6] == "-" {
                String::new()
            } else {
                field[6].to_string()
            },
            line: nomor,
        });
    }

    if galat.is_empty() && hasil.is_empty() {
        galat.push(format!("{label}: tidak ada produk yang terbaca"));
    }
    if galat.is_empty() {
        Ok(hasil)
    } else {
        Err(galat)
    }
}

pub(crate) fn parse_apps(isi: &str, label: &str) -> Result<Vec<App>, Vec<String>> {
    let mut hasil = Vec::new();
    let mut galat = Vec::new();

    for (indeks, baris) in isi.lines().enumerate() {
        let nomor = indeks + 1;
        let bersih = baris.trim();
        if bersih.is_empty() || bersih.starts_with('#') {
            continue;
        }
        let field: Vec<&str> = bersih.split('|').map(str::trim).collect();
        if field.len() != 8 {
            galat.push(format!(
                "{label}:{nomor}: butuh 8 field (id|service|nama|jenis|folder|deskripsi|dokumen|ikon), dapat {}",
                field.len()
            ));
            continue;
        }
        if field.iter().take(6).any(|f| f.is_empty()) {
            galat.push(format!(
                "{label}:{nomor}: field id sampai deskripsi wajib terisi"
            ));
            continue;
        }
        if field[7].is_empty() {
            galat.push(format!(
                "{label}:{nomor}: aplikasi {} wajib menyebut nama bentuk ikon",
                field[0]
            ));
            continue;
        }
        hasil.push(App {
            id: field[0].to_string(),
            service: field[1].to_string(),
            name: field[2].to_string(),
            kind: field[3].to_string(),
            folder: field[4].to_string(),
            description: field[5].to_string(),
            docs: split_daftar(field[6]),
            ikon: field[7].to_string(),
            line: nomor,
        });
    }

    if galat.is_empty() && hasil.is_empty() {
        galat.push(format!("{label}: tidak ada aplikasi yang terbaca"));
    }
    if galat.is_empty() {
        Ok(hasil)
    } else {
        Err(galat)
    }
}

pub(crate) fn parse_mappings(isi: &str, label: &str) -> Result<Vec<Mapping>, Vec<String>> {
    let mut hasil = Vec::new();
    let mut galat = Vec::new();

    for (indeks, baris) in isi.lines().enumerate() {
        let nomor = indeks + 1;
        let bersih = baris.trim();
        if bersih.is_empty() || bersih.starts_with('#') {
            continue;
        }
        let field: Vec<&str> = bersih.split('|').map(str::trim).collect();
        if field.len() != 3 {
            galat.push(format!(
                "{label}:{nomor}: butuh 3 field (subfitur|produk|aplikasi), dapat {}",
                field.len()
            ));
            continue;
        }
        if field.iter().any(|f| f.is_empty()) {
            galat.push(format!("{label}:{nomor}: semua field pemetaan wajib terisi"));
            continue;
        }
        hasil.push(Mapping {
            subfeature: field[0].to_string(),
            product: field[1].to_string(),
            app: field[2].to_string(),
            line: nomor,
        });
    }

    if galat.is_empty() && hasil.is_empty() {
        galat.push(format!("{label}: tidak ada pemetaan yang terbaca"));
    }
    if galat.is_empty() {
        Ok(hasil)
    } else {
        Err(galat)
    }
}

pub(crate) fn parse_features(isi: &str, label: &str) -> Result<Vec<Feature>, Vec<String>> {
    let mut hasil = Vec::new();
    let mut galat = Vec::new();

    for (indeks, baris) in isi.lines().enumerate() {
        let nomor = indeks + 1;
        let bersih = baris.trim();
        if bersih.is_empty() || bersih.starts_with('#') {
            continue;
        }
        let field: Vec<&str> = bersih.split('|').map(str::trim).collect();
        if field.len() != 10 {
            galat.push(format!(
                "{label}:{nomor}: butuh 10 field (id|service|fitur|status|ringkasan|cara_kerja|titik_masuk|bukti|dokumen|catatan), dapat {}",
                field.len()
            ));
            continue;
        }
        if field.iter().take(7).any(|f| f.is_empty()) {
            galat.push(format!(
                "{label}:{nomor}: field id sampai titik_masuk wajib terisi"
            ));
            continue;
        }
        let evidence = split_daftar(field[7]);
        if evidence.is_empty() {
            galat.push(format!("{label}:{nomor}: bukti wajib minimal satu path"));
            continue;
        }
        hasil.push(Feature {
            id: field[0].to_string(),
            product: String::new(),
            app: String::new(),
            service: field[1].to_string(),
            name: field[2].to_string(),
            status: field[3].to_string(),
            summary: field[4].to_string(),
            how: field[5].to_string(),
            entry: field[6].to_string(),
            evidence,
            docs: split_daftar(field[8]),
            notes: if field[9] == "-" {
                String::new()
            } else {
                field[9].to_string()
            },
            line: nomor,
        });
    }

    if galat.is_empty() && hasil.is_empty() {
        galat.push(format!("{label}: tidak ada fitur yang terbaca"));
    }
    if galat.is_empty() {
        Ok(hasil)
    } else {
        Err(galat)
    }
}

pub(crate) fn parse_edges(isi: &str, label: &str) -> Result<Vec<Edge>, Vec<String>> {
    let mut hasil = Vec::new();
    let mut galat = Vec::new();

    for (indeks, baris) in isi.lines().enumerate() {
        let nomor = indeks + 1;
        let bersih = baris.trim();
        if bersih.is_empty() || bersih.starts_with('#') {
            continue;
        }
        let field: Vec<&str> = bersih.split('|').map(str::trim).collect();
        if field.len() != 4 {
            galat.push(format!(
                "{label}:{nomor}: butuh 4 field (dari|ke|jenis|label), dapat {}",
                field.len()
            ));
            continue;
        }
        if field.iter().any(|f| f.is_empty()) {
            galat.push(format!("{label}:{nomor}: semua field edge wajib terisi"));
            continue;
        }
        hasil.push(Edge {
            from: field[0].to_string(),
            to: field[1].to_string(),
            kind: field[2].to_string(),
            label: field[3].to_string(),
            line: nomor,
        });
    }

    if galat.is_empty() && hasil.is_empty() {
        galat.push(format!("{label}: tidak ada relasi yang terbaca"));
    }
    if galat.is_empty() {
        Ok(hasil)
    } else {
        Err(galat)
    }
}

/// Baca `tugas.tsv` — `id|subfitur|judul|status|progres|pemegang|berkas|catatan`.
/// Hanya kolom terakhir yang boleh kosong. `progres` ditolak kalau bukan bilangan 0..=100:
/// angka yang tampil di layar harus bisa dipertanggungjawabkan, jadi nilai yang tidak terbaca
/// tidak pernah diam-diam jadi nol. Berkas tanpa baris adalah keadaan sah (papan belum dipakai),
/// tidak seperti `edges.tsv` yang menuntut isi.
pub(crate) fn parse_tugas(isi: &str, label: &str) -> Result<Vec<Tugas>, Vec<String>> {
    let mut hasil = Vec::new();
    let mut galat = Vec::new();

    for (indeks, baris) in isi.lines().enumerate() {
        let nomor = indeks + 1;
        let bersih = baris.trim();
        if bersih.is_empty() || bersih.starts_with('#') {
            continue;
        }
        let field: Vec<&str> = bersih.split('|').map(str::trim).collect();
        if field.len() != 8 {
            galat.push(format!(
                "{label}:{nomor}: butuh 8 field (id|subfitur|judul|status|progres|pemegang|berkas|catatan), dapat {}",
                field.len()
            ));
            continue;
        }
        if field[..7].iter().any(|f| f.is_empty()) {
            galat.push(format!(
                "{label}:{nomor}: semua field kecuali `catatan` wajib terisi"
            ));
            continue;
        }
        if !id_aman(field[0]) {
            galat.push(format!("{label}:{nomor}: id tugas tidak sah: {}", field[0]));
            continue;
        }
        let progres: u8 = match field[4].parse() {
            Ok(v @ 0..=100) => v,
            _ => {
                galat.push(format!(
                    "{label}:{nomor}: `progres` harus bilangan 0..=100, dapat `{}`",
                    field[4]
                ));
                continue;
            }
        };
        hasil.push(Tugas {
            id: field[0].to_string(),
            subfitur: field[1].to_string(),
            judul: field[2].to_string(),
            status: field[3].to_string(),
            progres,
            pemegang: field[5].to_string(),
            berkas: field[6].to_string(),
            notes: field[7].to_string(),
            line: nomor,
        });
    }

    if galat.is_empty() {
        Ok(hasil)
    } else {
        Err(galat)
    }
}

pub(crate) fn split_daftar(nilai: &str) -> Vec<String> {
    nilai
        .split(',')
        .map(str::trim)
        .filter(|p| !p.is_empty() && *p != "-")
        .map(ToString::to_string)
        .collect()
}

/// Sidik jari isi manifest TSV (FNV-1a 64 bit). Bukan checksum keamanan: hanya pemicu
/// "manifest berubah" bagi halaman yang sedang bersaji, dan penanda provenance di HTML.
pub(crate) fn cap_manifest(root: &Path) -> Result<String, String> {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for nama in MANIFEST_FILES {
        let path = root.join(nama);
        let isi = match fs::read(&path) {
            Ok(v) => v,
            // Satu-satunya manifest yang boleh tidak ada. Yang lain tetap gagal seperti sebelumnya,
            // supaya cap tidak pernah dihitung dari dataset yang bolong tanpa ada yang tahu.
            Err(_) if nama == TASKS_FILE => Vec::new(),
            Err(e) => return Err(format!("GAGAL membaca {}: {e}", path.display())),
        };
        for byte in &isi {
            hash ^= *byte as u64;
            hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
        }
        // Pemisah: tanpa ini ("ab" + "c") dan ("a" + "bc") menghasilkan sidik jari yang sama.
        hash ^= 0x1f;
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    Ok(format!("{hash:016x}"))
}

pub(crate) fn pesan_manifest(galat: LoadError) -> String {
    match galat {
        LoadError::Manifest(daftar) => {
            let mut teks = String::from("MANIFEST GAGAL VALIDASI:\n");
            for g in &daftar {
                let _ = writeln!(teks, "- {g}");
            }
            teks
        }
        LoadError::Io(pesan) => format!("MANIFEST TIDAK BISA DIBACA:\n{pesan}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::uji::*;

    #[test]
    fn parse_tugas_menerima_delapan_field_dan_mengabaikan_komentar() {
        let isi = "# komentar\n# contoh: a|b|c|d|e|f|g|h\nt1|svc-a|Siapkan rute|jalan|40|agen|plan/t1.md|\n";
        let hasil = parse_tugas(isi, "tugas.tsv").expect("harus valid");
        assert_eq!(hasil.len(), 1, "baris komentar tidak boleh dihitung");
        assert_eq!(hasil[0].progres, 40);
        assert_eq!(hasil[0].berkas, "plan/t1.md");
        assert!(hasil[0].notes.is_empty(), "catatan satu-satunya kolom yang boleh kosong");
    }

    #[test]
    fn parse_tugas_menolak_progres_di_luar_seratus_dan_kolom_kurang() {
        let jauh = "t1|svc-a|Judul|jalan|140|agen|plan/t1.md|\n";
        let galat = parse_tugas(jauh, "tugas.tsv").expect_err("140 bukan progres yang sah");
        assert!(
            galat.iter().any(|g| g.contains("0..=100, dapat `140`")),
            "pesan harus menyebut nilai yang salah, bukan diam-diam memotong: {galat:?}"
        );
        let kurang = "t1|svc-a|Judul|jalan|40|agen|\n";
        let galat = parse_tugas(kurang, "tugas.tsv").expect_err("tujuh field ditolak");
        assert!(galat.iter().any(|g| g.contains("butuh 8 field")), "{galat:?}");
    }

    #[test]
    fn parse_services_menerima_tujuh_field_termasuk_ikon() {
        let isi = "# komentar\napp|App|Flutter|Deskripsi|svc-muka|docs|flutter\n";
        let hasil = parse_services(isi, "services.tsv").expect("harus valid");
        assert_eq!(hasil.len(), 1);
        assert_eq!(hasil[0].docs, vec!["docs"]);
        assert_eq!(hasil[0].ikon, "flutter", "kolom ikon wajib terbaca");

        // Enam field tanpa ikon sekarang cacat, bukan lolos diam-diam.
        let cacat = parse_services("app|App|Flutter|Deskripsi|folder|docs\n", "services.tsv");
        assert!(cacat.unwrap_err()[0].contains("butuh 7 field"));
        // Ikon kosong ditolak dengan pesan yang menunjuk service-nya.
        let kosong = parse_services("app|App|Flutter|Deskripsi|folder|docs|\n", "services.tsv");
        assert!(kosong.unwrap_err()[0].contains("wajib menyebut nama bentuk ikon"));
    }

    #[test]
    fn parse_features_menerima_sepuluh_field() {
        let isi = "a|app|Fitur|aktif|Ringkas|Cara|Masuk|a.rs|docs|-\n";
        let hasil = parse_features(isi, "features.tsv").expect("harus valid");
        assert_eq!(hasil[0].id, "a");
        assert_eq!(hasil[0].notes, "");
    }

    #[test]
    fn parse_edges_menerima_empat_field() {
        let isi = "a|b|alur|label\n";
        let hasil = parse_edges(isi, "edges.tsv").expect("harus valid");
        assert_eq!(hasil[0].from, "a");
    }

    #[test]
    fn parse_products_menerima_tujuh_field() {
        let isi = "p|Produk|Pengguna|Hasil|Ringkasan|docs|-\n";
        let hasil = parse_products(isi, "produk.tsv").expect("harus valid");
        assert_eq!(hasil[0].actor, "Pengguna");
        assert_eq!(hasil[0].notes, "");
    }

    #[test]
    fn parse_apps_menerima_delapan_field_termasuk_ikon() {
        let isi = "a|app|Aplikasi|aplikasi|docs|Deskripsi|docs|kiosk\n";
        let hasil = parse_apps(isi, "applications.tsv").expect("harus valid");
        assert_eq!(hasil[0].service, "app");
        assert_eq!(hasil[0].folder, "docs");
        assert_eq!(hasil[0].ikon, "kiosk", "kolom ikon wajib terbaca");

        let cacat = parse_apps("a|app|Aplikasi|aplikasi|docs|Deskripsi|docs\n", "applications.tsv");
        assert!(cacat.unwrap_err()[0].contains("butuh 8 field"));
    }

    #[test]
    fn parse_mappings_menerima_tiga_field() {
        let isi = "a|p|a\n";
        let hasil = parse_mappings(isi, "pemetaan.tsv").expect("harus valid");
        assert_eq!(hasil[0].subfeature, "a");
        assert_eq!(hasil[0].product, "p");
    }

    #[test]
    fn parse_config_menerima_kosakata_dari_data_dan_menolak_peran_asing() {
        let baik = "merek|Repo X\njudul|Peta Repo X\nbahasa|en\nstatus|done,wip,off\n\
                    status_peran_utama|done\nstatus_peran_separuh|wip\nstatus_peran_mati|off\n\
                    jenis_relasi|flow,call\naktor_ikon|User:person\nminimum_fitur_per_service|3\n";
        let config = parse_config(baik, CONFIG_FILE).expect("config wajib diterima");
        assert_eq!(config.merek, "Repo X");
        assert_eq!(config.minimum_fitur, 3);
        assert_eq!(config.status_peran("utama"), "done");
        assert_eq!(config.statuses.len(), 3);
        // Kosakata asing harus dipakai apa adanya, bukan dipetakan ke kata bawaan repo ini.
        assert_eq!(peran_status("wip", &config), Peran::Separuh);
        assert_eq!(peran_status("unknown", &config), Peran::Belum);

        // Peran yang menunjuk status di luar daftar = galat, supaya vonis tidak dihitung dari nama kosong.
        let cacat = baik.replace("status_peran_utama|done", "status_peran_utama|selesai");
        let galat = parse_config(&cacat, CONFIG_FILE).unwrap_err();
        assert!(
            galat.iter().any(|g| g.contains("bukan anggota daftar `status`")),
            "{galat:?}"
        );

        // Kunci wajib hilang harus disebut satu per satu.
        let hilang = "merek|Repo X\n";
        let galat = parse_config(hilang, CONFIG_FILE).unwrap_err();
        assert!(galat.iter().any(|g| g.contains("kunci wajib hilang: jenis_relasi")), "{galat:?}");
    }

    #[test]
    fn cap_manifest_stabil_dan_berupa_hex() {
        // Root dicari lewat satu pintu yang sama dengan penjaga lain, supaya tes ini tetap jalan di
        // repo mandiri yang tidak punya docs/view/ di induknya (ia jatuh ke dataset contoh).
        let root = root_uji();
        assert!(
            root.join(SERVICES_FILE).exists(),
            "root uji {} harus punya manifest",
            root.display()
        );
        let pertama = cap_manifest(&root).expect("cap pertama");
        let kedua = cap_manifest(&root).expect("cap kedua");
        assert_eq!(pertama, kedua);
        assert_eq!(pertama.len(), 16);
        assert!(pertama.chars().all(|c| c.is_ascii_hexdigit()));
    }

}
