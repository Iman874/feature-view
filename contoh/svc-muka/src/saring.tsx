// SaringPengguna memotong daftar nama sampai BatasTampilan tertentu.
export const BatasTampilan = 20;

export function SaringPengguna(daftar: string[], batas: number = BatasTampilan) {
  return daftar.slice(0, batas);
}
