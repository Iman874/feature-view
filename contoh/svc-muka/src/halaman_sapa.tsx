// TampilSalam menulis salam ke dalam KotakSalam.
export function TampilSalam(props: { salam: string }) {
  return <KotakSalam value={props.salam} />;
}
