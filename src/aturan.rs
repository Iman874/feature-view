//! Aturan yang membuat `check` gagal: keunikan id, cakupan pemetaan, ujung relasi, path bukti
//! dan dokumen, ambang jumlah fitur, dan nilai ikon yang harus ada di kamus bentuk.

use std::collections::HashMap;
use std::collections::HashSet;
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

}
