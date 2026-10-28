//! Perangkai keluaran: HTML halaman, JSON data untuk render.js, dan perhitungan status yang
//! ditampilkan. Modul ini tidak memvalidasi dan tidak membaca kode service.

use std::collections::BTreeMap;
use std::collections::HashMap;
use std::fmt::Write as _;
use std::fs;
use std::path::Path;
use crate::data::*;
use crate::manifest::*;
use crate::aset::*;

pub(crate) fn bangun_html(model: &Model, cap: &str) -> String {
    let (css, _) = aset("view.css", CSS);
    let (js, _) = aset("render.js", JS);
    render_html(model, cap, &css, &js)
}

/// Satu-satunya jalan tulis untuk index.html. Dipakai perintah `generate` DAN endpoint
/// `/regenerate`, jadi tombol di halaman menjalankan kode yang sama persis dengan CLI.
pub(crate) fn tulis_index(root: &Path, model: &Model, cap: &str) -> Result<String, String> {
    let html = bangun_html(model, cap);
    let keluaran = root.join(OUTPUT_FILE);
    fs::write(&keluaran, &html)
        .map_err(|e| format!("GAGAL menulis {}: {e}", keluaran.display()))?;
    Ok(format!(
        "{} produk, {} aplikasi, {} sub-fitur, {} service, {} relasi, {} byte ditulis ke {}",
        model.products.len(),
        model.apps.len(),
        model.features.len(),
        model.services.len(),
        model.edges.len(),
        html.len(),
        keluaran.display()
    ))
}

pub(crate) fn generate_lewat_http(root: &Path) -> Result<String, String> {
    let model = muat_model(root).map_err(pesan_manifest)?;
    let cap = cap_manifest(root)?;
    tulis_index(root, &model, &cap)
}

/// Buang hasil generate saja. Path-nya konstan `OUTPUT_FILE`: manifest TSV tidak pernah
/// jadi sasaran, jadi tombol ini tidak bisa menghancurkan sumber kebenaran.
pub(crate) fn hapus_lewat_http(root: &Path) -> Result<String, String> {
    let keluaran = root.join(OUTPUT_FILE);
    match fs::remove_file(&keluaran) {
        Ok(_) => Ok(format!("DIHAPUS: {}", keluaran.display())),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            Ok(format!("SUDAH KOSONG: {} memang belum ada", keluaran.display()))
        }
        Err(e) => Err(format!("GAGAL menghapus {}: {e}", keluaran.display())),
    }
}

/// Halaman saat file hasil belum ada di disk. Bukan halaman kosong dan bukan pesan browser:
/// dari sini owner bisa melahirkan filenya lagi sekali klik.
/// Merek untuk halaman darurat (file belum di-generate / manifest ditolak). Dibaca dari config;
/// kalau config tidak terbaca, pakai frasa netral, bukan nama proyek tertentu.
pub(crate) fn merek_config(root: &Path) -> String {
    fs::read_to_string(root.join(CONFIG_FILE))
        .ok()
        .and_then(|isi| parse_config(&isi, CONFIG_FILE).ok())
        .map(|c| c.merek)
        .filter(|m| !m.is_empty())
        .unwrap_or_else(|| "peta fitur".to_string())
}

pub(crate) fn halaman_belum_generate(keluaran: &Path, merek: &str, root: &Path) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "<!doctype html>");
    let _ = writeln!(out, "<html lang=\"id\"><head><meta charset=\"utf-8\">");
    let _ = writeln!(out, "<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">");
    let _ = writeln!(out, "<title>Belum di-generate - {merek}</title>");
    out.push_str("<style>body{margin:0;padding:2rem 1.25rem;font:14px/1.55 system-ui,sans-serif;color:#1a2027;background:#f2f3f5}main{max-width:46rem}h1{font-size:1.15rem;margin:0 0 .6rem}p{margin:0 0 .8rem}code{font-family:ui-monospace,Consolas,monospace;font-size:.85rem;background:#eceff3;padding:.1rem .35rem;border-radius:4px}button{font:inherit;padding:.42rem .85rem;border:1px solid #bcc4ce;border-radius:6px;background:#fff;color:#1a2027;cursor:pointer}button:hover{border-color:#1f7a5c;color:#1f7a5c}@media(prefers-color-scheme:dark){body{color:#e6e9ee;background:#14181e}code{background:#1e242c}button{background:#1a2027;border-color:#3a434f;color:#e6e9ee}}</style>");
    let _ = writeln!(out, "</head><body><main>");
    let _ = writeln!(out, "<h1>Peta belum di-generate</h1>");
    let _ = writeln!(
        out,
        "<p>Server membaca halaman dari disk, dan file hasilnya tidak ada: <code>{}</code></p>",
        esc(&keluaran.display().to_string())
    );
    let _ = writeln!(out, "<p>Sumber datanya tidak tersentuh: manifest TSV di <code>docs/view/</code> tetap utuh. Yang dihapus tadi hanya hasil generate.</p>");
    let _ = writeln!(out, "<p><button type=\"button\" id=\"gen\">Generate ulang</button></p>");
    let _ = writeln!(out, "<p id=\"status\"></p>");
    let _ = writeln!(
        out,
        "<p>Atau dari terminal, pada root yang sama dengan yang dibaca server ini:<br><code>{}</code></p>",
        esc(&crate::petunjuk("generate", root))
    );
    let _ = writeln!(out, "</main><script>");
    out.push_str("document.getElementById('gen').addEventListener('click', function () {\n");
    out.push_str("  var status = document.getElementById('status');\n");
    out.push_str("  status.textContent = 'Menjalankan generate...';\n");
    out.push_str("  fetch('/regenerate', { method: 'POST' }).then(function (r) {\n");
    out.push_str("    return r.text().then(function (t) { return { ok: r.ok, teks: t.trim() }; });\n");
    out.push_str("  }).then(function (hasil) {\n");
    out.push_str("    status.textContent = hasil.teks;\n");
    out.push_str("    if (!hasil.ok) { return; }\n");
    out.push_str("    setTimeout(function () { location.reload(); }, 700);\n");
    out.push_str("  }).catch(function (e) { status.textContent = String(e); });\n");
    out.push_str("});\n");
    let _ = writeln!(out, "</script></body></html>");
    out
}

pub(crate) fn render_html(model: &Model, cap: &str, css: &str, js: &str) -> String {
    let total = model.features.len();
    let config = &model.config;
    let jumlah = |status: &str| {
        model
            .features
            .iter()
            .filter(|f| f.status == status)
            .count()
    };
    // Opsi penyaring: tiga peran inti selalu ada, sisanya hanya bila ada isinya.
    let peran_inti = ["utama", "separuh", "mati"];
    let mut opsi_status = String::new();
    for status in &config.statuses {
        let ada = jumlah(status);
        let inti = peran_inti
            .iter()
            .any(|p| config.status_peran(p) == status.as_str());
        if ada == 0 && !inti {
            continue;
        }
        let _ = write!(
            opsi_status,
            "<option value=\"{}\">{} · {}</option>",
            esc(status),
            esc(&huruf_pertama_besar(status)),
            ada
        );
    }

    let mut out = String::with_capacity(120_000);
    let bahasa = esc(&config.bahasa);
    let judul = esc(&config.judul);
    let merek = esc(&config.merek);
    let _ = writeln!(out, "<!doctype html>");
    let _ = writeln!(out, "<html lang=\"{bahasa}\">");
    let _ = writeln!(out, "<head>");
    let _ = writeln!(out, "<meta charset=\"utf-8\">");
    let _ = writeln!(
        out,
        "<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">"
    );
    let _ = writeln!(out, "<title>{judul}</title>");
    let _ = writeln!(out, "<style>{css}</style>");
    // Tema dipasang sebelum <body> pertama kali dilukis. Tanpa baris ini, pengguna
    // bersistem gelap melihat sekilas halaman terang lalu berkedip saat render.js jalan,
    // karena CSS tidak lagi memakai @media (prefers-color-scheme): satu sumber kebenaran
    // adalah atribut data-tema, yang juga dibaca warna graf di JavaScript.
    // Kunci 'fv-tema' harus sama dengan KUNCI_TEMA di render.js — dijaga oleh tes
    // kunci_tema_sama_di_cangkang_dan_di_skrip.
    let _ = writeln!(
        out,
        "<script>(function(){{var k='fv-tema',t=null;try{{t=localStorage.getItem(k)}}catch(e){{}}\
         if(t!=='gelap'&&t!=='terang'){{t=(window.matchMedia&&window.matchMedia('(prefers-color-scheme: dark)').matches)?'gelap':'terang'}}\
         document.documentElement.setAttribute('data-tema',t)}})();</script>"
    );
    let _ = writeln!(out, "</head>");
    let _ = writeln!(out, "<body>");
    let _ = writeln!(
        out,
        "<a class=\"lewati\" href=\"#peta\">Lewati ke peta</a>"
    );
    let _ = writeln!(out, "<div class=\"app\">");
    let _ = writeln!(out, "<header class=\"bar\">");
    let _ = writeln!(out, "<div class=\"bar-merek\">");
    // Inisial untuk kotak merek di CSS (content: attr(...)). Diambil dari nilai mentah
    // config.merek, bukan dari `merek` yang sudah diesc, supaya huruf pertamanya tidak
    // bisa berupa '&' hasil entitas.
    let inisial_huruf = config
        .merek
        .trim()
        .chars()
        .next()
        .map(|c| c.to_uppercase().to_string())
        .unwrap_or_default();
    let inisial = esc(&inisial_huruf);
    let _ = writeln!(out, "<h1 data-inisial=\"{inisial}\">{merek}</h1>");
    let _ = writeln!(
        out,
        // Kata sumbu hanya lewat `title`, tidak ditulis sebagai teks: lima pil berlabel
        // membuat header melengkung jadi dua baris bahkan di 1536px, dan tinggi header 56px
        // adalah syarat tertulis.
        "<p class=\"bar-meta\"><span title=\"service\"><i data-ikon=\"layanan\"></i><b>{}</b></span><span title=\"aplikasi\"><i data-ikon=\"aplikasi\"></i><b>{}</b></span><span title=\"produk\"><i data-ikon=\"produk\"></i><b>{}</b></span><span title=\"sub-fitur\"><i data-ikon=\"subfitur\"></i><b>{}</b></span><span title=\"relasi\"><i data-ikon=\"relasi\"></i><b>{}</b></span></p>",
        model.services.len(),
        model.apps.len(),
        model.products.len(),
        total,
        model.edges.len()
    );
    let _ = writeln!(out, "</div>");
    let _ = writeln!(out, "<div class=\"bar-alat\">");
    let _ = writeln!(out, "<div class=\"segmen\" role=\"group\" aria-label=\"Sumbu tampilan\">");
    let _ = writeln!(out, "<button type=\"button\" class=\"tombol terpilih\" data-mode=\"produk\" aria-pressed=\"true\" title=\"Sumbu produk\"><i data-ikon=\"produk\"></i><span>Produk</span></button>");
    let _ = writeln!(out, "<button type=\"button\" class=\"tombol\" data-mode=\"subfitur\" aria-pressed=\"false\" title=\"Alur antar sub-fitur\"><i data-ikon=\"subfitur\"></i><span>Sub-fitur</span></button>");
    let _ = writeln!(out, "<button type=\"button\" class=\"tombol\" data-mode=\"implementasi\" aria-pressed=\"false\" title=\"Sumbu implementasi\"><i data-ikon=\"implementasi\"></i><span>Implementasi</span></button>");
    let _ = writeln!(out, "</div>");
    let _ = writeln!(out, "<button type=\"button\" class=\"tombol ikon-saja tombol-pohon\" id=\"tombol-pohon\" aria-controls=\"pohon\" aria-expanded=\"false\" aria-label=\"Struktur hierarki\" title=\"Struktur hierarki\"><i data-ikon=\"struktur\"></i></button>");
    let _ = writeln!(
        out,
        "<label class=\"pilih-kontrol\"><i data-ikon=\"saring\"></i><select id=\"filter-status\" aria-label=\"Saring berdasar status\"><option value=\"semua\">Semua status · {total}</option>{opsi_status}"
    );
    let _ = writeln!(out, "</select></label>");
    let _ = writeln!(
        out,
        "<label class=\"pilih-kontrol\"><i data-ikon=\"layanan\"></i><select id=\"filter-service\" aria-label=\"Saring berdasar service\"><option value=\"semua\">Semua service · {}</option>",
        model.services.len()
    );
    for service in &model.services {
        let jumlah = model
            .features
            .iter()
            .filter(|f| f.service == service.key)
            .count();
        let _ = writeln!(
            out,
            "<option value=\"{}\">{} · {jumlah}</option>",
            esc(&service.key),
            esc(&service.name)
        );
    }
    let _ = writeln!(out, "</select></label>");
    // Saklar tema: labelnya ditulis render.js dari keadaan GELAP yang sebenarnya, jadi
    // tombol ini tidak pernah menjanjikan tema yang sama dengan yang sedang tampil.
    let _ = writeln!(
        out,
        "<button type=\"button\" class=\"tombol\" id=\"tombol-tema\" data-aksi=\"tema\"><span>Gelap</span></button>"
    );
    // Dua aksi alat, keduanya lahir dalam keadaan hidden: halaman dari disk tidak boleh
    // menawarkan tombol yang toh tidak bisa dijalankan browser.
    let _ = writeln!(
        out,
        "<button type=\"button\" class=\"tombol ikon-saja\" id=\"generate-ulang\" data-aksi=\"generate-ulang\" aria-label=\"Jalankan tools generate dan tulis index.html\" title=\"Generate ulang\" hidden><i data-ikon=\"kode\"></i></button>"
    );
    let _ = writeln!(
        out,
        "<button type=\"button\" class=\"tombol ikon-saja\" id=\"hapus-hasil\" data-aksi=\"hapus-hasil\" aria-label=\"Hapus hasil generate index.html\" title=\"Hapus hasil\" hidden><i data-ikon=\"buang\"></i></button>"
    );
    let _ = writeln!(
        out,
        "<button type=\"button\" class=\"tombol ikon-saja\" data-aksi=\"muat-data\" aria-label=\"Muat ulang data dari manifest\" title=\"Muat data\"><i data-ikon=\"folder\"></i></button>"
    );
    let _ = writeln!(
        out,
        "<button type=\"button\" class=\"tombol ikon-saja\" data-aksi=\"reset\" aria-label=\"Setel ulang penyaring, pilihan, dan zoom\" title=\"Setel ulang\"><i data-ikon=\"perbarui\"></i></button>"
    );
    let _ = writeln!(
        out,
        "<button type=\"button\" class=\"kap\" id=\"kap\" data-aksi=\"muat-ubah\" data-cap=\"{cap}\" hidden aria-live=\"polite\"></button>"
    );
    let _ = writeln!(out, "</div>");
    let _ = writeln!(out, "</header>");

    let _ = writeln!(out, "<div class=\"badan\">");
    let _ = writeln!(out, "<aside id=\"pohon\" class=\"pohon\" aria-label=\"Struktur hierarki\">");
    let _ = writeln!(out, "<div class=\"pohon-kepala\"><h2 id=\"pohon-judul\">Produk</h2><div class=\"pohon-kepala-alat\"><button type=\"button\" class=\"tombol ikon-saja pohon-kecil\" id=\"kecil-pohon\" aria-label=\"Perkecil struktur\" aria-pressed=\"false\" title=\"Perkecil\"><i data-ikon=\"struktur\"></i></button><button type=\"button\" class=\"tombol ikon-saja pohon-tutup\" id=\"tutup-pohon\" aria-label=\"Tutup struktur\" title=\"Tutup\"><i data-ikon=\"tutup\"></i></button></div></div>");
    let _ = writeln!(
        out,
        "<label class=\"cari\"><i data-ikon=\"cari\"></i><span class=\"sr-only\">Cari</span><input id=\"cari\" type=\"search\" placeholder=\"Cari\" autocomplete=\"off\"></label>"
    );
    let _ = writeln!(
        out,
        "<p id=\"hasil\" class=\"hasil\" aria-live=\"polite\"></p>"
    );
    let _ = writeln!(out, "<nav id=\"pohon-isi\" class=\"pohon-isi\"></nav>");
    // Legenda TIDAK lagi di kolom ini. Ia pindah ke tombol info di kanvas (lihat section
    // `peta`), jadi daftar struktur dan papan pekerjaan dapat seluruh tinggi sidebar.
    // Papan pekerjaan: disembunyikan sampai JS punya sesuatu untuk ditampilkan. Judul dan nama
    // status datang dari config, jadi panel ini tidak pernah membawa kata milik satu repo pun
    // di kode alat. Dua daftar berbagi satu ruang lewat tab, bukan ditumpuk ke bawah.
    let _ = writeln!(out, "<section id=\"papan\" class=\"papan\" hidden aria-live=\"polite\">");
    let _ = writeln!(out, "<div class=\"papan-tab\" id=\"papan-tab\" role=\"tablist\" aria-label=\"Papan pekerjaan\">");
    let _ = writeln!(out, "<button type=\"button\" class=\"papan-tab-tombol terpilih\" id=\"papan-tab-berjalan\" role=\"tab\" aria-selected=\"true\" aria-controls=\"papan-isi\" tabindex=\"0\"><span id=\"papan-tab-berjalan-nama\"></span><b id=\"papan-jumlah-berjalan\" class=\"papan-angka\"></b></button>");
    let _ = writeln!(out, "<button type=\"button\" class=\"papan-tab-tombol\" id=\"papan-tab-selesai\" role=\"tab\" aria-selected=\"false\" aria-controls=\"papan-selesai-isi\" tabindex=\"-1\"><span id=\"papan-tab-selesai-nama\"></span><b id=\"papan-jumlah-selesai\" class=\"papan-angka\"></b></button>");
    let _ = writeln!(out, "</div>");
    let _ = writeln!(out, "<div id=\"papan-isi\" class=\"papan-isi\" role=\"tabpanel\" aria-labelledby=\"papan-tab-berjalan\" tabindex=\"0\"></div>");
    let _ = writeln!(out, "<div id=\"papan-selesai-isi\" class=\"papan-isi\" role=\"tabpanel\" aria-labelledby=\"papan-tab-selesai\" tabindex=\"0\" hidden></div>");
    let _ = writeln!(out, "</section>");
    let _ = writeln!(out, "</aside>");

    let _ = writeln!(out, "<section class=\"peta\" id=\"peta\" aria-label=\"Jejaring fitur\">");
    let _ = writeln!(out, "<div class=\"alat-peta\" role=\"group\" aria-label=\"Kontrol peta\">");
    let _ = writeln!(out, "<button type=\"button\" class=\"tombol ikon-saja\" data-zoom=\"in\" aria-label=\"Perbesar\" title=\"Perbesar\"><i data-ikon=\"tambah\"></i></button>");
    let _ = writeln!(out, "<button type=\"button\" class=\"tombol ikon-saja\" data-zoom=\"out\" aria-label=\"Perkecil\" title=\"Perkecil\"><i data-ikon=\"kurang\"></i></button>");
    let _ = writeln!(out, "<button type=\"button\" class=\"tombol ikon-saja\" data-zoom=\"reset\" aria-label=\"Sesuaikan tampilan\" title=\"Sesuaikan\"><i data-ikon=\"sesuaikan\"></i></button>");
    let _ = writeln!(out, "<button type=\"button\" class=\"tombol ikon-saja\" data-aksi=\"semua\" aria-label=\"Tampilkan semua node\" title=\"Semua node\"><i data-ikon=\"semua\"></i></button>");
    let _ = writeln!(out, "<span class=\"zoom-tingkat\" id=\"zoom-tingkat\">100%</span>");
    let _ = writeln!(out, "</div>");
    let _ = writeln!(
        out,
        "<svg id=\"jejaring\" role=\"group\" aria-label=\"Jejaring fitur {merek}\"></svg>"
    );
    let _ = writeln!(
        out,
        "<div class=\"minimap\" aria-label=\"Peta mini\"><svg id=\"minimap\" role=\"img\" aria-label=\"Peta mini jejaring\"></svg></div>"
    );
    // Legenda dijinjing satu tombol `i` di kanan atas kanvas. Popupnya ada di luar <svg>,
    // jadi menggeser peta tidak pernah menyeret legenda ikut berpindah.
    let _ = writeln!(out, "<div class=\"info-peta\">");
    let _ = writeln!(out, "<button type=\"button\" class=\"tombol ikon-saja tombol-info\" id=\"tombol-info\" aria-label=\"Info legenda peta\" aria-expanded=\"false\" aria-controls=\"legenda-popover\" title=\"Legenda\"><i data-ikon=\"info\"></i></button>");
    let _ = writeln!(out, "<section id=\"legenda-popover\" class=\"legenda legenda-popover\" aria-label=\"Legenda peta\" hidden>");
    let _ = writeln!(out, "<div><h2><i data-ikon=\"relasi\"></i><span>Relasi</span></h2><div id=\"legenda-edge\" class=\"legenda-baris\"></div></div>");
    let _ = writeln!(out, "<div><h2><i data-ikon=\"status\"></i><span>Status</span></h2><div id=\"legenda-status\" class=\"legenda-baris\"></div></div>");
    let _ = writeln!(out, "</section>");
    let _ = writeln!(out, "</div>");
    let _ = writeln!(out, "</section>");
    let _ = writeln!(
        out,
        "<aside id=\"detail\" class=\"detail\" aria-live=\"polite\">"
    );
    let _ = writeln!(out, "<div class=\"detail-kosong\"><i data-ikon=\"produk\"></i><p>Pilih produk</p></div>");
    let _ = writeln!(out, "</aside>");
    let _ = writeln!(out, "</div>");
    let _ = writeln!(out, "</div>");
    let _ = writeln!(
        out,
        "<script id=\"data\" type=\"application/json\">{}</script>",
        render_json(model)
    );
    let _ = writeln!(out, "<script>{js}</script>");
    let _ = writeln!(out, "</body>");
    let _ = writeln!(out, "</html>");
    out
}

pub(crate) fn render_json(model: &Model) -> String {
    let mut out = String::with_capacity(140_000);
    let mut derajat: HashMap<&str, usize> = HashMap::new();
    for edge in &model.edges {
        *derajat.entry(edge.from.as_str()).or_insert(0) += 1;
        *derajat.entry(edge.to.as_str()).or_insert(0) += 1;
    }
    let product_of: HashMap<&str, &str> = model
        .features
        .iter()
        .map(|f| (f.id.as_str(), f.product.as_str()))
        .collect();
    let app_of: HashMap<&str, &str> = model
        .features
        .iter()
        .map(|f| (f.id.as_str(), f.app.as_str()))
        .collect();
    let product_edges = agregasi_edges(&model.edges, &product_of);
    let app_edges = agregasi_edges(&model.edges, &app_of);

    out.push_str("{\"services\":[");
    for (i, service) in model.services.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        let _ = write!(
            out,
            "{{\"key\":{},\"name\":{},\"stack\":{},\"description\":{},\"folder\":{},\"docs\":{},\"ikon\":{}}}",
            json_str(&service.key),
            json_str(&service.name),
            json_str(&service.stack),
            json_str(&service.description),
            json_str(&service.folder),
            json_arr(&service.docs),
            json_str(&service.ikon)
        );
    }

    out.push_str("],\"products\":[");
    for (i, produk) in model.products.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        let fitur_produk: Vec<&Feature> = model
            .features
            .iter()
            .filter(|f| f.product == produk.id)
            .collect();
        let subfeatures: Vec<String> = fitur_produk.iter().map(|f| f.id.clone()).collect();
        let mut apps: Vec<String> = Vec::new();
        for fitur in &fitur_produk {
            if !apps.contains(&fitur.app) {
                apps.push(fitur.app.clone());
            }
        }
        let _ = write!(
            out,
            "{{\"id\":{},\"name\":{},\"actor\":{},\"outcome\":{},\"summary\":{},\"docs\":{},\"notes\":{},\"status\":{},\"counts\":{},\"subfeatures\":{},\"apps\":{}}}",
            json_str(&produk.id),
            json_str(&produk.name),
            json_str(&produk.actor),
            json_str(&produk.outcome),
            json_str(&produk.summary),
            json_arr(&produk.docs),
            json_str(&produk.notes),
            json_str(&hitung_status(&fitur_produk, &model.config)),
            json_counts(&fitur_produk, &model.config),
            json_arr(&subfeatures),
            json_arr(&apps)
        );
    }

    out.push_str("],\"apps\":[");
    for (i, aplikasi) in model.apps.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        let fitur_app: Vec<&Feature> = model
            .features
            .iter()
            .filter(|f| f.app == aplikasi.id)
            .collect();
        let subfeatures: Vec<String> = fitur_app.iter().map(|f| f.id.clone()).collect();
        let mut products: Vec<String> = Vec::new();
        let mut operations: Vec<String> = Vec::new();
        for fitur in &fitur_app {
            if !products.contains(&fitur.product) {
                products.push(fitur.product.clone());
            }
            for operasi in operasi_dari(&fitur.entry) {
                if !operations.contains(&operasi) {
                    operations.push(operasi);
                }
            }
        }
        let _ = write!(
            out,
            "{{\"id\":{},\"service\":{},\"name\":{},\"kind\":{},\"folder\":{},\"description\":{},\"docs\":{},\"ikon\":{},\"status\":{},\"counts\":{},\"products\":{},\"subfeatures\":{},\"operations\":{}}}",
            json_str(&aplikasi.id),
            json_str(&aplikasi.service),
            json_str(&aplikasi.name),
            json_str(&aplikasi.kind),
            json_str(&aplikasi.folder),
            json_str(&aplikasi.description),
            json_arr(&aplikasi.docs),
            json_str(&aplikasi.ikon),
            json_str(&hitung_status(&fitur_app, &model.config)),
            json_counts(&fitur_app, &model.config),
            json_arr(&products),
            json_arr(&subfeatures),
            json_arr(&operations)
        );
    }

    out.push_str("],\"features\":[");
    for (i, fitur) in model.features.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        let operations = operasi_dari(&fitur.entry);
        let _ = write!(
            out,
            "{{\"id\":{},\"product\":{},\"app\":{},\"service\":{},\"name\":{},\"status\":{},\"summary\":{},\"how\":{},\"entry\":{},\"operations\":{},\"evidence\":{},\"docs\":{},\"notes\":{},\"degree\":{}}}",
            json_str(&fitur.id),
            json_str(&fitur.product),
            json_str(&fitur.app),
            json_str(&fitur.service),
            json_str(&fitur.name),
            json_str(&fitur.status),
            json_str(&fitur.summary),
            json_str(&fitur.how),
            json_str(&fitur.entry),
            json_arr(&operations),
            json_arr(&fitur.evidence),
            json_arr(&fitur.docs),
            json_str(&fitur.notes),
            derajat.get(fitur.id.as_str()).copied().unwrap_or(0)
        );
    }

    out.push_str("],\"edges\":[");
    for (i, edge) in model.edges.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        let _ = write!(
            out,
            "{{\"from\":{},\"to\":{},\"kind\":{},\"label\":{}}}",
            json_str(&edge.from),
            json_str(&edge.to),
            json_str(&edge.kind),
            json_str(&edge.label)
        );
    }

    out.push_str("],\"productEdges\":[");
    for (i, edge) in product_edges.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        let _ = write!(
            out,
            "{{\"from\":{},\"to\":{},\"kind\":{},\"label\":{}}}",
            json_str(&edge.from),
            json_str(&edge.to),
            json_str(&edge.kind),
            json_str(&edge.label)
        );
    }

    out.push_str("],\"appEdges\":[");
    for (i, edge) in app_edges.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        let _ = write!(
            out,
            "{{\"from\":{},\"to\":{},\"kind\":{},\"label\":{}}}",
            json_str(&edge.from),
            json_str(&edge.to),
            json_str(&edge.kind),
            json_str(&edge.label)
        );
    }

    out.push_str("],\"statuses\":[");
    for (i, status) in model.config.statuses.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        out.push_str(&json_str(status));
    }
    out.push_str("],\"kinds\":[");
    for (i, kind) in model.config.kinds.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        out.push_str(&json_str(kind));
    }
    out.push_str("],\"peran\":{");
    for (i, nama) in ["utama", "separuh", "mati"].iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        let _ = write!(
            out,
            "{}:{}",
            json_str(nama),
            json_str(model.config.status_peran(nama))
        );
    }
    out.push_str("},\"aktorIkon\":{");
    for (i, (aktor, ikon)) in model.config.aktor_ikon.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        let _ = write!(out, "{}:{}", json_str(aktor), json_str(ikon));
    }
    let _ = write!(
        out,
        "}},\"merek\":{},\"judul\":{},\"papanJudul\":{}",
        json_str(&model.config.merek),
        json_str(&model.config.judul),
        json_str(&model.config.judul_papan)
    );
    // Papan pekerjaan: kosakata status tugas dan daftarnya. Repo yang belum memakai `tugas.tsv`
    // mengirim daftar kosong, dan halaman tidak merender panelnya sama sekali.
    out.push_str(",\"statusTugas\":[");
    for (i, status) in model.config.status_tugas.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        out.push_str(&json_str(status));
    }
    out.push_str("],\"tugas\":[");
    for (i, t) in model.tugas.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        let _ = write!(
            out,
            "{{\"id\":{},\"subfitur\":{},\"judul\":{},\"status\":{},\"progres\":{},\"pemegang\":{},\"berkas\":{},\"catatan\":{},\"baris\":{}}}",
            json_str(&t.id),
            json_str(&t.subfitur),
            json_str(&t.judul),
            json_str(&t.status),
            t.progres,
            json_str(&t.pemegang),
            json_str(&t.berkas),
            json_str(&t.notes),
            t.line
        );
    }
    out.push_str("]}");
    out
}

/// Naikkan huruf pertama sebuah label untuk tampilan; bukan untuk perbandingan data.
pub(crate) fn huruf_pertama_besar(nilai: &str) -> String {
    let mut karakter: Vec<char> = nilai.chars().collect();
    if let Some(huruf) = karakter.first_mut() {
        *huruf = hurup_naik(*huruf);
    }
    karakter.into_iter().collect()
}

pub(crate) fn hurup_naik(c: char) -> char {
    c.to_uppercase().next().unwrap_or(c)
}

pub(crate) fn operasi_dari(entry: &str) -> Vec<String> {
    entry
        .split(';')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(ToString::to_string)
        .collect()
}

/// Status sebuah produk/aplikasi DITURUNKAN dari sub-fiturnya, tidak pernah ditulis tangan.
/// Nama status yang dipakai diambil dari config, jadi peralatannya tidak tahu kosakata repo mana pun.
pub(crate) fn hitung_status(features: &[&Feature], config: &Config) -> String {
    let jumlah = features.len();
    if jumlah == 0 {
        // Produk atau aplikasi yang belum punya sub-fitur sama sekali tetap butuh label.
        // Dulu string kosong, jadi badge-nya hilang tanpa penjelasan. Yang jujur adalah peran
        // "mati": tidak ada satu pun hal yang berjalan di sini.
        return config.status_peran("mati").to_string();
    }
    let hitung = |peran: Peran| {
        features
            .iter()
            .filter(|f| peran_status(&f.status, config) == peran)
            .count()
    };
    let seragam = |peran: Peran| hitung(peran) == jumlah;
    if seragam(Peran::Utama) {
        config.status_peran("utama").to_string()
    } else if seragam(Peran::Mati) {
        config.status_peran("mati").to_string()
    } else if seragam(Peran::Belum) {
        // Tidak ada peran khusus untuk "belum": pakai status yang bukan utama/separuh/mati.
        features[0].status.clone()
    } else {
        config.status_peran("separuh").to_string()
    }
}

/// Hitungan per status, sebagai pasangan `[status, jumlah]` seurut daftar `status` di config.
pub(crate) fn json_counts(features: &[&Feature], config: &Config) -> String {
    let pasangan: Vec<String> = config
        .statuses
        .iter()
        .map(|status| {
            let ada = features.iter().filter(|f| &f.status == status).count();
            format!("[{},{}]", json_str(status), ada)
        })
        .collect();
    format!("[{}]", pasangan.join(","))
}

pub(crate) fn agregasi_edges(edges: &[Edge], peta: &HashMap<&str, &str>) -> Vec<AggEdge> {
    let mut agregat: BTreeMap<(String, String, String), (usize, String)> = BTreeMap::new();
    for edge in edges {
        let from = match peta.get(edge.from.as_str()) {
            Some(nilai) => *nilai,
            None => continue,
        };
        let to = match peta.get(edge.to.as_str()) {
            Some(nilai) => *nilai,
            None => continue,
        };
        if from == to {
            continue;
        }
        let kunci = (from.to_string(), to.to_string(), edge.kind.clone());
        let entri = agregat
            .entry(kunci)
            .or_insert_with(|| (0, edge.label.clone()));
        entri.0 += 1;
    }
    agregat
        .into_iter()
        .map(|((from, to, kind), (jumlah, label))| AggEdge {
            from,
            to,
            kind,
            label: if jumlah > 1 {
                format!("{jumlah} relasi")
            } else {
                label
            },
        })
        .collect()
}

pub(crate) fn json_arr(items: &[String]) -> String {
    let mut out = String::from("[");
    for (i, item) in items.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        out.push_str(&json_str(item));
    }
    out.push(']');
    out
}

pub(crate) fn json_str(nilai: &str) -> String {
    let mut out = String::with_capacity(nilai.len() + 2);
    out.push('"');
    for karakter in nilai.chars() {
        match karakter {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '<' => out.push_str("\\u003c"),
            '>' => out.push_str("\\u003e"),
            '&' => out.push_str("\\u0026"),
            lain if (lain as u32) < 0x20 => {
                let _ = write!(out, "\\u{:04x}", lain as u32);
            }
            lain => out.push(lain),
        }
    }
    out.push('"');
    out
}

pub(crate) fn esc(nilai: &str) -> String {
    let mut hasil = String::with_capacity(nilai.len());
    for karakter in nilai.chars() {
        match karakter {
            '&' => hasil.push_str("&amp;"),
            '<' => hasil.push_str("&lt;"),
            '>' => hasil.push_str("&gt;"),
            '"' => hasil.push_str("&quot;"),
            '\'' => hasil.push_str("&#39;"),
            lain => hasil.push(lain),
        }
    }
    hasil
}



#[cfg(test)]
mod tests {
    use super::*;
    use crate::uji::*;
    #[test]
    fn hitung_status_mengikuti_subfitur() {
        let a = fitur("a", "app", "aktif", "tools/feature-view/Cargo.toml");
        let b = fitur("b", "app", "aktif", "tools/feature-view/Cargo.toml");
        let c = fitur("c", "app", "sebagian", "tools/feature-view/Cargo.toml");
        assert_eq!(hitung_status(&[&a, &b], &config_uji()), "aktif");
        assert_eq!(hitung_status(&[&a, &c], &config_uji()), "sebagian");
        // Nama status datang dari config, jadi peran yang menentukan, bukan kata-katanya.
        let asing = Config {
            statuses: vec!["selesai".to_string(), "berjalan".to_string()],
            kinds: vec!["alur".to_string()],
            aktor_ikon: vec![],
            merek: "X".to_string(),
            judul: "Y".to_string(),
            bahasa: "id".to_string(),
            minimum_fitur: 1,
            peran: [
                ("utama", "selesai".to_string()),
                ("separuh", "berjalan".to_string()),
                ("mati", "berjalan".to_string()),
            ],
            status_tugas: vec![],
            judul_papan: String::new(),
        };
        let x = fitur("x", "app", "selesai", "tools/feature-view/Cargo.toml");
        assert_eq!(hitung_status(&[&x], &asing), "selesai");
    }

    #[test]
    fn json_meng_escape_tag_script() {
        let hasil = json_str("</script><b>&");
        assert!(hasil.contains("\\u003c/script\\u003e"));
        assert!(hasil.contains("\\u0026"));
        assert!(!hasil.contains("</script>"));
    }

    #[test]
    fn agregat_tanpa_subfitur_punya_label_peran_mati() {
        // Validator tidak mewajibkan setiap produk punya sub-fitur, jadi produk yatim itu sah
        // dan halamannya tetap harus menampilkan sesuatu yang benar: tidak ada yang berjalan.
        assert_eq!(hitung_status(&[], &config_uji()), "dimatikan");
        let asing = Config {
            statuses: vec!["bersih".to_string(), "jalan".to_string()],
            kinds: vec!["alur".to_string()],
            aktor_ikon: vec![],
            merek: "X".to_string(),
            judul: "Y".to_string(),
            bahasa: "id".to_string(),
            minimum_fitur: 1,
            peran: [
                ("utama", "jalan".to_string()),
                ("separuh", "jalan".to_string()),
                ("mati", "bersih".to_string()),
            ],
            status_tugas: vec![],
            judul_papan: String::new(),
        };
        assert_eq!(hitung_status(&[], &asing), "bersih");
    }

    #[test]
    fn kanvas_interaktif_bukan_peran_gambar() {
        let model = model_uji(
            vec![
                fitur("a", "app", "aktif", "tools/feature-view/Cargo.toml"),
                fitur("b", "app", "aktif", "tools/feature-view/Cargo.toml"),
            ],
            vec![edge("a", "b")],
        );
        let html = render_html(&model, "satu-dua", "gaya-uji", "skrip-uji");
        // Kanvas utama diisi kartu yang bisa diklik dan difokus. `role="img"` membuat
        // seluruh turunannya presentational, jadi anak ber-`role="button"` hilang
        // bagi pembaca layar.
        assert!(
            !html.contains("id=\"jejaring\" role=\"img\""),
            "kanvas yang berisi kartu interaktif tidak boleh berperan sebagai gambar"
        );
        assert!(
            html.contains("id=\"jejaring\" role=\"group\""),
            "kanvas tetap harus punya label yang bisa dibaca: {html}"
        );
        assert!(
            html.contains("id=\"minimap\" role=\"img\""),
            "peta mini memang gambar tanpa anak interaktif, perannya tetap img"
        );
    }

    #[test]
    fn render_deterministik() {
        let model = model_uji(
            vec![
                fitur("a", "app", "aktif", "tools/feature-view/Cargo.toml"),
                fitur("b", "app", "aktif", "tools/feature-view/Cargo.toml"),
            ],
            vec![edge("a", "b")],
        );
        assert_eq!(
            render_html(&model, "satu-dua", "gaya-uji", "skrip-uji"),
            render_html(&model, "satu-dua", "gaya-uji", "skrip-uji")
        );
    }

    #[test]
    fn render_memakai_aset_dan_cap_yang_diberikan() {
        let model = model_uji(
            vec![
                fitur("a", "app", "aktif", "tools/feature-view/Cargo.toml"),
                fitur("b", "app", "aktif", "tools/feature-view/Cargo.toml"),
            ],
            vec![edge("a", "b")],
        );
        let html = render_html(&model, "cap-abc", "GAYA-123", "SKRIP-456");
        assert!(html.contains("GAYA-123"), "css dari argumen tidak dipakai");
        assert!(html.contains("SKRIP-456"), "js dari argumen tidak dipakai");
        assert!(html.contains("data-cap=\"cap-abc\""), "cap tidak tertanam di chip");
        assert!(!html.contains("<style>{css}</style>"), "penyetak css tidak terganti");
    }

    #[test]
    fn merek_config_jatuh_ke_frasa_netral_ketika_config_hilang() {
        let kosong = std::env::temp_dir().join(format!("fv-merek-{}", std::process::id()));
        assert_eq!(merek_config(&kosong), "peta fitur");
        fs::create_dir_all(kosong.join("docs/view")).expect("buat folder uji");
        fs::write(
            kosong.join(CONFIG_FILE),
            "merek|Repo Asing\njudul|Peta\nbahasa|en\nstatus|aktif\n\
             status_peran_utama|aktif\nstatus_peran_separuh|aktif\nstatus_peran_mati|aktif\n\
             jenis_relasi|alur\naktor_ikon|A:aplikasi\nminimum_fitur_per_service|1\n",
        )
        .expect("tulis config uji");
        assert_eq!(merek_config(&kosong), "Repo Asing");
        let _ = fs::remove_dir_all(&kosong);
    }

    #[test]
    fn halaman_belum_generate_tunjuk_path_dan_tombol_regenerate() {
        let html = halaman_belum_generate(
            Path::new("docs/view/index.html"),
            "Repo Uji",
            Path::new("/home/orang/kerja"),
        );
        assert!(html.contains("Peta belum di-generate"));
        assert!(html.contains("docs/view/index.html"), "path sasaran tidak shown");
        assert!(html.contains("POST"), "tombolnya tidak memanggil /regenerate dengan POST");
        assert!(html.contains("/regenerate"));
        assert!(
            !html.contains("<h1>Peta Fitur"),
            "bocor ke halaman penuh padahal filenya belum ada"
        );
        // Petunjuk terminal harus menyebut root yang sungguh dibaca server. Bentuk pendek
        // `cargo run -- generate` hanya benar kalau orang itu kebetulan berada di root crate;
        // di repo induk yang menyimpan alat di folder `tools/...` perintah itu membaca
        // `tools/feature-view/docs/view/config.tsv` dan gagal.
        assert!(
            html.contains("generate --root /home/orang/kerja"),
            "petunjuk harus mengulang root server: {html}"
        );
        assert!(
            !html.contains("di folder alat ini"),
            "petunjuk tidak boleh mengasumsikan direktori kerja pemakai"
        );
    }

}
