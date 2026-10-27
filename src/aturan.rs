//! Aturan yang membuat `check` gagal: keunikan id, cakupan pemetaan, ujung relasi, path bukti
//! dan dokumen, ambang jumlah fitur, dan nilai ikon yang harus ada di kamus bentuk.

use std::collections::HashMap;
use std::collections::HashSet;
use std::fs;
use std::path::Component;
use std::path::Path;
use crate::data::*;
use crate::manifest::*;
use crate::aset::*;

pub(crate) fn validate(model: &Model, root: &Path) -> Vec<String> {
    let mut galat = Vec::new();
    let mut kunci_service = HashSet::new();
    let mut id_fitur = HashSet::new();
    let mut nama_fitur = HashSet::new();

    // Kamus bentuk ikon ada di kode tampilan; manifest wajib menunjuk nama yang ada di dalamnya.
    let (js_teks, _) = aset("render.js", JS);
    let bentuk = bentuk_terdaftar(&js_teks);
    if bentuk.is_empty() {
        galat.push(
            "kamus bentuk ikon tidak terbaca dari render.js — tidak ada satu pun nilai `ikon` yang bisa dibuktikan"
                .to_string(),
        );
    }
    for service in &model.services {
        if !bentuk.is_empty() && !bentuk.contains(&service.ikon) {
            galat.push(format!(
                "{}: service {} memakai ikon `{}` yang tidak ada di kamus bentuk ({} bentuk tersedia)",
                SERVICES_FILE,
                service.key,
                service.ikon,
                bentuk.len()
            ));
        }
    }
    for aplikasi in &model.apps {
        if !bentuk.is_empty() && !bentuk.contains(&aplikasi.ikon) {
            galat.push(format!(
                "{}:{}: aplikasi {} memakai ikon `{}` yang tidak ada di kamus bentuk",
                APPS_FILE, aplikasi.line, aplikasi.id, aplikasi.ikon
            ));
        }
    }
    for (aktor, ikon) in &model.config.aktor_ikon {
        if !bentuk.is_empty() && !bentuk.contains(ikon) {
            galat.push(format!(
                "{}: aktor `{aktor}` menunjuk ikon `{ikon}` yang tidak ada di kamus bentuk",
                CONFIG_FILE
            ));
        }
    }
    if model.services.len() > 40 {
        galat.push(format!(
            "{}: terlalu banyak service ({}); halaman ini tidak dirancang untuk graf selebar itu",
            SERVICES_FILE,
            model.services.len()
        ));
    }

    for service in &model.services {
        if !kunci_service.insert(service.key.clone()) {
            galat.push(format!("service duplikat: {}", service.key));
        }
        if !kunci_service_aman(&service.key) {
            galat.push(format!("{}: key service tidak valid: {}", SERVICES_FILE, service.key));
        }
        if !path_relatif_aman(&service.folder) {
            galat.push(format!(
                "{}: folder service harus path relatif tanpa '..': {}",
                SERVICES_FILE, service.folder
            ));
        } else if !root.join(&service.folder).is_dir() {
            galat.push(format!(
                "{}: folder service tidak ditemukan: {}",
                SERVICES_FILE, service.folder
            ));
        }
        for dokumen in &service.docs {
            if !path_relatif_aman(dokumen) || !path_ada(root, dokumen) {
                galat.push(format!(
                    "{}: dokumen service tidak ditemukan: {}",
                    SERVICES_FILE, dokumen
                ));
            }
        }
    }

    let mut id_produk = HashSet::new();
    for produk in &model.products {
        if !id_aman(&produk.id) {
            galat.push(format!(
                "{}:{}: id produk tidak valid: {}",
                PRODUCTS_FILE, produk.line, produk.id
            ));
        }
        if !id_produk.insert(produk.id.clone()) {
            galat.push(format!(
                "{}:{}: id produk duplikat: {}",
                PRODUCTS_FILE, produk.line, produk.id
            ));
        }
        for dokumen in &produk.docs {
            if !path_relatif_aman(dokumen) || !path_ada(root, dokumen) {
                galat.push(format!(
                    "{}:{}: dokumen produk tidak ditemukan: {}",
                    PRODUCTS_FILE, produk.line, dokumen
                ));
            }
        }
    }

    let mut id_app = HashSet::new();
    for aplikasi in &model.apps {
        if !id_aman(&aplikasi.id) {
            galat.push(format!(
                "{}:{}: id aplikasi tidak valid: {}",
                APPS_FILE, aplikasi.line, aplikasi.id
            ));
        }
        if !id_app.insert(aplikasi.id.clone()) {
            galat.push(format!(
                "{}:{}: id aplikasi duplikat: {}",
                APPS_FILE, aplikasi.line, aplikasi.id
            ));
        }
        if !kunci_service.contains(&aplikasi.service) {
            galat.push(format!(
                "{}:{}: service aplikasi tidak dikenal: {}",
                APPS_FILE, aplikasi.line, aplikasi.service
            ));
        }
        if !path_relatif_aman(&aplikasi.folder) || !root.join(&aplikasi.folder).is_dir() {
            galat.push(format!(
                "{}:{}: folder aplikasi tidak ditemukan: {}",
                APPS_FILE, aplikasi.line, aplikasi.folder
            ));
        }
        for dokumen in &aplikasi.docs {
            if !path_relatif_aman(dokumen) || !path_ada(root, dokumen) {
                galat.push(format!(
                    "{}:{}: dokumen aplikasi tidak ditemukan: {}",
                    APPS_FILE, aplikasi.line, dokumen
                ));
            }
        }
    }

    // Pemetaan diperiksa dua arah. Arah "fitur belum dipetakan" sudah ada; arah sebaliknya
    // tanpa ini meloloskan baris yang menunjuk sub-fitur yang tidak pernah ada, dan baris itu
    // hilang tanpa jejak karena halaman hanya membaca pemetaan yang punya fitur.
    let semua_id_fitur: HashSet<&str> = model.features.iter().map(|f| f.id.as_str()).collect();
    let mut peta = HashMap::new();
    for mapping in &model.mappings {
        if !semua_id_fitur.contains(&mapping.subfeature.as_str()) {
            galat.push(format!(
                "{}:{}: sub-fitur tidak dikenal: {}",
                MAPPING_FILE, mapping.line, mapping.subfeature
            ));
        }
        if peta
            .insert(
                mapping.subfeature.clone(),
                (mapping.product.clone(), mapping.app.clone()),
            )
            .is_some()
        {
            galat.push(format!(
                "{}:{}: sub-fitur dipetakan lebih dari sekali: {}",
                MAPPING_FILE, mapping.line, mapping.subfeature
            ));
        }
        if !id_produk.contains(&mapping.product) {
            galat.push(format!(
                "{}:{}: produk tidak dikenal: {}",
                MAPPING_FILE, mapping.line, mapping.product
            ));
        }
        if !id_app.contains(&mapping.app) {
            galat.push(format!(
                "{}:{}: aplikasi tidak dikenal: {}",
                MAPPING_FILE, mapping.line, mapping.app
            ));
        }
    }

    for fitur in &model.features {
        if !id_aman(&fitur.id) {
            galat.push(format!(
                "{}:{}: id fitur tidak valid (huruf kecil, angka, tanda hubung): {}",
                FEATURES_FILE, fitur.line, fitur.id
            ));
        }
        if !id_fitur.insert(fitur.id.clone()) {
            galat.push(format!(
                "{}:{}: id fitur duplikat: {}",
                FEATURES_FILE, fitur.line, fitur.id
            ));
        }
        if !kunci_service.contains(&fitur.service) {
            galat.push(format!(
                "{}:{}: service tidak dikenal: {}",
                FEATURES_FILE, fitur.line, fitur.service
            ));
        }
        if fitur.product.trim().is_empty() || fitur.app.trim().is_empty() {
            galat.push(format!(
                "{}:{}: sub-fitur belum dipetakan ke produk/aplikasi: {}",
                FEATURES_FILE, fitur.line, fitur.id
            ));
        } else {
            if !id_produk.contains(&fitur.product) {
                galat.push(format!(
                    "{}:{}: produk tidak dikenal: {}",
                    FEATURES_FILE, fitur.line, fitur.product
                ));
            }
            if !id_app.contains(&fitur.app) {
                galat.push(format!(
                    "{}:{}: aplikasi tidak dikenal: {}",
                    FEATURES_FILE, fitur.line, fitur.app
                ));
            }
            if let Some(aplikasi) = model.apps.iter().find(|a| a.id == fitur.app) {
                if aplikasi.service != fitur.service {
                    galat.push(format!(
                        "{}:{}: aplikasi {} milik service {}, bukan {}",
                        FEATURES_FILE, fitur.line, fitur.app, aplikasi.service, fitur.service
                    ));
                }
            }
        }
        if !model.config.statuses.iter().any(|s| s == &fitur.status) {
            galat.push(format!(
                "{}:{}: status tidak valid: {} (boleh: {})",
                FEATURES_FILE,
                fitur.line,
                fitur.status,
                model.config.statuses.join(", ")
            ));
        }
        if !nama_fitur.insert((fitur.service.clone(), fitur.name.clone())) {
            galat.push(format!(
                "{}:{}: fitur duplikat untuk service {}: {}",
                FEATURES_FILE, fitur.line, fitur.service, fitur.name
            ));
        }
        for bukti in &fitur.evidence {
            if !path_relatif_aman(bukti) || !root.join(bukti).is_file() {
                galat.push(format!(
                    "{}:{}: bukti tidak ditemukan: {}",
                    FEATURES_FILE, fitur.line, bukti
                ));
            }
        }
        for dokumen in &fitur.docs {
            if !path_relatif_aman(dokumen) || !path_ada(root, dokumen) {
                galat.push(format!(
                    "{}:{}: dokumen fitur tidak ditemukan: {}",
                    FEATURES_FILE, fitur.line, dokumen
                ));
            }
        }
        let peran_catatan = peran_status(&fitur.status, &model.config);
        if (peran_catatan == Peran::Separuh || peran_catatan == Peran::Mati)
            && fitur.notes.trim().is_empty()
        {
            galat.push(format!(
                "{}:{}: fitur {} berstatus {} wajib punya catatan",
                FEATURES_FILE, fitur.line, fitur.id, fitur.status
            ));
        }
    }

    for fitur in &model.features {
        if !peta.contains_key(&fitur.id) {
            galat.push(format!(
                "{}:{}: sub-fitur belum dipetakan ke produk dan aplikasi: {}",
                MAPPING_FILE, fitur.line, fitur.id
            ));
        }
    }

    for produk in &model.products {
        let jumlah = model
            .features
            .iter()
            .filter(|f| f.product == produk.id)
            .count();
        if jumlah == 0 {
            galat.push(format!(
                "produk {} tidak punya sub-fitur",
                produk.id
            ));
        }
    }

    for aplikasi in &model.apps {
        let jumlah = model
            .features
            .iter()
            .filter(|f| f.app == aplikasi.id)
            .count();
        if jumlah == 0 {
            galat.push(format!(
                "aplikasi {} tidak punya sub-fitur",
                aplikasi.id
            ));
        }
    }

    for service in &model.services {
        if !model.apps.iter().any(|a| a.service == service.key) {
            galat.push(format!(
                "service {} tidak punya aplikasi terdaftar",
                service.key
            ));
        }
    }

    let mut derajat: HashMap<String, usize> = HashMap::new();
    for edge in &model.edges {
        if !model.config.kinds.iter().any(|k| k == &edge.kind) {
            galat.push(format!(
                "{}:{}: jenis edge tidak valid: {} (boleh: {})",
                EDGES_FILE,
                edge.line,
                edge.kind,
                model.config.kinds.join(", ")
            ));
        }
        if edge.from == edge.to {
            galat.push(format!(
                "{}:{}: self-loop tidak boleh: {}",
                EDGES_FILE, edge.line, edge.from
            ));
        }
        for ujung in [&edge.from, &edge.to] {
            if !id_fitur.contains(ujung) {
                galat.push(format!(
                    "{}:{}: fitur tidak dikenal di edge: {}",
                    EDGES_FILE, edge.line, ujung
                ));
            }
        }
        *derajat.entry(edge.from.clone()).or_insert(0) += 1;
        *derajat.entry(edge.to.clone()).or_insert(0) += 1;
    }

    let mut edge_terlihat = HashSet::new();
    for edge in &model.edges {
        let kunci = (
            edge.from.clone(),
            edge.to.clone(),
            edge.kind.clone(),
            edge.label.clone(),
        );
        if !edge_terlihat.insert(kunci) {
            galat.push(format!(
                "{}:{}: edge duplikat: {} -> {} ({})",
                EDGES_FILE, edge.line, edge.from, edge.to, edge.label
            ));
        }
    }

    for fitur in &model.features {
        if derajat.get(&fitur.id).copied().unwrap_or(0) == 0 {
            galat.push(format!(
                "{}:{}: fitur {} tidak punya relasi; tambahkan minimal satu edge",
                FEATURES_FILE, fitur.line, fitur.id
            ));
        }
    }

    galat.extend(aturan_tugas(model, root));

    for service in &model.services {
        let jumlah = model
            .features
            .iter()
            .filter(|f| f.service == service.key)
            .count();
        if jumlah < model.config.minimum_fitur {
            galat.push(format!(
                "service {} hanya punya {} fitur; minimal {}",
                service.key, jumlah, model.config.minimum_fitur
            ));
        }
    }

    galat
}

/// Aturan papan pekerjaan. `tugas.tsv` opsional, jadi repo yang belum memakainya tidak mendapat
/// galat apa pun dari sini.
///
/// Yang dijaga adalah KEJUJURAN angka, bukan bentuknya: sebuah baris hanya boleh ada kalau punya
/// rumah (sub-fitur yang nyata di `features.tsv`), punya bukti (berkas task nyata di repo ini),
/// dan tidak membantah berkasnya sendiri.
fn aturan_tugas(model: &Model, root: &Path) -> Vec<String> {
    let mut galat = Vec::new();
    if model.tugas.is_empty() {
        return galat;
    }
    if model.config.status_tugas.is_empty() {
        galat.push(format!(
            "{TASKS_FILE}: papan pekerjaan dipakai, tapi kosakata `status_tugas` tidak dideklarasikan di config.tsv — nama status tidak boleh hidup di kode alat"
        ));
    }
    let selesai = model
        .config
        .status_tugas
        .last()
        .cloned()
        .unwrap_or_default();
    let mut id_terpakai: HashSet<&str> = HashSet::new();

    for t in &model.tugas {
        if !id_terpakai.insert(t.id.as_str()) {
            galat.push(format!("{}:{}: id tugas duplikat: {}", TASKS_FILE, t.line, t.id));
        }
        if !model.features.iter().any(|f| f.id == t.subfitur) {
            galat.push(format!(
                "{}:{}: tugas `{}` menunjuk sub-fitur yang tidak ada di features.tsv: {}",
                TASKS_FILE, t.line, t.id, t.subfitur
            ));
        }
        if !model.config.status_tugas.iter().any(|s| s == &t.status) {
            galat.push(format!(
                "{}:{}: status tugas `{}` bukan anggota `status_tugas` ({})",
                TASKS_FILE,
                t.line,
                t.status,
                model.config.status_tugas.join(", ")
            ));
        }
        if !path_relatif_aman(&t.berkas) || !root.join(&t.berkas).is_file() {
            galat.push(format!(
                "{}:{}: berkas tugas tidak ditemukan: {} — tulis task-nya di repo ini lebih dulu, papan ini tidak menerima kerja tanpa berkas",
                TASKS_FILE, t.line, t.berkas
            ));
            continue;
        }
        if t.progres == 100 && !selesai.is_empty() && t.status != selesai {
            galat.push(format!(
                "{}:{}: progres 100 tapi status `{}` bukan status terakhir (`{}`)",
                TASKS_FILE, t.line, t.status, selesai
            ));
        }
        if !selesai.is_empty() && t.status == selesai && t.progres != 100 {
            galat.push(format!(
                "{}:{}: status `{}` wajib progres 100, dapat {}",
                TASKS_FILE, t.line, t.status, t.progres
            ));
        }
        // Anti-bohong: berkas task yang menulis Status tidak boleh dibantah manifestnya.
        if let Ok(isi) = fs::read_to_string(root.join(&t.berkas)) {
            if let Some(kata) = kata_status_pertama(&isi) {
                if kata != t.status.to_ascii_lowercase() {
                    galat.push(format!(
                        "{}:{}: tugas `{}` mengklaim `{}` tetapi berkas {} menulis `Status: {}`",
                        TASKS_FILE, t.line, t.id, t.status, t.berkas, kata
                    ));
                }
            }
        }
    }
    galat
}

/// Kata pertama setelah label `Status:` pada baris status sebuah berkas task, sudah dikasarkeun.
/// Hanya baris yang DIAWALI label itu yang dihitung — `Status:` yang muncul di tengah kalimat
/// (misalnya di dalam tanda backtick pada langkah kerja) bukan status pekerjaan.
///
/// Bentuk barisnya di repo nyata tidak seragam: `> **Status**: selesai`, `Status: SELESAI — ...`.
/// Yang dibandingkan adalah KATA PERTAMA, bukan substring: `belum dikerjakan` harus tetap berbeda
/// dari `dikerjakan`, karena di situlah celah paling mungkin membuat papan terlihat sibuk.
fn kata_status_pertama(isi: &str) -> Option<String> {
    for baris in isi.lines() {
        let bersih = baris.trim();
        if bersih.is_empty() || bersih.starts_with('#') {
            continue;
        }
        let kutip = bersih.trim_start_matches(['>', '#', '*', '`', '-', ' ']);
        let rendah = kutip.to_ascii_lowercase();
        let tanpa_bintang = rendah.trim_start_matches('*').trim_start_matches('`');
        if !tanpa_bintang.starts_with("status") {
            continue;
        }
        let sesudah = &tanpa_bintang["status".len()..];
        let Some(titik) = sesudah.find(':') else { continue };
        let kata: String = sesudah[titik + 1..]
            .chars()
            .skip_while(|c| !c.is_alphanumeric())
            .take_while(|c| c.is_alphanumeric())
            .collect();
        if !kata.is_empty() {
            return Some(kata);
        }
    }
    None
}

pub(crate) fn id_aman(id: &str) -> bool {
    if id.is_empty() || id.starts_with('-') || id.ends_with('-') {
        return false;
    }
    id.chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

pub(crate) fn kunci_service_aman(id: &str) -> bool {
    !id.is_empty()
        && id
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '-')
}

pub(crate) fn path_relatif_aman(path: &str) -> bool {
    if path.trim().is_empty() {
        return false;
    }
    let p = Path::new(path);
    if p.is_absolute() {
        return false;
    }
    for komponen in p.components() {
        match komponen {
            Component::Normal(_) => {}
            _ => return false,
        }
    }
    true
}

pub(crate) fn path_ada(root: &Path, path: &str) -> bool {
    let target = root.join(path);
    target.is_file() || target.is_dir()
}

pub(crate) fn beda_baris_pertama(a: &str, b: &str) -> usize {
    for (indeks, (baris_a, baris_b)) in a.lines().zip(b.lines()).enumerate() {
        if baris_a != baris_b {
            return indeks + 1;
        }
    }
    a.lines().count().min(b.lines().count()) + 1
}



#[cfg(test)]
mod tests {
    use super::*;
    use crate::uji::*;
    #[test]
    fn validate_menolak_pemetaan_hilang() {
        let mut model = model_uji(
            vec![
                fitur("a", "app", "aktif", "tools/feature-view/Cargo.toml"),
                fitur("b", "app", "aktif", "tools/feature-view/Cargo.toml"),
            ],
            vec![edge("a", "b")],
        );
        model.mappings.clear();
        let galat = validate(&model, Path::new("."));
        assert!(galat.iter().any(|g| g.contains("belum dipetakan")));
    }

    #[test]
    fn validate_menolak_pemetaan_subfitur_hantu() {
        let mut model = model_uji(
            vec![
                fitur("a", "app", "aktif", "tools/feature-view/Cargo.toml"),
                fitur("b", "app", "aktif", "tools/feature-view/Cargo.toml"),
            ],
            vec![edge("a", "b")],
        );
        // Arah pengecekan yang belum ada: baris pemetaan yang menunjuk sub-fitur yang tidak
        // pernah ditulis. Dulu `check` dan `generate` keduanya OK, barisnya tidak muncul di
        // halaman, dan ringkasan tetap bilang jumlah sub-fitur yang benar.
        model.mappings.push(Mapping {
            subfeature: "hantu".to_string(),
            product: "p".to_string(),
            app: "a".to_string(),
            line: 9,
        });
        let galat = validate(&model, Path::new("."));
        assert!(
            galat.iter().any(|g| g.contains("sub-fitur tidak dikenal: hantu")),
            "pemetaan ke sub-fitur yang tidak ada harus ditolak, bukan diabaikan: {galat:?}"
        );
    }

    #[test]
    fn validate_menolak_edge_tanpa_ujung() {
        let model = model_uji(
            vec![
                fitur("a", "app", "aktif", "tools/feature-view/Cargo.toml"),
                fitur("b", "app", "aktif", "tools/feature-view/Cargo.toml"),
            ],
            vec![edge("a", "tidak-ada")],
        );
        let galat = validate(&model, Path::new("."));
        assert!(galat.iter().any(|g| g.contains("fitur tidak dikenal di edge")));
    }

    #[test]
    fn validate_menolak_fitur_tanpa_relasi() {
        let model = model_uji(
            vec![
                fitur("a", "app", "aktif", "tools/feature-view/Cargo.toml"),
                fitur("b", "app", "aktif", "tools/feature-view/Cargo.toml"),
            ],
            vec![],
        );
        let galat = validate(&model, Path::new("."));
        assert!(galat.iter().any(|g| g.contains("tidak punya relasi")));
    }

    #[test]
    fn validate_menolak_status_tidak_dikenal() {
        let model = model_uji(
            vec![
                fitur("a", "app", "aktif", "tools/feature-view/Cargo.toml"),
                fitur("b", "app", "ngawur", "tools/feature-view/Cargo.toml"),
            ],
            vec![edge("a", "b")],
        );
        let galat = validate(&model, Path::new("."));
        assert!(galat.iter().any(|g| g.contains("status tidak valid")));
    }

    #[test]
    fn validate_menolak_bukti_hilang() {
        let model = model_uji(
            vec![
                fitur("a", "app", "aktif", "file-tidak-ada-abc.txt"),
                fitur("b", "app", "aktif", "tools/feature-view/Cargo.toml"),
            ],
            vec![edge("a", "b")],
        );
        let galat = validate(&model, Path::new("."));
        assert!(galat.iter().any(|g| g.contains("bukti tidak ditemukan")));
    }

    #[test]
    fn validate_menolak_path_keluar_root() {
        assert!(!path_relatif_aman("../rahasia.txt"));
        assert!(!path_relatif_aman("C:/Windows/win.ini"));
        assert!(path_relatif_aman("docs/view/index.html"));
    }

    #[test]
    fn validate_menolak_ikon_yang_tidak_ada_di_kamus_bentuk() {
        let mut model = model_uji(
            vec![
                fitur("a", "app", "aktif", "tools/feature-view/Cargo.toml"),
                fitur("b", "app", "aktif", "tools/feature-view/Cargo.toml"),
            ],
            vec![edge("a", "b")],
        );
        model.apps[0].ikon = "bentuk-tidak-ada".to_string();
        let galat = validate(&model, Path::new("."));
        assert!(
            galat.iter().any(|g| g.contains("bentuk-tidak-ada") && g.contains("kamus bentuk")),
            "ikon aplikasi asing harus tertangkap: {galat:?}"
        );

        model.apps[0].ikon = "aplikasi".to_string();
        model.services[0].ikon = "juga-tidak-ada".to_string();
        let galat = validate(&model, Path::new("."));
        assert!(
            galat.iter().any(|g| g.contains("service app")),
            "ikon service asing harus tertangkap: {galat:?}"
        );
    }

    #[test]
    fn beda_baris_pertama_melaporkan_baris_yang_berbeda() {
        assert_eq!(beda_baris_pertama("a\nb\nc", "a\nx\nc"), 2);
        assert_eq!(beda_baris_pertama("a\nb", "a\nb\nc"), 3);
    }

    fn tugas_uji(id: &str, rumah: &str, status: &str, progres: u8, berkas: &str) -> Tugas {
        Tugas {
            id: id.to_string(),
            subfitur: rumah.to_string(),
            judul: "Pekerjaan uji".to_string(),
            status: status.to_string(),
            progres,
            pemegang: "agen".to_string(),
            berkas: berkas.to_string(),
            notes: String::new(),
            line: 2,
        }
    }

    fn dua_fitur() -> Vec<Feature> {
        vec![
            fitur("a", "app", "aktif", "tools/feature-view/Cargo.toml"),
            fitur("b", "app", "aktif", "tools/feature-view/Cargo.toml"),
        ]
    }

    #[test]
    fn papan_menolak_tugas_tanpa_rumah_dan_berkas_hilang() {
        let mut model = model_uji(dua_fitur(), vec![edge("a", "b")]);
        model.config.status_tugas = vec!["jalan".to_string(), "selesai".to_string()];
        model
            .tugas
            .push(tugas_uji("t1", "hantu", "jalan", 40, "tidak/ada.md"));
        let galat = validate(&model, Path::new("."));
        assert!(
            galat.iter().any(|g| g.contains("sub-fitur yang tidak ada di features.tsv: hantu")),
            "kerja tanpa rumah harus ditolak: {galat:?}"
        );
        assert!(
            galat.iter().any(|g| g.contains("berkas tugas tidak ditemukan")),
            "kerja tanpa berkas task harus ditolak: {galat:?}"
        );
    }

    #[test]
    fn papan_menolak_kosakata_status_yang_belum_dideklarasikan() {
        let mut model = model_uji(dua_fitur(), vec![edge("a", "b")]);
        // `status_tugas` sengaja kosong: nama status adalah data, dan papan tidak boleh
        // memakai nama yang tidak pernah diumumkan config.
        model
            .tugas
            .push(tugas_uji("t1", "a", "jalan", 40, "tools/feature-view/Cargo.toml"));
        let galat = validate(&model, Path::new("."));
        assert!(
            galat
                .iter()
                .any(|g| g.contains("kosakata `status_tugas` tidak dideklarasikan")),
            "papan tanpa kosakata harus ditolak: {galat:?}"
        );
    }

    #[test]
    fn papan_mengikat_seratus_persen_ke_status_terakhir() {
        let mut model = model_uji(dua_fitur(), vec![edge("a", "b")]);
        model.config.status_tugas = vec!["jalan".to_string(), "selesai".to_string()];
        model
            .tugas
            .push(tugas_uji("t1", "a", "jalan", 100, "Cargo.toml"));
        model
            .tugas
            .push(tugas_uji("t2", "b", "selesai", 60, "Cargo.toml"));
        let galat = validate(&model, Path::new("."));
        assert!(
            galat.iter().any(|g| g.contains("bukan status terakhir")),
            "progres 100 dengan status berjalan harus ditolak: {galat:?}"
        );
        assert!(
            galat.iter().any(|g| g.contains("wajib progres 100, dapat 60")),
            "status terakhir dengan progres di bawah seratus harus ditolak: {galat:?}"
        );
    }

    #[test]
    fn kata_status_membedakan_belum_dari_dikerjakan() {
        // Bentuk nyata di repo pemakai: blockquote tebal, polos, dan kalimat yang menyebut
        // `Status:` di tengahnya. Yang terakhir bukan baris status dan tidak boleh terbaca.
        assert_eq!(
            kata_status_pertama("> **Status**: selesai\n"),
            Some("selesai".to_string())
        );
        assert_eq!(
            kata_status_pertama("Status: DIKERJAKAN — parser selesai, dokumen belum\n"),
            Some("dikerjakan".to_string())
        );
        assert_eq!(
            kata_status_pertama("Status: BELUM DIKERJAKAN\n"),
            Some("belum".to_string()),
            "`belum dikerjakan` tidak boleh terbaca sebagai `dikerjakan`"
        );
        assert_eq!(
            kata_status_pertama("5. Anti-bohong: berkas task menulis `Status:` harus sama\n"),
            None,
            "label di tengah kalimat bukan baris status"
        );
    }

}
