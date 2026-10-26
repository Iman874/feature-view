package main

// KirimUlang mengirim ulang salam yang gagal terkirim.
// Bagian antrean belum ditulis; sengaja tidak diberi nama agar jalur status `setengah`
// pada dataset contoh tetap terlihat kuning dan bukan hijau palsu.
func KirimUlang(coba int) bool {
	return coba < 3
}
