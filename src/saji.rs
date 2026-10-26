//! Antarmuka proses: server HTTP kecil untuk membaca ulang manifest tanpa bangun ulang, dan
//! penghentian pendengar yang hanya boleh mematikan proses feature-view.

use std::fs;
use std::io::Read;
use std::io::Write;
use std::net::TcpListener;
use std::net::TcpStream;
use std::path::Path;
use std::process::ExitCode;
use std::time::Duration;
use crate::data::*;
use crate::manifest::*;
use crate::halaman::*;

pub(crate) const PORT_BAWAAN: u16 = 8787;

pub(crate) const BATAS_BACA_PERMINTAAN: usize = 8192;

pub(crate) fn sajikan(root: &Path, port: u16) -> Result<ExitCode, String> {
    let alamat = format!("127.0.0.1:{port}");
    let dengar = TcpListener::bind(alamat.as_str()).map_err(|e| {
        format!(
            "GAGAL membuka {alamat}: {e}\nPort mungkin sudah dipakai; jalankan ulang dengan --port <angka>."
        )
    })?;
    println!("OK: serve di http://{alamat}/");
    println!("    root     : {}", root.display());
    println!("    halaman  : dibaca dari docs/view/index.html seperti adanya di disk");
    println!("    data     : /stamp membaca ulang ketujuh TSV tiap permintaan");
    println!("    tulis    : hanya POST /regenerate dan POST /hapus, keduanya ke docs/view/index.html saja");
    println!("    berhenti : Ctrl+C, atau jalankan: feature-view stop");
    for panggilan in dengar.incoming() {
        let mut stream = match panggilan {
            Ok(stream) => stream,
            Err(e) => {
                eprintln!("GAGAL menerima koneksi: {e}");
                continue;
            }
        };
        // Browser sering membuka koneksi lalu meninggalkannya (favicon, prabaca).
        // Tanpa batas baca, satu koneksi menggantung membekukan seluruh server.
        let _ = stream.set_read_timeout(Some(Duration::from_millis(2000)));
        if let Err(e) = layani(&mut stream, root) {
            eprintln!("GAGAL melayani: {e}");
        }
    }
    Ok(ExitCode::SUCCESS)
}

/// Ambil keluaran sebuah perintah Windows sebagai teks; None kalau gagal jalan
/// atau kode keluarnya bukan 0.
pub(crate) fn jalankan_perintah(perintah: &str, argumen: &[&str]) -> Option<String> {
    let keluaran = std::process::Command::new(perintah)
        .args(argumen)
        .output()
        .ok()?;
    if !keluaran.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&keluaran.stdout).into_owned())
}

/// Daftar proses seluruh mesin, satu panggilan. Dipakai `hentikan` supaya nama PID tidak
/// perlu diminta satu per satu ke tasklist.
pub(crate) fn tasklist_semua() -> Option<String> {
    jalankan_perintah("tasklist", &["/FO", "CSV", "/NH"])
}

/// Semua pendengar TCP pada keluaran `netstat -ano -p TCP`, sebagai (port, PID). Baris
/// pendengar dikenal dari alamat jarak jauh yang `:0`, jadi TIME_WAIT/ESTABLISHED tidak ikut.
pub(crate) fn pendengar_semua(teks: &str) -> Vec<(u16, u32)> {
    let mut hasil: Vec<(u16, u32)> = Vec::new();
    for baris in teks.lines() {
        let kolom: Vec<&str> = baris.split_whitespace().collect();
        if kolom.len() < 5 || kolom[0] != "TCP" || !kolom[2].ends_with(":0") {
            continue;
        }
        let port = match kolom[1].rsplit(':').next().and_then(|p| p.parse::<u16>().ok()) {
            Some(port) => port,
            None => continue,
        };
        let pid = match kolom[4].parse::<u32>() {
            Ok(pid) if pid != 0 => pid,
            _ => continue,
        };
        if !hasil.contains(&(port, pid)) {
            hasil.push((port, pid));
        }
    }
    hasil
}

/// PID yang mendengarkan pada port ini. Kolom State tidak dipakai sebagai penyaring karena
/// Windows menerjemahkannya; bentuk baris dikenal dari alamat jarak jauh `:0` (lihat
/// `pendengar_semua`).
pub(crate) fn pid_pendengar(teks: &str, port: u16) -> Vec<u32> {
    pendengar_semua(teks)
        .into_iter()
        .filter(|(p, _)| *p == port)
        .map(|(_, pid)| pid)
        .collect()
}

/// Ambil satu baris `tasklist /FO CSV /NH` lalu baca kolom kedua (PID) dengan benar: cocok
/// dengan angka di mana pun bukan pemeriksaan yang aman.
fn baris_csv(baris: &str) -> Vec<String> {
    if !baris.trim_start().starts_with('"') {
        return Vec::new();
    }
    baris
        .trim()
        .trim_matches('"')
        .split("\",\"")
        .map(|k| k.trim_matches('"').to_string())
        .collect()
}

/// Nama proses milik PID dari keluaran tasklist; None kalau PID itu tidak ada di daftar.
pub(crate) fn nama_dari(teks_tasklist: &str, pid: u32) -> Option<String> {
    for baris in teks_tasklist.lines() {
        let kolom = baris_csv(baris);
        if kolom.len() < 2 {
            continue;
        }
        if kolom[1].parse::<u32>() == Ok(pid) && !kolom[0].is_empty() {
            return Some(kolom[0].clone());
        }
    }
    None
}

/// Proses milik alat ini sendiri, dari satu panggilan tasklist: nama diawali `feature-view`.
pub(crate) fn proses_alat(teks_tasklist: &str) -> Vec<(String, u32)> {
    let mut hasil = Vec::new();
    for baris in teks_tasklist.lines() {
        let kolom = baris_csv(baris);
        if kolom.len() < 2 || !kolom[0].starts_with("feature-view") {
            continue;
        }
        if let Ok(pid) = kolom[1].parse::<u32>() {
            hasil.push((kolom[0].clone(), pid));
        }
    }
    hasil
}

pub(crate) fn hentikan(port: u16) -> Result<ExitCode, String> {
    let netstat = jalankan_perintah("netstat", &["-ano", "-p", "TCP"]).ok_or_else(|| {
        "GAGAL menjalankan netstat: perintah stop ini khusus Windows.".to_string()
    })?;
    let tasklist = tasklist_semua().ok_or_else(|| {
        "GAGAL menjalankan tasklist: perintah stop ini khusus Windows.".to_string()
    })?;
    let pendengar = pid_pendengar(&netstat, port);
    if pendengar.is_empty() {
        // Port yang diminta sepi bukan berarti tidak ada server. `serve --port 9999` tetap
        // hidup, dan dulu kasus ini ditutup dengan "port sudah bebas" + exit 0 — laporan
        // sukses yang salah.
        let milik = proses_alat(&tasklist);
        let memegang: Vec<String> = pendengar_semua(&netstat)
            .into_iter()
            .filter(|(_, pid)| milik.iter().any(|(_, m_pid)| m_pid == pid))
            .map(|(p, pid)| format!("{p} (PID {pid})"))
            .collect();
        if memegang.is_empty() {
            println!(
                "OK: 127.0.0.1:{port} bebas dan tidak ada proses feature-view yang memegang port lain."
            );
            return Ok(ExitCode::SUCCESS);
        }
        for pegang in &memegang {
            println!("AWAS: feature-view masih mendengarkan {pegang}, bukan port {port} yang diminta.");
        }
        println!("     Jalankan ulang dengan port itu, mis. feature-view stop --port <angka di atas>.");
        return Ok(ExitCode::from(1));
    }
    let mut berhenti = 0usize;
    let mut bukan_miliknya = 0usize;
    for pid in pendengar {
        let nama = nama_dari(&tasklist, pid).unwrap_or_else(|| "(tidak terbaca)".to_string());
        if !nama.starts_with("feature-view") {
            println!("BIAR: 127.0.0.1:{port} dipegang PID {pid} = {nama}, bukan feature-view. Tidak saya sentuh.");
            bukan_miliknya += 1;
            continue;
        }
        let kode = std::process::Command::new("taskkill")
            .args(["/PID", &pid.to_string(), "/F"])
            .stdout(std::process::Stdio::null())
            .status();
        match kode {
            Ok(status) if status.success() => {
                println!("OK: PID {pid} ({nama}) dihentikan, port {port} bebas.");
                berhenti += 1;
            }
            Ok(status) => {
                println!(
                    "GAGAL: taskkill menolak PID {pid} (kode {}). Jalankan sendiri: taskkill /PID {pid} /F",
                    status.code().unwrap_or(-1)
                );
                bukan_miliknya += 1;
            }
            Err(e) => return Err(format!("GAGAL menjalankan taskkill: {e}")),
        }
    }
    if berhenti == 0 {
        return Ok(ExitCode::from(1));
    }
    if bukan_miliknya > 0 {
        println!("Catatan: masih ada pemegang port lain yang bukan feature-view.");
    }
    Ok(ExitCode::SUCCESS)
}

pub(crate) fn layani(stream: &mut TcpStream, root: &Path) -> Result<(), String> {
    let (metode, path) = match baca_permintaan(stream)? {
        Some(permintaan) => permintaan,
        None => return Ok(()),
    };
    // Dua aksi tulis, hanya POST, hanya lewat proses alat ini sendiri. Path sasaran
    // dipakai dari konstanta OUTPUT_FILE, tidak pernah dari permintaan, jadi tidak ada
    // cara mengarahkan tombol ke file lain (misalnya manifest TSV).
    if path == "/regenerate" || path == "/hapus" {
        if metode != "POST" {
            println!("[{metode} {path}] 405");
            return respons(
                stream,
                "405 Method Not Allowed",
                "text/plain; charset=utf-8",
                "Aksi tulis butuh POST.\n".as_bytes(),
            );
        }
        let hasil = if path == "/regenerate" {
            generate_lewat_http(root)
        } else {
            hapus_lewat_http(root)
        };
        match hasil {
            Ok(pesan) => {
                println!("[{metode} {path}] {pesan}");
                return respons(
                    stream,
                    "200 OK",
                    "text/plain; charset=utf-8",
                    format!("{pesan}\n").as_bytes(),
                );
            }
            Err(pesan) => {
                println!("[{metode} {path}] GAGAL: {pesan}");
                return respons(
                    stream,
                    "500 Kesalahan Server",
                    "text/plain; charset=utf-8",
                    format!("GAGAL: {pesan}\n").as_bytes(),
                );
            }
        }
    }

    if metode != "GET" && metode != "HEAD" {
        println!("[{metode} {path}] 405");
        return respons(
            stream,
            "405 Method Not Allowed",
            "text/plain; charset=utf-8",
            format!("GAGAL: metode {metode} tidak diterima, gunakan GET.\n").as_bytes(),
        );
    }

    match path.as_str() {
        // Opsi B: halaman adalah isi file, bukan hasil susun ulang. Tanpa ini tombol
        // "Hapus hasil" tidak pernah kelihatan efeknya di browser.
        "/" | "/index.html" => {
            let keluaran = root.join(OUTPUT_FILE);
            match fs::read(&keluaran) {
                Ok(isi) => {
                    println!(
                        "[{metode} {path}] {} byte disaji dari disk",
                        isi.len()
                    );
                    respons(stream, "200 OK", "text/html; charset=utf-8", &isi)
                }
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                    println!("[{metode} {path}] belum di-generate, file tidak ada");
                    let isi = halaman_belum_generate(&keluaran, &merek_config(root), root);
                    respons(stream, "200 OK", "text/html; charset=utf-8", isi.as_bytes())
                }
                Err(e) => {
                    let pesan = format!("GAGAL membaca {}: {e}", keluaran.display());
                    println!("[{metode} {path}] {pesan}");
                    respons(stream, "500 Kesalahan Server", "text/plain; charset=utf-8", pesan.as_bytes())
                }
            }
        }
        "/stamp" => {
            // Permintaan ini tidak boleh gagal karena isi manifest: halaman memakainya
            // untuk tahu apakah ada perubahan, jadi laporan galat justru hasilnya.
            let (cap, daftar) = match cap_manifest(root) {
                Ok(cap) => {
                    let daftar = match muat_model(root) {
                        Ok(_) => Vec::new(),
                        Err(LoadError::Manifest(daftar)) => daftar,
                        Err(LoadError::Io(pesan)) => vec![pesan],
                    };
                    (cap, daftar)
                }
                Err(pesan) => (String::new(), vec![pesan]),
            };
            let valid = daftar.is_empty();
            let isi = format!(
                "{{\"cap\":{},\"valid\":{},\"galat\":{}}}",
                json_str(&cap),
                if valid { "true" } else { "false" },
                json_arr(&daftar)
            );
            respons(stream, "200 OK", "application/json; charset=utf-8", isi.as_bytes())
        }
        // Browser selalu meminta ikon; jawab kosong supaya tidak muncul sebagai
        // galat 404 di console yang sebenarnya bukan masalah peta.
        "/favicon.ico" => {
            let _ = stream.write_all(b"HTTP/1.1 204 No Content\r\nConnection: close\r\n\r\n");
            Ok(())
        }
        lain => {
            println!("[GET {lain}] 404");
            respons(
                stream,
                "404 Tidak Ditemukan",
                "text/plain; charset=utf-8",
                format!("GAGAL: path tidak dikenal: {lain}\nPath yang ada: /, /stamp\n").as_bytes(),
            )
        }
    }
}

pub(crate) fn baca_permintaan(stream: &mut TcpStream) -> Result<Option<(String, String)>, String> {
    let mut buffer = Vec::new();
    let mut potong = [0u8; 1024];
    loop {
        let dibaca = match stream.read(&mut potong) {
            Ok(dibaca) => dibaca,
            Err(e) => {
                // Chrome membuka koneksi prabaca lalu meninggalkannya diam. Itu bukan
                // kegagalan: kalau belum ada satu byte pun, anggap permintaan kosong.
                let dibiarkan = matches!(
                    e.kind(),
                    std::io::ErrorKind::TimedOut | std::io::ErrorKind::WouldBlock
                );
                if dibiarkan && buffer.is_empty() {
                    return Ok(None);
                }
                return Err(format!("GAGAL membaca permintaan: {e}"));
            }
        };
        if dibaca == 0 {
            break;
        }
        buffer.extend_from_slice(&potong[..dibaca]);
        if buffer.windows(4).any(|w| w == b"\r\n\r\n") || buffer.len() >= BATAS_BACA_PERMINTAAN {
            break;
        }
    }
    if buffer.is_empty() {
        return Ok(None);
    }
    let teks = String::from_utf8_lossy(&buffer).into_owned();
    Ok(uraikan_permintaan(&teks))
}

pub(crate) fn uraikan_permintaan(teks: &str) -> Option<(String, String)> {
    let baris = teks.lines().next().unwrap_or_default();
    let bagian: Vec<&str> = baris.split_whitespace().collect();
    if bagian.len() < 2 {
        return None;
    }
    // Query string dibuang supaya /stamp?cek=123 tidak jatuh ke 404.
    let path = bagian[1].split('?').next().unwrap_or(bagian[1]).to_string();
    Some((bagian[0].to_string(), path))
}

pub(crate) fn respons(stream: &mut TcpStream, kode: &str, jenis: &str, isi: &[u8]) -> Result<(), String> {
    let kepala = format!(
        "HTTP/1.1 {kode}\r\nContent-Type: {jenis}\r\nContent-Length: {}\r\nCache-Control: no-store\r\nConnection: close\r\n\r\n",
        isi.len()
    );
    // Koneksi yang sudah ditutup browser bukan kesalahan server: tulis, lalu lanjut.
    if stream.write_all(kepala.as_bytes()).is_err() {
        return Ok(());
    }
    let _ = stream.write_all(isi);
    let _ = stream.flush();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn uraikan_permintaan_memotong_query_string() {
        assert_eq!(
            uraikan_permintaan("GET /stamp?cek=1729 HTTP/1.1\r\nHost: x\r\n\r\n"),
            Some(("GET".to_string(), "/stamp".to_string()))
        );
        assert_eq!(
            uraikan_permintaan("HEAD / HTTP/1.1\r\n\r\n"),
            Some(("HEAD".to_string(), "/".to_string()))
        );
    }

    #[test]
    fn uraikan_permintaan_menolak_baris_cacat() {
        assert_eq!(uraikan_permintaan(""), None);
        assert_eq!(uraikan_permintaan("GET\r\n\r\n"), None);
    }

    #[test]
    fn pid_pendengar_ambil_baris_penyandang_saja() {
        let teks = "  Proto  Local Address          Foreign Address        State           PID
  TCP    127.0.0.1:8787         0.0.0.0:0              LISTENING       1880
  TCP    127.0.0.1:8787         127.0.0.1:49998        TIME_WAIT       0
  TCP    127.0.0.1:8787         127.0.0.1:50100        ESTABLISHED     1880
  TCP    127.0.0.1:8788         0.0.0.0:0              LISTENING       999
  TCP    127.0.0.1:87870        0.0.0.0:0              LISTENING       777
  UDP    127.0.0.1:8787         *:*                                    555
";
        assert_eq!(pid_pendengar(teks, 8787), vec![1880]);
        assert_eq!(pid_pendengar(teks, 8788), vec![999]);
        assert!(pid_pendengar(teks, 8790).is_empty());
    }

    #[test]
    fn pid_pendengar_tidak_bergantung_kata_listening() {
        // Windows menerjemahkan kolom State, jadi penyaringnya bukan kata itu.
        let teks = "  TCP    127.0.0.1:8787         0.0.0.0:0              MENYIMAK        4242\n";
        assert_eq!(pid_pendengar(teks, 8787), vec![4242]);
    }

    #[test]
    fn pid_pendengar_menolak_pid_nol_dan_baris_cacat() {
        let teks = "  TCP    127.0.0.1:8787         0.0.0.0:0              LISTENING       0
  TCP    pendek
";
        assert!(pid_pendengar(teks, 8787).is_empty());
    }

    #[test]
    fn pendengar_semua_ambil_semua_port_karena_stop_harus_tahu_server_lain() {
        let teks = "  TCP    127.0.0.1:8787         0.0.0.0:0              LISTENING       1880
  TCP    127.0.0.1:9999         0.0.0.0:0              MENYIMAK        4242
  TCP    127.0.0.1:8787         127.0.0.1:49998        TIME_WAIT       7
  UDP    127.0.0.1:7777         *:*                                    555
";
        let got = pendengar_semua(teks);
        assert!(got.contains(&(8787, 1880)), "{got:?}");
        assert!(got.contains(&(9999, 4242)), "{got:?}");
        assert_eq!(got.len(), 2, "hanya baris pendengar TCP yang boleh ikut: {got:?}");
    }

    #[test]
    fn nama_dari_dan_proses_alat_hanya_mengenal_kolom_pid() {
        let teks = "\"feature-view.exe\",\"1880\",\"Console\",\"1\",\"20.000 K\"
\"chrome.exe\",\"4242\",\"Console\",\"1\",\"300.000 K\"
baris sampah tanpa kutip
";
        assert_eq!(nama_dari(teks, 1880).as_deref(), Some("feature-view.exe"));
        assert_eq!(nama_dari(teks, 9999), None, "PID yang tidak ada tidak boleh dikarang namanya");
        assert_eq!(
            proses_alat(teks),
            vec![("feature-view.exe".to_string(), 1880)],
            "proses lain tidak boleh dianggap milik alat ini"
        );
    }

    #[test]
    fn nama_dari_tidak_tertipu_angka_di_luar_kolom_pid() {
        // Angka 1880 muncul di kolom memori baris chrome; pencocokan longgar akan memberi
        // nama yang salah dan stop akan mematikan proses orang lain.
        let teks = "\"chrome.exe\",\"4242\",\"Console\",\"1\",\"1.880.000 K\"\n";
        assert_eq!(nama_dari(teks, 1880), None);
        assert_eq!(proses_alat(teks), Vec::<(String, u32)>::new());
    }

}
